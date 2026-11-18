# Spike S3 · Fuente de verdad del documento — informe

> **Tarea:** `tasks.md` S3 · **Decisiones afectadas:** ADR-04 (`lopdf`), `design.md` §3.4 (`DocumentSession`) · **Estado:** cerrado (2026-11-17) — **se confirma ADR-04 (`lopdf` fuente de verdad)**
> **Código:** `spikes/s3-fuente-verdad/` (desechable) · **Medición multiplataforma:** workflow `Spike S3 · fuente de verdad` (PR #13)

## Pregunta

¿Puede `lopdf` ser la fuente de verdad del documento editable, con PDFium solo para leer y renderizar, aunque cada acción obligue a resincronizar PDFium?

**Criterio de éxito:** < 200 ms desde la acción hasta la página afectada visible, en un PDF de 1 000 páginas.
**Plan B:** PDFium como fuente de verdad y `lopdf` solo al guardar.

## Qué se construyó

`s3-bench` aplica tres acciones (girar página, añadir anotación cuadrada, eliminar página) sobre la página central de cada documento por tres caminos y mide cada paso (mediana de 5 repeticiones, tras un render de calentamiento):

| Camino | Pasos medidos |
|---|---|
| **A · `lopdf` fuente de verdad, recarga completa** | aplicar en `lopdf::Document` → serializar todo (`save_to`) → PDFium carga los bytes y renderiza la página afectada a 1024 px |
| **A' · `lopdf` con guardado incremental** | `IncrementalDocument`: copiar solo el objeto de la página, editarlo, añadir la sección incremental al final → PDFium recarga y renderiza |
| **B · PDFium fuente de verdad** | aplicar con la API de PDFium (`set_rotation`, `create_square_annotation`, `delete`) → renderizar; guardado completo aparte (`save_to_bytes`) |

Tras cada acción del camino A/A' el bench **comprueba que PDFium ve el cambio** (rotación de 90°, anotación `Square` presente, una página menos); si no, falla.

Documentos: PDFs de texto sintéticos de 10, 100 y 1 000 páginas, y tres del corpus de pdf.js de una página con mucha imagen o fuentes (`images.pdf`, `perf_images_cmyk.pdf`, `mixedfonts.pdf`).

## Resultados (Linux x64, contenedor de desarrollo, release)

**Carga:** 1 000 páginas → `lopdf` 3,6 ms, PDFium 0,4 ms. En el resto, < 1 ms en ambos.

Acción hasta página visible, PDF de 1 000 páginas:

| Acción | A: aplicar | A: guardar todo | A: recargar + render | **A: total** | A': guardar incremental (bytes añadidos) | **B: aplicar + render** | B: guardar todo |
|---|---|---|---|---|---|---|---|
| Girar página | 0,37 ms | 2,2 ms | 4,5 ms | **7,0 ms** | 0,5 ms (254 B) | **3,7 ms** | 29,6 ms |
| Añadir anotación | 0,39 ms | 2,3 ms | 5,4 ms | **8,1 ms** | 0,5 ms (395 B) | **4,8 ms** | 30,3 ms |
| Eliminar página | 1,21 ms | 2,2 ms | 5,3 ms | **8,7 ms** | — | **6,7 ms** | 27,7 ms |

En 10 y 100 páginas los dos caminos están entre 2 y 3 ms. En los documentos con imágenes el render domina y los dos caminos empatan (`images.pdf` 90–96 ms en A frente a 80–92 ms en B; CMYK 52–62 ms en ambos; `mixedfonts.pdf` 15–17 ms en ambos).

### Resultados en las 4 plataformas

Runners de GitHub, release, medianas de 5 repeticiones. Acción hasta página visible en el PDF de 1 000 páginas:

| Acción · camino | Linux x64 | Windows x64 | macOS arm64 | macOS Intel | Objetivo |
|---|---|---|---|---|---|
| Girar · **A** (`lopdf` + recarga) | 4,5 ms | 8,7 ms | 8,7 ms | 20,0 ms | < 200 ms ✅ |
| Girar · B (PDFium) | 2,4 ms | 5,6 ms | 3,9 ms | 11,0 ms | |
| Anotar · **A** | 5,4 ms | 9,7 ms | 7,0 ms | 16,9 ms | < 200 ms ✅ |
| Anotar · B | 4,6 ms | 6,4 ms | 4,8 ms | 10,7 ms | |
| Eliminar · **A** | 5,1 ms | 9,8 ms | 7,7 ms | 30,6 ms | < 200 ms ✅ |
| Eliminar · B | 4,1 ms | 9,3 ms | 6,8 ms | 26,8 ms | |
| Guardado incremental A' | 0,3–0,8 ms | 1,3 ms | 0,9–1,2 ms | 1,7–2,0 ms | |
| Guardado completo con PDFium | 19 ms | 28–31 ms | 30–34 ms | 68–111 ms | |
| Carga con `lopdf` / PDFium | 1,6 / 0,3 ms | 4,6 / 0,5 ms | 3,5 / 0,9 ms | 6,6 / 1,4 ms | |

Páginas con mucha imagen o fuentes (una página; A frente a B, girar · anotar):

| Documento | Linux x64 | Windows x64 | macOS arm64 | macOS Intel |
|---|---|---|---|---|
| `images.pdf` | 61 · 74 / 60 · 74 ms | 70 · 86 / 70 · 84 ms | 65 · 79 / 64 · 83 ms | 158 · 230 / 167 · 189 ms |
| `perf_images_cmyk.pdf` | 46 · 36 / 45 · 36 ms | 58 · 52 / 57 · 51 ms | 50 · 43 / 49 · 47 ms | 112 · 101 / 94 · 88 ms |
| `mixedfonts.pdf` | 10 · 11 / 10 · 10 ms | 34 · 34 / 33 · 34 ms | 18 · 19 / 17 · 17 ms | 50 · 49 / 46 · 49 ms |

En las cuatro plataformas PDFium vio todos los cambios hechos con `lopdf` (las comprobaciones del bench pasaron).

## Conclusiones

1. **El criterio se cumple con mucho margen en las 4 plataformas:** el camino A tarda entre 4,5 y 31 ms en 1 000 páginas (el peor caso es macOS Intel; objetivo: 200 ms). El sobrecoste de resincronizar frente a B es de 1–9 ms, invisible para el usuario.
2. **El coste de A crece con el tamaño del archivo** (serializar + que PDFium vuelva a parsear la tabla xref): unos 3 ms por cada 1 000 páginas de texto. Extrapolando, un PDF de 10 000 páginas rondaría los 30–40 ms: sigue lejos del límite.
3. **El guardado incremental de `lopdf` es casi gratis** (0,5 ms y unos cientos de bytes), mientras que el guardado completo con PDFium cuesta ~30 ms en 1 000 páginas. Además, el incremental es lo que necesita la firma PAdES (R-FIR-04) para no invalidar firmas previas.
4. **Eliminar página no tiene versión incremental en este spike** (`IncrementalDocument` solo copia objetos; borrar implica reescribir el árbol `/Pages`). Se puede hacer copiando el nodo `/Pages` padre; queda para T2.3.
5. **En páginas pesadas manda el render, no la fuente de verdad:** `images.pdf` en macOS Intel pasa de 200 ms con los dos caminos (230 ms A, 189 ms B al anotar). Eso lo resuelve el render por teselas, con vista previa a baja resolución y cancelación (T2.4), no el modelo del documento.
6. PDFium es rápido para aplicar cambios, pero tener dos modelos vivos (PDFium editado + `lopdf` al guardar) obliga a traducir cada cambio de uno a otro al final: más superficie de errores que resincronizar en una sola dirección.

## Decisión

**Se confirma ADR-04: `lopdf` es la fuente de verdad.** PDFium solo lee y renderiza los bytes que produce `lopdf`.

Notas para T2.3 (sesión de documento):
- Resincronizar con **recarga completa en memoria** desde los bytes de `lopdf` (más simple; < 10 ms en 1 000 páginas). Si un documento muy grande lo hiciera notar, pasar a recargar desde los bytes de A' (base + sección incremental).
- Mantener los bytes originales intactos y acumular las ediciones como secciones incrementales cuando sea posible: guardado rápido, deshacer barato y compatible con documentos firmados.
- Comprobar en los tests que PDFium ve cada cambio aplicado con `lopdf` (como hace este bench).
- Plan B descartado: el peor caso medido en 1 000 páginas son 31 ms.
- T2.4: en páginas con mucha imagen, mostrar primero la versión ya renderizada (girada o con la anotación dibujada por la UI) y sustituirla cuando llegue el render nuevo, para no depender de los 60–230 ms del render.
