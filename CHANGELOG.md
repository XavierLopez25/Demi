# Changelog

Formato basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/); versionado [SemVer](https://semver.org/lang/es/).

## [Sin publicar]

### Añadido
- T0.1: workspace de Cargo con los crates `demi-core`, `demi-engine`, `demi-text`, `demi-sign`, `demi-ipc`, `demi-store`, `demi-worker`, `demi-app`, `demi-cli` y `xtask`.
- Licencias: GPL-3.0-only para la aplicación; MIT OR Apache-2.0 para las bibliotecas.
- Verificación de licencias con `cargo-deny` (`deny.toml` para el workspace, `deny-lib.toml` sin GPL para las bibliotecas).
- `README.md`, `CONTRIBUTING.md`, `SECURITY.md`.
- T0.2: CI en GitHub Actions (formato, clippy, cargo-deny, tests en Linux x64, Windows x64, macOS arm64 y macOS Intel, MSRV 1.85), trabajo nocturno de avisos de seguridad, Dependabot y plantilla de PR.
