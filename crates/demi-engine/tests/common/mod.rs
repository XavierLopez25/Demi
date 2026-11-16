//! Utilidades compartidas por los tests de integración de `demi-engine`.
#![allow(dead_code)]

pub mod golden;

use std::path::PathBuf;

use demi_engine::pdfium_lib;

/// Raíz del workspace.
pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Directorio de PDFium: `DEMI_PDFIUM_DIR` o `vendor/pdfium/<plataforma>/{lib,bin}`.
/// `None` si la plataforma no está soportada por pdfium-binaries.
pub fn pdfium_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os(pdfium_lib::ENV_PDFIUM_DIR) {
        return Some(PathBuf::from(dir));
    }
    let platform = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "linux-x64",
        ("linux", "aarch64") => "linux-arm64",
        ("windows", "x86_64") => "win-x64",
        ("windows", "aarch64") => "win-arm64",
        ("macos", "aarch64") => "mac-arm64",
        ("macos", "x86_64") => "mac-x64",
        _ => return None,
    };
    let sub = if platform.starts_with("win") {
        "bin"
    } else {
        "lib"
    };
    Some(
        workspace_root()
            .join("vendor/pdfium")
            .join(platform)
            .join(sub),
    )
}
