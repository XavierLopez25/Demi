//! Corpus de PDFs de prueba (T0.4).
//!
//! `corpus/manifest.toml` describe cada archivo: origen, hash, conjuntos, etiquetas
//! y resultado esperado al abrirlo. Los PDFs no se versionan: se descargan
//! verificados por SHA-256 o se generan de forma determinista.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::pdfgen;

/// Manifiesto del corpus.
#[derive(Deserialize)]
pub struct Manifest {
    /// Orígenes de descarga, por nombre.
    pub sources: BTreeMap<String, Source>,
    /// Archivos descargados.
    #[serde(default, rename = "file")]
    pub files: Vec<FileEntry>,
    /// Archivos sintéticos generados por `pdfgen`.
    #[serde(default, rename = "generated")]
    pub generated: Vec<GeneratedEntry>,
}

/// Origen de descarga.
#[derive(Deserialize)]
pub struct Source {
    url_template: String,
    commit: String,
    license: String,
}

/// Archivo descargado.
#[derive(Deserialize)]
pub struct FileEntry {
    id: String,
    source: String,
    path: String,
    sha256: String,
    sets: Vec<String>,
    tags: Vec<String>,
    expect: String,
}

/// Archivo sintético.
#[derive(Deserialize)]
pub struct GeneratedEntry {
    id: String,
    kind: String,
    file: String,
    sets: Vec<String>,
    tags: Vec<String>,
    expect: String,
}

const EXPECTS: [&str; 3] = ["ok", "password", "error"];

fn corpus_dir() -> PathBuf {
    crate::pdfium::workspace_root().join("corpus")
}

/// Lee y valida `corpus/manifest.toml`.
pub fn read_manifest() -> Result<Manifest, String> {
    let path = corpus_dir().join("manifest.toml");
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("no se pudo leer {}: {e}", path.display()))?;
    let m: Manifest = toml::from_str(&text).map_err(|e| format!("manifest.toml inválido: {e}"))?;

    let mut ids = std::collections::BTreeSet::new();
    for (id, expect, sets) in m
        .files
        .iter()
        .map(|f| (&f.id, &f.expect, &f.sets))
        .chain(m.generated.iter().map(|g| (&g.id, &g.expect, &g.sets)))
    {
        if !ids.insert(id.clone()) {
            return Err(format!("id duplicado en el manifiesto: {id}"));
        }
        if !EXPECTS.contains(&expect.as_str()) {
            return Err(format!("{id}: expect debe ser uno de {EXPECTS:?}"));
        }
        if sets.is_empty() {
            return Err(format!("{id}: debe pertenecer al menos a un conjunto"));
        }
    }
    for f in &m.files {
        if !m.sources.contains_key(&f.source) {
            return Err(format!("{}: origen desconocido {}", f.id, f.source));
        }
        if f.path.contains("..") || f.path.starts_with('/') {
            return Err(format!("{}: ruta no permitida {}", f.id, f.path));
        }
    }
    for g in &m.generated {
        if g.file.contains(['/', '\\']) || g.file.contains("..") {
            return Err(format!(
                "{}: nombre de archivo no permitido {}",
                g.id, g.file
            ));
        }
    }
    Ok(m)
}

/// Lee `--set <nombre>` (por defecto `smoke`).
pub fn set_arg(rest: &[&str]) -> Result<String, String> {
    match rest {
        [] => Ok("smoke".to_string()),
        ["--set", s] => Ok((*s).to_string()),
        other => Err(format!("argumentos no reconocidos: {}", other.join(" "))),
    }
}

/// Descarga (verificando SHA-256) y genera los archivos del conjunto indicado.
pub fn fetch(set: &str) -> Result<(), String> {
    let m = read_manifest()?;
    let dir = corpus_dir();
    let (mut downloaded, mut cached, mut generated) = (0, 0, 0);

    for f in m.files.iter().filter(|f| f.sets.iter().any(|s| s == set)) {
        let src = &m.sources[&f.source];
        let dest = dir.join(&f.source).join(&f.path);
        if dest.exists() && sha256_file(&dest)?.eq_ignore_ascii_case(&f.sha256) {
            cached += 1;
            continue;
        }
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let url = src
            .url_template
            .replace("{commit}", &src.commit)
            .replace("{path}", &f.path);
        let status = Command::new("curl")
            .args([
                "--fail",
                "--silent",
                "--show-error",
                "--location",
                "--output",
            ])
            .arg(&dest)
            .arg(&url)
            .status()
            .map_err(|e| format!("no se pudo ejecutar curl: {e}"))?;
        if !status.success() {
            return Err(format!("{}: la descarga falló ({status})", f.id));
        }
        let actual = sha256_file(&dest)?;
        if !actual.eq_ignore_ascii_case(&f.sha256) {
            let _ = fs::remove_file(&dest);
            return Err(format!(
                "{}: checksum incorrecto (esperado {}, obtenido {actual})",
                f.id, f.sha256
            ));
        }
        downloaded += 1;
    }

    let gen_dir = dir.join("generated");
    fs::create_dir_all(&gen_dir).map_err(|e| e.to_string())?;
    for g in m
        .generated
        .iter()
        .filter(|g| g.sets.iter().any(|s| s == set))
    {
        let bytes = pdfgen::generate(&g.kind).map_err(|e| format!("{}: {e}", g.id))?;
        fs::write(gen_dir.join(&g.file), bytes).map_err(|e| e.to_string())?;
        generated += 1;
    }

    println!(
        "Corpus '{set}': {downloaded} descargados, {cached} ya presentes, {generated} generados en {}",
        dir.display()
    );
    Ok(())
}

/// Imprime el contenido del manifiesto.
pub fn list() -> Result<(), String> {
    let m = read_manifest()?;
    for (name, s) in &m.sources {
        println!("origen {name}: commit {} · {}", s.commit, s.license);
    }
    for f in &m.files {
        println!(
            "{:<34} {:<8} [{}] {}",
            f.id,
            f.expect,
            f.sets.join(","),
            f.tags.join(", ")
        );
    }
    for g in &m.generated {
        println!(
            "{:<34} {:<8} [{}] {} (sintético)",
            g.id,
            g.expect,
            g.sets.join(","),
            g.tags.join(", ")
        );
    }
    println!(
        "Total: {} descargables + {} sintéticos",
        m.files.len(),
        m.generated.len()
    );
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}
