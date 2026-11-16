//! Tareas de desarrollo de Demi (`cargo xtask <tarea>`).
//!
//! Tareas previstas: `pdfium fetch` (T0.3), `corpus fetch` (T0.4), `golden update` (T0.5).

use std::process::ExitCode;

fn main() -> ExitCode {
    let task = std::env::args().nth(1);
    match task.as_deref() {
        None | Some("help") => {
            println!("Uso: cargo xtask <tarea>\n\nTareas: (pendientes de T0.3–T0.5)");
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("Tarea desconocida: {other}");
            ExitCode::FAILURE
        }
    }
}
