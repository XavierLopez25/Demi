# Corpus de pruebas

Los PDFs **no se versionan**: solo `manifest.toml`.

```sh
cargo xtask corpus fetch               # conjunto "smoke" (por defecto)
cargo xtask corpus fetch --set full    # todo el corpus
cargo xtask corpus list                # contenido del manifiesto
```

- **Descargados** (`corpus/<origen>/…`): archivos de la suite de pruebas de pdf.js, fijados por commit y verificados por SHA-256.
- **Sintéticos** (`corpus/generated/…`): casos borde generados de forma determinista por `xtask/src/pdfgen.rs` (archivo vacío, basura antes de `%PDF`, truncado, xref roto, 1 000 páginas, página A0, 100 000 operadores, nombre de archivo no ASCII).

Cada entrada declara `expect` (`ok`, `password` o `error`). El test `crates/demi-engine/tests/corpus_smoke.rs` comprueba que el motor se comporta así con cada documento.

Para añadir un archivo: súmalo al manifiesto con su SHA-256, etiquetas y `expect`, ejecuta `cargo xtask corpus fetch` y `cargo test -p demi-engine --test corpus_smoke`.
