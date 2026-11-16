//! Motor de documentos de Demi: el trait `Engine` y su implementación sobre PDFium.
//!
//! Ver `specs/00-producto/design.md` §4. Este crate solo se usa dentro de
//! `demi-worker`, nunca en el proceso de la interfaz (ADR-05).
#![deny(unsafe_code)]

pub mod pdfium_lib;

pub use pdfium_render;
