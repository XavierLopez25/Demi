//! Núcleo de Demi: modelo de documento, catálogo de acciones y deshacer.
//!
//! Ver `specs/00-producto/design.md` §3. Este crate no depende de ninguna
//! interfaz gráfica ni de PDFium (R-NFR-CAL-01).
#![forbid(unsafe_code)]

/// Versión de Demi (la misma para todos los crates del workspace).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::VERSION;

    #[test]
    fn version_is_semver() {
        assert_eq!(VERSION.split('.').count(), 3);
    }
}
