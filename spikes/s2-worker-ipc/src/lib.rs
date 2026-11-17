//! Spike S2 de Demi: protocolo mínimo app ↔ worker (design.md §5).
//!
//! - Control: socket local (`interprocess`) con mensajes `postcard` precedidos de su longitud.
//! - Píxeles: anillo de ranuras en memoria compartida (archivo mapeado con `memmap2`);
//!   el worker escribe la tesela en una ranura y solo envía su número por el socket.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::io::{self, Read, Write};

use interprocess::local_socket::{GenericFilePath, GenericNamespaced, Name, prelude::*};
use serde::{Deserialize, Serialize};

/// Lado de una tesela, en píxeles.
pub const TILE: u32 = 512;
/// Bytes por ranura (RGBA).
pub const SLOT_BYTES: usize = (TILE * TILE * 4) as usize;
/// Número de ranuras del anillo.
pub const SLOTS: usize = 16;

/// Petición de la app al worker.
#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    /// Renderiza la tesela (tx, ty) de `page` con la página a `page_width_px` de ancho.
    Tile {
        id: u64,
        page: u32,
        tx: u32,
        ty: u32,
        page_width_px: u32,
        slot: u16,
    },
    /// Simula un fallo grave del motor (p. ej. un segfault de PDFium).
    Crash,
    /// Termina limpiamente.
    Shutdown,
}

/// Respuesta del worker a la app.
#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    /// Worker listo (PDFium cargado y documento abierto), con el token recibido.
    Hello { token: u64, pages: u32 },
    /// Tesela escrita en `slot`.
    TileReady {
        id: u64,
        slot: u16,
        width: u32,
        height: u32,
        render_us: u64,
        copy_us: u64,
        checksum: u64,
    },
    /// Error controlado.
    Error { id: u64, message: String },
}

/// Envía un mensaje: longitud `u32` LE + postcard.
pub fn send<T: Serialize, W: Write>(w: &mut W, msg: &T) -> io::Result<()> {
    let bytes = postcard::to_stdvec(msg).map_err(io::Error::other)?;
    w.write_all(&(bytes.len() as u32).to_le_bytes())?;
    w.write_all(&bytes)?;
    w.flush()
}

/// Recibe un mensaje (rechaza tamaños > 1 MiB: entrada no confiable).
pub fn recv<T: for<'de> Deserialize<'de>, R: Read>(r: &mut R) -> io::Result<T> {
    let mut len = [0u8; 4];
    r.read_exact(&mut len)?;
    let len = u32::from_le_bytes(len) as usize;
    if len > 1 << 20 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "mensaje demasiado grande",
        ));
    }
    let mut buf = vec![0; len];
    r.read_exact(&mut buf)?;
    postcard::from_bytes(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Nombre del socket: espacio de nombres del sistema si existe (Windows, Linux),
/// si no un archivo en el directorio temporal (macOS).
pub fn socket_name(id: &str) -> io::Result<Name<'static>> {
    if GenericNamespaced::is_supported() {
        format!("demi-s2-{id}.sock")
            .to_ns_name::<GenericNamespaced>()
            .map(|n| n.into_owned())
    } else {
        let p = std::env::temp_dir().join(format!("demi-s2-{id}.sock"));
        p.to_fs_name::<GenericFilePath>().map(|n| n.into_owned())
    }
}

/// Suma de comprobación simple (FNV-1a de 64 bits) para verificar la tesela del otro lado.
pub fn checksum(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// PDF de prueba de N páginas A4 con texto.
pub fn sample_pdf(pages: usize) -> Vec<u8> {
    let mut objects: Vec<Vec<u8>> = vec![Vec::new(), Vec::new()];
    objects.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec());
    let mut kids = Vec::new();
    for i in 0..pages {
        let mut c = format!("BT /F1 20 Tf 60 780 Td (Demi - pagina {}) Tj ET\n", i + 1);
        for line in 0..32 {
            c.push_str(&format!(
                "BT /F1 11 Tf 60 {} Td (Linea {} de la pagina {}: texto de relleno.) Tj ET\n",
                740 - line * 20,
                line + 1,
                i + 1
            ));
        }
        objects.push(format!("<< /Length {} >>\nstream\n{c}endstream", c.len()).into_bytes());
        let content = objects.len();
        objects.push(
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] \
                 /Resources << /Font << /F1 3 0 R >> >> /Contents {content} 0 R >>"
            )
            .into_bytes(),
        );
        kids.push(format!("{} 0 R", objects.len()));
    }
    objects[0] = b"<< /Type /Catalog /Pages 2 0 R >>".to_vec();
    objects[1] = format!(
        "<< /Type /Pages /Kids [{}] /Count {pages} >>",
        kids.join(" ")
    )
    .into_bytes();
    let mut out = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref = out.len();
    let n = objects.len() + 1;
    out.extend_from_slice(format!("xref\n0 {n}\n0000000000 65535 f \n").as_bytes());
    for off in offsets {
        out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!("trailer\n<< /Size {n} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n").as_bytes(),
    );
    out
}
