//! `demi-worker`: proceso aislado que abre y procesa los PDFs (ADR-05).
//!
//! Por ahora solo un esqueleto; el bucle IPC llega en la tarea T2.4.

fn main() {
    println!("demi-worker {}", demi_core::VERSION);
}
