//! T0.4: cada documento del corpus `smoke` se abre (o falla) como indica su `expect`.
//!
//! Requiere `cargo xtask pdfium fetch` y `cargo xtask corpus fetch`.
//! Cubre los casos borde de `requirements.md` §14 con PDFium (el motor se integra en T2.2).

mod common;

use std::collections::BTreeMap;
use std::path::PathBuf;

use demi_engine::pdfium_lib;
use demi_engine::pdfium_render::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Manifest {
    #[serde(default, rename = "file")]
    files: Vec<Entry>,
    #[serde(default, rename = "generated")]
    generated: Vec<Generated>,
}

#[derive(Deserialize)]
struct Entry {
    id: String,
    source: String,
    path: String,
    sets: Vec<String>,
    expect: String,
}

#[derive(Deserialize)]
struct Generated {
    id: String,
    file: String,
    sets: Vec<String>,
    expect: String,
}

/// Abre el documento y, si se puede, renderiza la primera página a 200 px de ancho.
fn outcome(pdfium: &Pdfium, path: &PathBuf) -> String {
    match pdfium.load_pdf_from_file(path, None) {
        Ok(doc) => {
            let pages = doc.pages();
            if pages.is_empty() {
                return "ok".to_string();
            }
            let rendered = pages.first().and_then(|p| {
                p.render_with_config(&PdfRenderConfig::new().set_target_width(200))
                    .map(|_| ())
            });
            match rendered {
                Ok(()) => "ok".to_string(),
                Err(e) => format!("error-render ({e:?})"),
            }
        }
        Err(PdfiumError::PdfiumLibraryInternalError(PdfiumInternalError::PasswordError)) => {
            "password".to_string()
        }
        Err(_) => "error".to_string(),
    }
}

#[test]
fn smoke_corpus_matches_expectations() {
    let root = common::workspace_root();
    let manifest: Manifest = toml::from_str(
        &std::fs::read_to_string(root.join("corpus/manifest.toml")).expect("manifest.toml"),
    )
    .expect("manifest.toml válido");

    let mut cases: Vec<(String, PathBuf, String)> = Vec::new();
    for f in manifest
        .files
        .iter()
        .filter(|f| f.sets.iter().any(|s| s == "smoke"))
    {
        cases.push((
            f.id.clone(),
            root.join("corpus").join(&f.source).join(&f.path),
            f.expect.clone(),
        ));
    }
    for g in manifest
        .generated
        .iter()
        .filter(|g| g.sets.iter().any(|s| s == "smoke"))
    {
        cases.push((
            g.id.clone(),
            root.join("corpus/generated").join(&g.file),
            g.expect.clone(),
        ));
    }
    let missing: Vec<_> = cases
        .iter()
        .filter(|(_, p, _)| !p.exists())
        .map(|(id, _, _)| id)
        .collect();
    assert!(
        missing.is_empty(),
        "Faltan archivos del corpus ({missing:?}). Ejecuta `cargo xtask corpus fetch`."
    );

    let dir = common::pdfium_dir().expect("plataforma no soportada por pdfium-binaries");
    let pdfium = pdfium_lib::bind(&dir).expect("PDFium no disponible: `cargo xtask pdfium fetch`");

    let mut mismatches = BTreeMap::new();
    for (id, path, expect) in &cases {
        let got = outcome(&pdfium, path);
        if &got != expect {
            mismatches.insert(id.clone(), format!("esperado {expect}, obtenido {got}"));
        }
    }
    assert!(
        mismatches.is_empty(),
        "Diferencias con el manifiesto: {mismatches:#?}"
    );
    println!("{} documentos del corpus smoke verificados", cases.len());
}
