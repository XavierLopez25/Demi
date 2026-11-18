//! Spike S3 de Demi: ¿quién es la fuente de verdad del documento? (design.md §3.4, §14)
//!
//! A) `lopdf` es la fuente de verdad: cada acción modifica el modelo lopdf, se serializa
//!    y PDFium recarga el documento para mostrarlo (o se guarda de forma incremental).
//! B) PDFium es la fuente de verdad: la acción se aplica con la API de PDFium y solo se
//!    serializa al guardar.
//!
//! Criterio: < 200 ms desde la acción hasta la página visible actualizada con 1 000 páginas.
//! Uso: `s3-bench` (PDFium en `DEMI_PDFIUM_DIR` o `vendor/pdfium/<plataforma>/…`).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

use std::io::Write as _;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use demi_engine::pdfium_lib;
use demi_engine::pdfium_render::prelude::*;
use lopdf::{Dictionary, Document, IncrementalDocument, Object, ObjectId};

const REPS: usize = 5;

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

/// PDF de N páginas A4 con texto (sin flujos comprimidos ni object streams).
fn sample_pdf(pages: usize) -> Vec<u8> {
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

#[derive(Clone, Copy, PartialEq)]
enum Action {
    Rotate,
    Annotate,
    Delete,
}

impl Action {
    fn name(self) -> &'static str {
        match self {
            Action::Rotate => "girar página",
            Action::Annotate => "añadir anotación",
            Action::Delete => "eliminar página",
        }
    }
}

fn page_dict(doc: &mut Document, id: ObjectId) -> &mut Dictionary {
    doc.get_object_mut(id).unwrap().as_dict_mut().unwrap()
}

fn square_annot() -> Dictionary {
    let mut a = Dictionary::new();
    a.set("Type", Object::Name(b"Annot".to_vec()));
    a.set("Subtype", Object::Name(b"Square".to_vec()));
    a.set("Rect", vec![100.into(), 600.into(), 300.into(), 680.into()]);
    a.set(
        "C",
        vec![Object::Real(0.17), Object::Real(0.37), Object::Real(0.38)],
    );
    a.set("Contents", Object::string_literal("Demi S3"));
    a
}

/// Aplica la acción sobre el modelo lopdf (página `k`, base 0).
fn lopdf_apply(doc: &mut Document, action: Action, k: u32) {
    let pages = doc.get_pages();
    let id = pages[&(k + 1)];
    match action {
        Action::Rotate => {
            page_dict(doc, id).set("Rotate", 90);
        }
        Action::Annotate => {
            let annot = doc.add_object(square_annot());
            let page = page_dict(doc, id);
            match page.get_mut(b"Annots") {
                Ok(Object::Array(arr)) => arr.push(Object::Reference(annot)),
                _ => page.set("Annots", vec![Object::Reference(annot)]),
            }
        }
        Action::Delete => doc.delete_pages(&[k + 1]),
    }
}

fn render(pdfium: &Pdfium, bytes: &[u8], k: i32) -> f64 {
    let t = Instant::now();
    let doc = pdfium.load_pdf_from_byte_slice(bytes, None).unwrap();
    let page = doc.pages().get(k).unwrap();
    page.render_with_config(&PdfRenderConfig::new().set_target_width(1024))
        .unwrap();
    ms(t.elapsed())
}

/// Comprueba que PDFium ve el cambio hecho con lopdf.
fn verify(pdfium: &Pdfium, bytes: &[u8], action: Action, k: i32, n_pages: i32) {
    let doc = pdfium.load_pdf_from_byte_slice(bytes, None).unwrap();
    match action {
        Action::Rotate => {
            let rot = doc.pages().get(k).unwrap().rotation().unwrap();
            assert_eq!(
                rot,
                PdfPageRenderRotation::Degrees90,
                "giro no visible en PDFium"
            );
        }
        Action::Annotate => {
            let page = doc.pages().get(k).unwrap();
            let found = page
                .annotations()
                .iter()
                .any(|a| a.annotation_type() == PdfPageAnnotationType::Square);
            assert!(found, "anotación no visible en PDFium");
        }
        Action::Delete => assert_eq!(doc.pages().len(), n_pages - 1, "página no eliminada"),
    }
}

