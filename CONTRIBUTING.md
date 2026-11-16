# Contribuir a Demi

¡Gracias por tu interés! Demi se desarrolla con **Spec-Driven Development**.

## Flujo de trabajo

1. Toda funcionalidad parte de un requisito en `specs/00-producto/requirements.md` (ID `R-…`) y de una tarea en `specs/00-producto/tasks.md` (ID `T…` o `S…`).
2. Las funciones grandes tienen su propio spec en `specs/<nn>-<nombre>/` con `requirements.md`, `design.md` y `tasks.md`.
3. Una tarea = una rama = un PR. Nombre de rama: `t0.1-workspace`, `s1-visor-slint`, etc.
4. El PR enlaza la tarea y los requisitos, y cumple la **definición de terminado** de `tasks.md` §0.

## Antes de abrir un PR

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
```

## Reglas

- Nada de `unwrap()`/`expect()` en código que procese datos de un PDF.
- Ningún texto de interfaz fijo en el código: todo se traduce (es/en).
- Las bibliotecas (MIT/Apache) no pueden depender de código GPL ni AGPL.
- Prohibido copiar código de proyectos AGPL (MuPDF, PyMuPDF, iText) aunque sirvan de referencia.
- Accesibilidad: todo lo nuevo en la interfaz es usable con teclado y tiene nombre accesible.
