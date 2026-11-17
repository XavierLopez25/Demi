# Changelog

Formato basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/); versionado [SemVer](https://semver.org/lang/es/).

## [Sin publicar]

### Añadido
- T0.1: workspace de Cargo con los crates `demi-core`, `demi-engine`, `demi-text`, `demi-sign`, `demi-ipc`, `demi-store`, `demi-worker`, `demi-app`, `demi-cli` y `xtask`.
- Licencias: GPL-3.0-only para la aplicación; MIT OR Apache-2.0 para las bibliotecas.
- Verificación de licencias con `cargo-deny` (`deny.toml` para el workspace, `deny-lib.toml` sin GPL para las bibliotecas).
- `README.md`, `CONTRIBUTING.md`, `SECURITY.md`.
- T0.2: CI en GitHub Actions (formato, clippy, cargo-deny, tests en Linux x64, Windows x64, macOS arm64 y macOS Intel, MSRV 1.85), trabajo nocturno de avisos de seguridad, Dependabot y plantilla de PR.
- T0.3: `cargo xtask pdfium fetch|dir` descarga PDFium chromium/8125 fijado con SHA-256 por plataforma (`xtask/pdfium.lock`); `demi-engine` integra `pdfium-render` 0.9.5 y un test que renderiza la página 1 de un PDF de ejemplo; CI descarga PDFium en las cuatro plataformas.
- T0.4: corpus de pruebas (`corpus/manifest.toml`): 35 PDFs de la suite de pdf.js fijados por commit y SHA-256 y 8 casos borde sintéticos; `cargo xtask corpus fetch|list`; test `corpus_smoke` que verifica el resultado esperado de cada documento; caché del corpus en CI.
- T0.5: regresión visual: la primera página de cada documento del corpus `smoke` se compara por SSIM con referencias PNG versionadas (`crates/demi-engine/tests/golden/`), umbrales por documento en `golden.toml`, `cargo xtask golden update` para regenerarlas y artefacto con las diferencias en CI.
- S1 (spike): visor en Slint con virtualización propia, zoom con Ctrl + rueda, capa de anotaciones arrastrables y nodos accesibles por página (`spikes/s1-visor-slint/`); workflow que publica un ejecutable de Windows para probarlo; informe en `specs/00-producto/spikes/s1-visor-slint.md`.
- S2 (spike): proceso aislado (`s2-worker`) con socket local + anillo de teselas en memoria compartida y medición en las 4 plataformas por CI (`spikes/s2-worker-ipc/`, informe en `specs/00-producto/spikes/s2-worker-ipc.md`).

### Corregido
- Tests de render: los bitmaps de `pdfium-render` vienen en RGBA (no BGRA); referencias visuales regeneradas.
