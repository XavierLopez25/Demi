//! Tareas de desarrollo de Demi (`cargo xtask <tarea>`).
//!
//! - `pdfium fetch [--platform <p>]`: descarga PDFium fijado en `xtask/pdfium.lock`,
//!   verifica su SHA-256 y lo extrae en `vendor/pdfium/<plataforma>/` (T0.3).
//! - `pdfium dir [--platform <p>]`: imprime el directorio de la biblioteca.
//!
//! Previstas: `corpus fetch` (T0.4), `golden update` (T0.5).

mod pdfium;

use std::process::ExitCode;

const HELP: &str = "Uso: cargo xtask <tarea>

Tareas:
  pdfium fetch [--platform <p>]   Descarga y verifica PDFium fijado
  pdfium dir   [--platform <p>]   Imprime el directorio de la biblioteca
  help                            Muestra esta ayuda

Plataformas: linux-x64, linux-arm64, win-x64, win-arm64, mac-arm64, mac-x64
(por defecto, la del equipo actual).";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] | ["help"] | ["--help"] | ["-h"] => {
            println!("{HELP}");
            Ok(())
        }
        ["pdfium", "fetch", rest @ ..] => {
            pdfium::platform_arg(rest).and_then(|p| pdfium::fetch(p.as_deref()))
        }
        ["pdfium", "dir", rest @ ..] => pdfium::platform_arg(rest).and_then(|p| {
            let dir = pdfium::library_dir(p.as_deref())?;
            println!("{}", dir.display());
            Ok(())
        }),
        _ => Err(format!("Tarea desconocida: {}\n\n{HELP}", args.join(" "))),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
