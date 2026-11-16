//! Generador mínimo y determinista de PDFs sintéticos para el corpus (T0.4).
//!
//! Produce los casos borde de `requirements.md` §14 que no conviene descargar:
//! archivos vacíos, truncados, con basura delante, con xref roto, muchos
//! páginas u operadores, páginas gigantes y nombres de archivo no ASCII.

/// Documento PDF en construcción: lista de objetos (sin numerar) y su serialización.
struct Doc {
    objects: Vec<Vec<u8>>,
}

impl Doc {
    fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    /// Añade un objeto y devuelve su número.
    fn add(&mut self, body: impl Into<Vec<u8>>) -> usize {
        self.objects.push(body.into());
        self.objects.len()
    }

    /// Reemplaza el cuerpo de un objeto ya reservado.
    fn set(&mut self, num: usize, body: impl Into<Vec<u8>>) {
        self.objects[num - 1] = body.into();
    }

    fn stream(content: &[u8]) -> Vec<u8> {
        let mut v = format!("<< /Length {} >>\nstream\n", content.len()).into_bytes();
        v.extend_from_slice(content);
        v.extend_from_slice(b"\nendstream");
        v
    }

    /// Serializa con tabla xref correcta. `root` es el número del catálogo.
    fn finish(&self, root: usize) -> Vec<u8> {
        let mut out = b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n".to_vec();
        let mut offsets = Vec::with_capacity(self.objects.len());
        for (i, body) in self.objects.iter().enumerate() {
            offsets.push(out.len());
            out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
            out.extend_from_slice(body);
            out.extend_from_slice(b"\nendobj\n");
        }
        let xref = out.len();
        let n = self.objects.len() + 1;
        out.extend_from_slice(format!("xref\n0 {n}\n0000000000 65535 f \n").as_bytes());
        for off in offsets {
            out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
        }
        out.extend_from_slice(
            format!("trailer\n<< /Size {n} /Root {root} 0 R >>\nstartxref\n{xref}\n%%EOF\n")
                .as_bytes(),
        );
        out
    }
}

/// Documento de `pages` páginas de `w`×`h` pt; `content(i)` da el content stream de la página i.
fn document(pages: usize, w: f32, h: f32, content: impl Fn(usize) -> Vec<u8>) -> Vec<u8> {
    let mut d = Doc::new();
    let catalog = d.add(Vec::new());
    let pages_obj = d.add(Vec::new());
    let font = d.add(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec());
    let mut kids = Vec::with_capacity(pages);
    for i in 0..pages {
        let c = d.add(Doc::stream(&content(i)));
        let p = d.add(
            format!(
                "<< /Type /Page /Parent {pages_obj} 0 R /MediaBox [0 0 {w} {h}] \
                 /Resources << /Font << /F1 {font} 0 R >> >> /Contents {c} 0 R >>"
            )
            .into_bytes(),
        );
        kids.push(format!("{p} 0 R"));
    }
    d.set(
        catalog,
        format!("<< /Type /Catalog /Pages {pages_obj} 0 R >>").into_bytes(),
    );
    d.set(
        pages_obj,
        format!(
            "<< /Type /Pages /Kids [{}] /Count {pages} >>",
            kids.join(" ")
        )
        .into_bytes(),
    );
    d.finish(catalog)
}

fn text_page(i: usize) -> Vec<u8> {
    format!("BT /F1 24 Tf 72 720 Td (Demi - pagina {}) Tj ET", i + 1).into_bytes()
}

/// PDF simple de una página A4 con texto.
fn simple() -> Vec<u8> {
    document(1, 595.0, 842.0, text_page)
}

/// Genera el contenido de un caso sintético por su `kind`.
pub fn generate(kind: &str) -> Result<Vec<u8>, String> {
    Ok(match kind {
        "zero-bytes" => Vec::new(),
        "simple" => simple(),
        "garbage-prefix" => {
            // 1 KiB de basura determinista antes de `%PDF` (los offsets del xref quedan desplazados).
            let mut v: Vec<u8> = (0..1024u32).map(|i| (i * 37 % 251) as u8).collect();
            v.extend_from_slice(&simple());
            v
        }
        "truncated" => {
            let mut v = document(5, 595.0, 842.0, text_page);
            v.truncate(v.len() / 2);
            v
        }
        "broken-xref" => {
            // Todos los offsets del xref apuntan a 0: el lector debe reconstruirlo.
            let v = simple();
            let text = String::from_utf8_lossy(&v).into_owned();
            let mut out = String::with_capacity(text.len());
            for line in text.split_inclusive('\n') {
                if line.len() == 20 && line.ends_with(" 00000 n \n") {
                    out.push_str("0000000000 00000 n \n");
                } else {
                    out.push_str(line);
                }
            }
            out.into_bytes()
        }
        "pages-1000" => document(1000, 595.0, 842.0, text_page),
        "page-a0" => document(1, 2384.0, 3370.0, |_| {
            b"0.17 0.37 0.38 rg 100 100 2184 3170 re f BT /F1 200 Tf 300 1600 Td (A0) Tj ET"
                .to_vec()
        }),
        "ops-100k" => document(1, 595.0, 842.0, |_| {
            let mut c = Vec::with_capacity(100_000 * 24);
            for i in 0..100_000u32 {
                let x = (i % 500) as f32 + 40.0;
                let y = (i / 500) as f32 * 3.5 + 40.0;
                c.extend_from_slice(format!("{x} {y} 1 1 re f\n").as_bytes());
            }
            c
        }),
        other => return Err(format!("tipo de PDF sintético desconocido: {other}")),
    })
}

#[cfg(test)]
mod tests {
    use super::generate;

    #[test]
    fn generation_is_deterministic() {
        for kind in [
            "simple",
            "garbage-prefix",
            "truncated",
            "broken-xref",
            "page-a0",
        ] {
            assert_eq!(generate(kind).unwrap(), generate(kind).unwrap(), "{kind}");
        }
    }

    #[test]
    fn simple_has_valid_structure() {
        let pdf = generate("simple").unwrap();
        assert!(pdf.starts_with(b"%PDF-1.7"));
        assert!(pdf.ends_with(b"%%EOF\n"));
    }

    #[test]
    fn unknown_kind_is_an_error() {
        assert!(generate("nope").is_err());
    }
}
