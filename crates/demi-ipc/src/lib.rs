//! Protocolo entre la app de Demi y `demi-worker`: mensajes, framing y memoria compartida.
//!
//! Ver `specs/00-producto/design.md` §5. Todo lo que llega por este canal se trata
//! como entrada no confiable (R-NFR-SEG-01).
#![deny(unsafe_code)]
