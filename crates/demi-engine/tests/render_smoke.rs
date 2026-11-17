//! T0.3: PDFium fijado se carga y renderiza la página 1 de un PDF de ejemplo.
//!
//! Requiere `cargo xtask pdfium fetch` (o `DEMI_PDFIUM_DIR` apuntando a la biblioteca).

mod common;

use common::pdfium_dir;
use demi_engine::pdfium_lib;
use demi_engine::pdfium_render::prelude::*;

/// PDF mínimo de una página (200×200 pt) con un cuadrado negro de 100×100 pt en el centro.
fn sample_pdf() -> Vec<u8> {
    let content = b"0 0 0 rg 50 50 100 100 re f\n";
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Contents 4 0 R >>".to_string(),
        format!(
            "<< /Length {} >>\nstream\n{}endstream",
            content.len(),
            String::from_utf8_lossy(content)
        ),
    ];
    let mut out = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::new();
    for (i, obj) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{obj}\nendobj\n", i + 1).as_bytes());
    }
    let xref = out.len();
    out.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for off in offsets {
        out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    out
}

#[test]
fn renders_first_page_with_pinned_pdfium() {
    let dir = pdfium_dir().expect("plataforma no soportada por pdfium-binaries");
    assert!(
        pdfium_lib::library_path(&dir).exists(),
        "No se encontró PDFium en {}. Ejecuta `cargo xtask pdfium fetch`.",
        dir.display()
    );
    let pdfium = pdfium_lib::bind(&dir).unwrap();

    let bytes = sample_pdf();
    let doc = pdfium.load_pdf_from_byte_slice(&bytes, None).unwrap();
    assert_eq!(doc.pages().len(), 1);

    let page = doc.pages().first().unwrap();
    let bitmap = page
        .render_with_config(&PdfRenderConfig::new().set_target_width(200))
        .unwrap();
    assert_eq!(bitmap.width(), 200);
    assert_eq!(bitmap.height(), 200);

    // 4 bytes por píxel (RGBA, ver FPDF_REVERSE_BYTE_ORDER). Centro = negro, esquina = blanco.
    let raw = bitmap.as_raw_bytes();
    let px = |x: usize, y: usize| {
        let i = (y * 200 + x) * 4;
        (raw[i], raw[i + 1], raw[i + 2])
    };
    let (r, g, b) = px(100, 100);
    assert!(
        r < 32 && g < 32 && b < 32,
        "el centro debería ser negro: {r},{g},{b}"
    );
    let (r, g, b) = px(5, 5);
    assert!(
        r > 223 && g > 223 && b > 223,
        "la esquina debería ser blanca: {r},{g},{b}"
    );
}