struct Row {
    doc: String,
    action: &'static str,
    a_apply: f64,
    a_save: f64,
    a_reload_render: f64,
    a_total: f64,
    a_inc: Option<(f64, f64, usize)>, // (guardado incremental, recarga+render, bytes añadidos)
    b_apply_render: f64,
    b_save: f64,
}

fn main() {
    let lib = pdfium_lib::default_library_dir()
        .filter(|d| pdfium_lib::library_path(d).exists())
        .unwrap_or_else(vendor_pdfium);
    let pdfium = pdfium_lib::bind(&lib).expect("PDFium");

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut docs: Vec<(String, Vec<u8>)> = vec![
        ("texto 10 págs".into(), sample_pdf(10)),
        ("texto 100 págs".into(), sample_pdf(100)),
        ("texto 1 000 págs".into(), sample_pdf(1000)),
    ];
    for (name, rel) in [
        ("images.pdf (imágenes)", "corpus/pdfjs/images.pdf"),
        (
            "perf_images_cmyk.pdf (CMYK)",
            "corpus/pdfjs/perf_images_cmyk.pdf",
        ),
        ("mixedfonts.pdf (fuentes)", "corpus/pdfjs/mixedfonts.pdf"),
    ] {
        if let Ok(b) = std::fs::read(root.join(rel)) {
            docs.push((name.into(), b));
        }
    }

    // Calentamiento de PDFium (fuentes del sistema).
    let _ = render(&pdfium, &docs[0].1, 0);

    let mut rows = Vec::new();
    let mut loads = Vec::new();
    for (name, bytes) in &docs {
        let n_pages = pdfium
            .load_pdf_from_byte_slice(bytes, None)
            .unwrap()
            .pages()
            .len();
        let k = (n_pages / 2) as u32;
        let a_load = median(
            (0..REPS)
                .map(|_| {
                    let t = Instant::now();
                    let _ = Document::load_mem(bytes).unwrap();
                    ms(t.elapsed())
                })
                .collect(),
        );
        let b_load = median(
            (0..REPS)
                .map(|_| {
                    let t = Instant::now();
                    let _ = pdfium.load_pdf_from_byte_slice(bytes, None).unwrap();
                    ms(t.elapsed())
                })
                .collect(),
        );
        loads.push((name.clone(), bytes.len(), n_pages, a_load, b_load));

        for action in [Action::Rotate, Action::Annotate, Action::Delete] {
            if action == Action::Delete && n_pages < 2 {
                continue; // no se puede mostrar una página tras borrar la única
            }
            let (mut apply, mut save, mut reload, mut total) = (vec![], vec![], vec![], vec![]);
            let (mut inc_save, mut inc_reload, mut inc_bytes) = (vec![], vec![], 0usize);
            let (mut b_ar, mut b_sv) = (vec![], vec![]);
            let shown = if action == Action::Delete {
                k.min(n_pages as u32 - 2)
            } else {
                k
            };
            for _ in 0..REPS {
                // A) lopdf como fuente de verdad, reescritura completa.
                let mut doc = Document::load_mem(bytes).unwrap();
                let t0 = Instant::now();
                lopdf_apply(&mut doc, action, k);
                let t1 = Instant::now();
                let mut out = Vec::with_capacity(bytes.len() + 4096);
                doc.save_to(&mut out).unwrap();
                let t2 = Instant::now();
                let r = render(&pdfium, &out, shown as i32);
                verify(&pdfium, &out, action, k as i32, n_pages);
                apply.push(ms(t1 - t0));
                save.push(ms(t2 - t1));
                reload.push(r);
                total.push(ms(t2 - t0) + r);

                // A') lopdf con guardado incremental (anotación y giro).
                if action != Action::Delete {
                    let prev = Document::load_mem(bytes).unwrap();
                    let page_id = prev.get_pages()[&(k + 1)];
                    let t = Instant::now();
                    let mut inc = IncrementalDocument::create_from(bytes.to_vec(), prev);
                    inc.opt_clone_object_to_new_document(page_id).unwrap();
                    let nd = &mut inc.new_document;
                    match action {
                        Action::Rotate => {
                            page_dict(nd, page_id).set("Rotate", 90);
                        }
                        _ => {
                            let annot = nd.add_object(square_annot());
                            let page = page_dict(nd, page_id);
                            match page.get_mut(b"Annots") {
                                Ok(Object::Array(arr)) => arr.push(Object::Reference(annot)),
                                _ => page.set("Annots", vec![Object::Reference(annot)]),
                            }
                        }
                    }
                    let mut out = Vec::new();
                    inc.save_to(&mut out).unwrap();
                    inc_save.push(ms(t.elapsed()));
                    inc_bytes = out.len().saturating_sub(bytes.len());
                    inc_reload.push(render(&pdfium, &out, k as i32));
                    verify(&pdfium, &out, action, k as i32, n_pages);
                }

                // B) PDFium como fuente de verdad.
                let doc = pdfium.load_pdf_from_byte_slice(bytes, None).unwrap();
                let t = Instant::now();
                match action {
                    Action::Rotate => {
                        let mut p = doc.pages().get(k as i32).unwrap();
                        p.set_rotation(PdfPageRenderRotation::Degrees90);
                    }
                    Action::Annotate => {
                        let mut p = doc.pages().get(k as i32).unwrap();
                        let mut a = p.annotations_mut().create_square_annotation().unwrap();
                        a.set_bounds(PdfRect::new_from_values(600.0, 100.0, 680.0, 300.0))
                            .unwrap();
                        a.set_stroke_color(PdfColor::new(43, 94, 98, 255)).unwrap();
                    }
                    Action::Delete => {
                        doc.pages().get(k as i32).unwrap().delete().unwrap();
                    }
                }
                let p = doc.pages().get(shown as i32).unwrap();
                p.render_with_config(&PdfRenderConfig::new().set_target_width(1024))
                    .unwrap();
                b_ar.push(ms(t.elapsed()));
                let t = Instant::now();
                let _ = doc.save_to_bytes().unwrap();
                b_sv.push(ms(t.elapsed()));
            }
            rows.push(Row {
                doc: name.clone(),
                action: action.name(),
                a_apply: median(apply),
                a_save: median(save),
                a_reload_render: median(reload),
                a_total: median(total),
                a_inc: (!inc_save.is_empty())
                    .then(|| (median(inc_save), median(inc_reload), inc_bytes)),
                b_apply_render: median(b_ar),
                b_save: median(b_sv),
            });
        }
    }

    let os = format!("{} {}", std::env::consts::OS, std::env::consts::ARCH);
    let mut r = format!(
        "## Spike S3 · fuente de verdad ({os})\n\nMedianas de {REPS} repeticiones; render de la página afectada a 1024 px.\n\n"
    );
    r.push_str(
        "| Documento | Tamaño | Páginas | Carga lopdf | Carga PDFium |\n|---|---|---|---|---|\n",
    );
    for (name, size, n, a, b) in &loads {
        r.push_str(&format!(
            "| {name} | {:.1} MB | {n} | {a:.1} ms | {b:.1} ms |\n",
            *size as f64 / 1e6
        ));
    }
    r.push_str("\n| Documento | Acción | A: aplicar | A: guardar todo | A: recargar + render | **A: total** | A': incremental (guardar · recargar+render · bytes) | **B: PDFium aplicar + render** | B: guardar |\n|---|---|---|---|---|---|---|---|---|\n");
    for x in &rows {
        let inc = x
            .a_inc
            .map(|(s, rr, b)| format!("{s:.1} · {rr:.1} ms · {b} B"))
            .unwrap_or_else(|| "—".into());
        r.push_str(&format!(
            "| {} | {} | {:.2} ms | {:.1} ms | {:.1} ms | **{:.1} ms** | {} | **{:.1} ms** | {:.1} ms |\n",
            x.doc, x.action, x.a_apply, x.a_save, x.a_reload_render, x.a_total, inc, x.b_apply_render, x.b_save
        ));
    }
    println!("{r}");
    if let Some(path) = std::env::var_os("GITHUB_STEP_SUMMARY") {
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .unwrap();
        let _ = writeln!(f, "{r}");
    }
}

fn vendor_pdfium() -> PathBuf {
    let platform = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "linux-x64",
        ("linux", "aarch64") => "linux-arm64",
        ("windows", "x86_64") => "win-x64",
        ("windows", "aarch64") => "win-arm64",
        ("macos", "aarch64") => "mac-arm64",
        _ => "mac-x64",
    };
    let sub = if platform.starts_with("win") {
        "bin"
    } else {
        "lib"
    };
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/pdfium")
        .join(platform)
        .join(sub)
}
