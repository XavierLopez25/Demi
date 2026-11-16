//! Localización y carga de la biblioteca dinámica de PDFium (T0.3).
//!
//! Orden de búsqueda de [`default_library_dir`]:
//! 1. Variable de entorno `DEMI_PDFIUM_DIR` (desarrollo, CI).
//! 2. Directorio del ejecutable actual (instalaciones empaquetadas).
//!
//! En desarrollo, `cargo xtask pdfium fetch` descarga la versión fijada y
//! `cargo xtask pdfium dir` imprime el directorio a usar en `DEMI_PDFIUM_DIR`.

use std::path::{Path, PathBuf};

use pdfium_render::prelude::{Pdfium, PdfiumError};

/// Variable de entorno que indica el directorio de la biblioteca de PDFium.
pub const ENV_PDFIUM_DIR: &str = "DEMI_PDFIUM_DIR";

/// Directorio donde buscar la biblioteca de PDFium, si se puede determinar.
pub fn default_library_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os(ENV_PDFIUM_DIR) {
        return Some(PathBuf::from(dir));
    }
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
}

/// Ruta completa de la biblioteca de PDFium dentro de `dir` (`libpdfium.so`,
/// `libpdfium.dylib` o `pdfium.dll` según la plataforma).
pub fn library_path(dir: &Path) -> PathBuf {
    Pdfium::pdfium_platform_library_name_at_path(dir)
}

/// Carga PDFium desde `dir` y devuelve una instancia lista para usar.
///
/// Solo puede haber una biblioteca cargada por proceso: cada `demi-worker`
/// la carga una vez al arrancar.
pub fn bind(dir: &Path) -> Result<Pdfium, PdfiumError> {
    let bindings = Pdfium::bind_to_library(library_path(dir))?;
    Ok(Pdfium::new(bindings))
}
