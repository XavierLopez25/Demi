//! T0.5: regresión visual. La primera página de cada documento `ok` del corpus `smoke`
//! se renderiza y se compara por SSIM con su imagen de referencia.
//!
//! - Referencias: `tests/golden/refs/<id>.png` (versionadas).
//! - Umbrales: `tests/golden/golden.toml` (por defecto y por documento).
//! - Diferencias: `target/golden-diff/<id>-{actual,diff}.png` (artefacto de CI).
//! - Regenerar referencias: `cargo xtask golden update` (revisar el resultado a mano).
//!
//! Requiere `cargo xtask pdfium fetch` y `cargo xtask corpus fetch`.

mod common;

use std::collections::BTreeMap;
use std::path::PathBuf;

use common::golden::{Gray, ssim};
use demi_engine::pdfium_lib;
use demi_engine::pdfium_render::prelude::*;
use serde::Deserialize;

/// Ancho de render de las referencias, en píxeles.
const WIDTH: u16 = 300;
/// Variable que activa la regeneración de referencias.
const ENV_UPDATE: &str = "DEMI_GOLDEN_UPDATE";

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

#[derive(Deserialize)]
struct Config {
    default_threshold: f64,
    #[serde(default)]
    threshold: BTreeMap<String, f64>,
    #[serde(default)]
    skip: BTreeMap<String, String>,
}

fn render_first_page(pdfium: &Pdfium, path: &PathBuf) -> Result<Option<Gray>, String> {
    let doc = pdfium
        .load_pdf_from_file(path, None)
        .map_err(|e| format!("{e:?}"))?;
    let pages = doc.pages();
    if pages.is_empty() {
        return Ok(None);
    }
    let page = pages.first().map_err(|e| format!("{e:?}"))?;
    let bitmap = page
        .render_with_config(&PdfRenderConfig::new().set_target_width(WIDTH as i32))
        .map_err(|e| format!("{e:?}"))?;
    Ok(Some(Gray::from_bgra(
        &bitmap.as_raw_bytes(),
        bitmap.width() as u32,
        bitmap.height() as u32,
    )))
}

#[test]
fn ssim_of_identical_images_is_one() {
    let img = Gray {
        width: 16,
        height: 16,
        px: (0..=255).collect(),
    };
    assert!((ssim(&img, &img) - 1.0).abs() < 1e-9);
}

#[test]
fn ssim_detects_differences() {
    let a = Gray {
        width: 16,
        height: 16,
        px: (0..=255).collect(),
    };
    let b = Gray {
        width: 16,
        height: 16,
        px: (0..=255).map(|v| 255 - v).collect(),
    };
    assert!(ssim(&a, &b) < 0.5);
    let c = Gray {
        width: 8,
        height: 8,
        px: vec![0; 64],
    };
    assert_eq!(ssim(&a, &c), 0.0, "tamaños distintos");
}

#[test]
fn smoke_corpus_matches_golden_images() {
    let root = common::workspace_root();
    let golden_dir = root.join("crates/demi-engine/tests/golden");
    let diff_dir = root.join("target/golden-diff");
    let update = std::env::var_os(ENV_UPDATE).is_some();

    let manifest: Manifest = toml::from_str(
        &std::fs::read_to_string(root.join("corpus/manifest.toml")).expect("manifest.toml"),
    )
    .expect("manifest.toml válido");
    let config: Config = toml::from_str(
        &std::fs::read_to_string(golden_dir.join("golden.toml")).expect("golden.toml"),
    )
    .expect("golden.toml válido");

    let mut cases: Vec<(String, PathBuf)> = Vec::new();
    for f in &manifest.files {
        if f.expect == "ok" && f.sets.iter().any(|s| s == "smoke") {
            cases.push((
                f.id.clone(),
                root.join("corpus").join(&f.source).join(&f.path),
            ));
        }
    }
    for g in &manifest.generated {
        if g.expect == "ok" && g.sets.iter().any(|s| s == "smoke") {
            cases.push((g.id.clone(), root.join("corpus/generated").join(&g.file)));
        }
    }

    let dir = common::pdfium_dir().expect("plataforma no soportada por pdfium-binaries");
    let pdfium = pdfium_lib::bind(&dir).expect("PDFium no disponible: `cargo xtask pdfium fetch`");

    let mut failures = BTreeMap::new();
    let mut checked = 0;
    for (id, path) in &cases {
        if let Some(reason) = config.skip.get(id) {
            println!("omitido {id}: {reason}");
            continue;
        }
        assert!(
            path.exists(),
            "Falta {}. Ejecuta `cargo xtask corpus fetch`.",
            path.display()
        );
        let actual = match render_first_page(&pdfium, path) {
            Ok(Some(img)) => img,
            Ok(None) => continue,
            Err(e) => {
                failures.insert(id.clone(), format!("no se pudo renderizar: {e}"));
                continue;
            }
        };
        let ref_path = golden_dir.join("refs").join(format!("{id}.png"));
        if update {
            actual.save_png(&ref_path).expect("guardar referencia");
            continue;
        }
        let reference = match Gray::load_png(&ref_path) {
            Ok(r) => r,
            Err(_) => {
                failures.insert(id.clone(), "falta la imagen de referencia".to_string());
                let _ = actual.save_png(&diff_dir.join(format!("{id}-actual.png")));
                continue;
            }
        };
        let threshold = config
            .threshold
            .get(id)
            .copied()
            .unwrap_or(config.default_threshold);
        let score = ssim(&actual, &reference);
        checked += 1;
        if score < threshold {
            let _ = actual.save_png(&diff_dir.join(format!("{id}-actual.png")));
            if actual.width == reference.width && actual.height == reference.height {
                let _ = actual
                    .diff(&reference)
                    .save_png(&diff_dir.join(format!("{id}-diff.png")));
            }
            failures.insert(id.clone(), format!("SSIM {score:.4} < {threshold}"));
        }
    }

    if update {
        println!(
            "Referencias regeneradas en {}. Revísalas antes de hacer commit.",
            golden_dir.display()
        );
        return;
    }
    assert!(
        failures.is_empty(),
        "Regresión visual ({} de {} documentos). Imágenes en {}:\n{failures:#?}",
        failures.len(),
        cases.len(),
        diff_dir.display()
    );
    println!("{checked} documentos comparados con sus referencias");
}
