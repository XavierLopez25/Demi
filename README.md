# Demi

**Visor y editor de PDF libre, sin paywalls, sin cuentas y sin IA impuesta.**
Windows · macOS · Linux · escrito en Rust.

> Estado: en desarrollo temprano (fase F0 · Fundaciones). Todavía no hay versión usable.

## Qué será Demi

- Leer, buscar y navegar PDFs grandes con fluidez.
- Anotar: resaltar, subrayar, notas, texto, dibujo.
- Organizar páginas: unir, dividir, girar, reordenar, extraer.
- Llenar formularios y **firmar en 4 pasos**, incluso creando la firma desde una foto.
- Corregir texto existente en el PDF.
- Proteger: redacción real, contraseñas, limpieza de metadatos (v1.0).
- Todo en tu computadora: sin telemetría y sin conexiones que no pidas.

## Especificación

Demi se desarrolla con *Spec-Driven Development*. Todo el diseño está en [`specs/`](specs/):

- [`specs/00-producto/requirements.md`](specs/00-producto/requirements.md): requisitos (EARS)
- [`specs/00-producto/design.md`](specs/00-producto/design.md): arquitectura y sistema de diseño
- [`specs/00-producto/tasks.md`](specs/00-producto/tasks.md): plan de implementación

## Compilar

Requisitos: Rust estable (≥ 1.85).

```sh
cargo xtask pdfium fetch   # descarga PDFium fijado y verifica su SHA-256
cargo build
cargo test
cargo xtask help
```

PDFium se carga dinámicamente desde `vendor/pdfium/<plataforma>/` en desarrollo
(o desde `DEMI_PDFIUM_DIR`); en las versiones empaquetadas, desde el directorio del ejecutable.

## Estructura

| Crate | Licencia | Propósito |
|-------|----------|-----------|
| `demi-core` | MIT OR Apache-2.0 | Modelo, acciones, deshacer |
| `demi-engine` | MIT OR Apache-2.0 | Motor PDFium |
| `demi-text` | MIT OR Apache-2.0 | Edición de texto existente |
| `demi-sign` | MIT OR Apache-2.0 | Firmas visuales y PAdES |
| `demi-ipc` | MIT OR Apache-2.0 | Protocolo app ↔ worker |
| `demi-store` | MIT OR Apache-2.0 | Datos locales y firmas cifradas |
| `demi-worker` | MIT OR Apache-2.0 | Proceso aislado que abre los PDFs |
| `demi-app` (bin `demi`) | GPL-3.0-only | Aplicación de escritorio (Slint) |
| `demi-cli` | GPL-3.0-only | Línea de comandos |

## Licencias

- La aplicación (`demi-app`, `demi-cli`) se distribuye bajo **GPL-3.0-only** ([LICENSE-GPL](LICENSE-GPL)).
- Las bibliotecas se distribuyen bajo **MIT** ([LICENSE-MIT](LICENSE-MIT)) **o** **Apache-2.0** ([LICENSE-APACHE](LICENSE-APACHE)), a tu elección.

Salvo que indiques lo contrario, toda contribución a un crate de biblioteca se licencia como MIT OR Apache-2.0, y toda contribución a `demi-app` o `demi-cli`, como GPL-3.0-only.
