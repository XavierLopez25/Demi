//! Descarga verificada de PDFium (T0.3).

use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Deserialize)]
struct Lock {
    version: String,
    url_template: String,
    sha256: BTreeMap<String, String>,
}

/// Raíz del workspace (padre de `xtask/`).
fn workspace_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.parent().map(Path::to_path_buf).unwrap_or(manifest)
}

fn read_lock() -> Result<Lock, String> {
    let path = workspace_root().join("xtask").join("pdfium.lock");
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("no se pudo leer {}: {e}", path.display()))?;
    toml::from_str(&text).map_err(|e| format!("pdfium.lock inválido: {e}"))
}

/// Plataforma de PDFium (nomenclatura de pdfium-binaries) del equipo actual.
pub fn host_platform() -> Result<&'static str, String> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    match (os, arch) {
        ("linux", "x86_64") => Ok("linux-x64"),
        ("linux", "aarch64") => Ok("linux-arm64"),
        ("windows", "x86_64") => Ok("win-x64"),
        ("windows", "aarch64") => Ok("win-arm64"),
        ("macos", "aarch64") => Ok("mac-arm64"),
        ("macos", "x86_64") => Ok("mac-x64"),
        _ => Err(format!("plataforma no soportada: {os}/{arch}")),
    }
}

/// Lee `--platform <p>` de los argumentos restantes.
pub fn platform_arg(rest: &[&str]) -> Result<Option<String>, String> {
    match rest {
        [] => Ok(None),
        ["--platform", p] => Ok(Some((*p).to_string())),
        other => Err(format!("argumentos no reconocidos: {}", other.join(" "))),
    }
}

/// Directorio que contiene la biblioteca dinámica de PDFium para la plataforma.
pub fn library_dir(platform: Option<&str>) -> Result<PathBuf, String> {
    let platform = match platform {
        Some(p) => p,
        None => host_platform()?,
    };
    let sub = if platform.starts_with("win") {
        "bin"
    } else {
        "lib"
    };
    Ok(workspace_root()
        .join("vendor")
        .join("pdfium")
        .join(platform)
        .join(sub))
}

/// Descarga, verifica y extrae PDFium.
pub fn fetch(platform: Option<&str>) -> Result<(), String> {
    let lock = read_lock()?;
    let platform = match platform {
        Some(p) => p.to_string(),
        None => host_platform()?.to_string(),
    };
    let expected = lock
        .sha256
        .get(&platform)
        .ok_or_else(|| format!("pdfium.lock no tiene checksum para {platform}"))?;
    let url = lock
        .url_template
        .replace("{version}", &lock.version)
        .replace("{platform}", &platform);

    let dest = workspace_root()
        .join("vendor")
        .join("pdfium")
        .join(&platform);
    let marker = dest.join(".demi-version");
    if fs::read_to_string(&marker).ok().as_deref() == Some(lock.version.as_str()) {
        println!(
            "PDFium {} ({platform}) ya está en {}",
            lock.version,
            dest.display()
        );
        return Ok(());
    }

    let archive = workspace_root()
        .join("target")
        .join(format!("pdfium-{}-{platform}.tgz", lock.version));
    if let Some(parent) = archive.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    println!("Descargando {url}");
    // curl viene incluido en Windows 10+, macOS y las distribuciones Linux habituales,
    // y respeta la configuración de proxy del sistema.
    let status = Command::new("curl")
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--location",
            "--output",
        ])
        .arg(&archive)
        .arg(&url)
        .status()
        .map_err(|e| format!("no se pudo ejecutar curl: {e}"))?;
    if !status.success() {
        return Err(format!("la descarga falló ({status})"));
    }

    let bytes = fs::read(&archive).map_err(|e| e.to_string())?;
    let actual = hex(&Sha256::digest(&bytes));
    if !actual.eq_ignore_ascii_case(expected) {
        let _ = fs::remove_file(&archive);
        return Err(format!(
            "checksum incorrecto para {platform}: esperado {expected}, obtenido {actual}"
        ));
    }
    println!("SHA-256 verificado: {actual}");

    if dest.exists() {
        fs::remove_dir_all(&dest).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
    extract_tgz(&bytes, &dest)?;
    fs::write(&marker, &lock.version).map_err(|e| e.to_string())?;

    println!(
        "PDFium {} ({platform}) listo en {}",
        lock.version,
        dest.display()
    );
    println!("Biblioteca: {}", library_dir(Some(&platform))?.display());
    Ok(())
}

fn extract_tgz(bytes: &[u8], dest: &Path) -> Result<(), String> {
    let decoder = flate2::read::GzDecoder::new(bytes);
    let mut archive = tar::Archive::new(decoder);
    for entry in archive.entries().map_err(|e| e.to_string())? {
        let mut entry = entry.map_err(|e| e.to_string())?;
        // `unpack_in` rechaza rutas que intenten salir de `dest` (p. ej. `../`).
        let ok = entry.unpack_in(dest).map_err(|e| e.to_string())?;
        if !ok {
            let mut sink = Vec::new();
            let _ = entry.read_to_end(&mut sink);
            return Err("el archivo contiene rutas fuera del destino".to_string());
        }
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
