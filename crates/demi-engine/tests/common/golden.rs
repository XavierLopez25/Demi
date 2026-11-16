//! Comparación visual para la regresión de render (T0.5, ADR-07).
//!
//! Imágenes en escala de grises de 8 bits, PNG para guardarlas y SSIM por bloques
//! de 8×8 para compararlas (1,0 = idénticas).

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// Imagen en escala de grises.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Gray {
    pub width: u32,
    pub height: u32,
    pub px: Vec<u8>,
}

impl Gray {
    /// Convierte un búfer BGRA/BGRx de PDFium (4 bytes por píxel) a grises (luma BT.601).
    pub fn from_bgra(raw: &[u8], width: u32, height: u32) -> Self {
        let px = raw
            .chunks_exact(4)
            .map(|c| {
                let (b, g, r) = (c[0] as u32, c[1] as u32, c[2] as u32);
                ((299 * r + 587 * g + 114 * b + 500) / 1000) as u8
            })
            .collect();
        Self { width, height, px }
    }

    /// Lee un PNG en escala de grises de 8 bits.
    pub fn load_png(path: &Path) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut reader = png::Decoder::new(std::io::BufReader::new(file))
            .read_info()
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let size = reader
            .output_buffer_size()
            .ok_or_else(|| format!("{}: tamaño de búfer desconocido", path.display()))?;
        let mut buf = vec![0; size];
        let info = reader
            .next_frame(&mut buf)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        if info.color_type != png::ColorType::Grayscale || info.bit_depth != png::BitDepth::Eight {
            return Err(format!(
                "{}: se esperaba PNG en grises de 8 bits",
                path.display()
            ));
        }
        buf.truncate(info.buffer_size());
        Ok(Self {
            width: info.width,
            height: info.height,
            px: buf,
        })
    }

    /// Guarda como PNG en escala de grises de 8 bits.
    pub fn save_png(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let file = File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut enc = png::Encoder::new(BufWriter::new(file), self.width, self.height);
        enc.set_color(png::ColorType::Grayscale);
        enc.set_depth(png::BitDepth::Eight);
        enc.set_compression(png::Compression::High);
        let mut w = enc.write_header().map_err(|e| e.to_string())?;
        w.write_image_data(&self.px).map_err(|e| e.to_string())
    }

    /// Diferencia absoluta amplificada (×4), para inspección visual.
    pub fn diff(&self, other: &Gray) -> Gray {
        let px = self
            .px
            .iter()
            .zip(&other.px)
            .map(|(a, b)| 255 - (a.abs_diff(*b) as u32 * 4).min(255) as u8)
            .collect();
        Gray {
            width: self.width,
            height: self.height,
            px,
        }
    }
}

/// SSIM medio por bloques de 8×8 sin solape. 0,0 si los tamaños difieren.
pub fn ssim(a: &Gray, b: &Gray) -> f64 {
    if a.width != b.width || a.height != b.height || a.px.len() != b.px.len() {
        return 0.0;
    }
    const C1: f64 = (0.01 * 255.0) * (0.01 * 255.0);
    const C2: f64 = (0.03 * 255.0) * (0.03 * 255.0);
    let (w, h) = (a.width as usize, a.height as usize);
    let mut total = 0.0;
    let mut blocks = 0usize;
    for by in (0..h).step_by(8) {
        for bx in (0..w).step_by(8) {
            let (mut sa, mut sb, mut saa, mut sbb, mut sab, mut n) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
            for y in by..(by + 8).min(h) {
                for x in bx..(bx + 8).min(w) {
                    let (va, vb) = (a.px[y * w + x] as f64, b.px[y * w + x] as f64);
                    sa += va;
                    sb += vb;
                    saa += va * va;
                    sbb += vb * vb;
                    sab += va * vb;
                    n += 1.0;
                }
            }
            let (ma, mb) = (sa / n, sb / n);
            let (va, vb) = (saa / n - ma * ma, sbb / n - mb * mb);
            let cov = sab / n - ma * mb;
            total += ((2.0 * ma * mb + C1) * (2.0 * cov + C2))
                / ((ma * ma + mb * mb + C1) * (va + vb + C2));
            blocks += 1;
        }
    }
    if blocks == 0 {
        1.0
    } else {
        total / blocks as f64
    }
}
