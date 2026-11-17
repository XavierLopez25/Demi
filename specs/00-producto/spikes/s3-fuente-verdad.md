# Spike S3 · Fuente de verdad del documento — informe

> **Tarea:** `tasks.md` S3 · **Decisiones afectadas:** ADR-04 (`lopdf`), `design.md` §3.4 (`DocumentSession`) · **Estado:** abierto — resultados de Linux; faltan los de las 4 plataformas (workflow `Spike S3 · fuente de verdad`)
> **Código:** `spikes/s3-fuente-verdad/` (desechable)

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

_Pendiente: se completa con el resumen del workflow `Spike S3 · fuente de verdad` del PR._

## Conclusiones

1. **El criterio se cumple con mucho margen:** el camino A tarda menos de 9 ms en 1 000 páginas (objetivo: 200 ms). El sobrecoste de resincronizar frente a B es de 2–4 ms, invisible para el usuario.
2. **El coste de A crece con el tamaño del archivo** (serializar + que PDFium vuelva a parsear la tabla xref): unos 3 ms por cada 1 000 páginas de texto. Extrapolando, un PDF de 10 000 páginas rondaría los 30–40 ms: sigue lejos del límite.
3. **El guardado incremental de `lopdf` es casi gratis** (0,5 ms y unos cientos de bytes), mientras que el guardado completo con PDFium cuesta ~30 ms en 1 000 páginas. Además, el incremental es lo que necesita la firma PAdES (R-FIR-04) para no invalidar firmas previas.
4. **Eliminar página no tiene versión incremental en este spike** (`IncrementalDocument` solo copia objetos; borrar implica reescribir el árbol `/Pages`). Se puede hacer copiando el nodo `/Pages` padre; queda para T2.3.
5. PDFium es rápido para aplicar cambios, pero tener dos modelos vivos (PDFium editado + `lopdf` al guardar) obliga a traducir cada cambio de uno a otro al final: más superficie de errores que resincronizar en una sola dirección.

## Decisión

**Se confirma ADR-04: `lopdf` es la fuente de verdad.** PDFium solo lee y renderiza los bytes que produce `lopdf`.

Notas para T2.3 (sesión de documento):
- Resincronizar con **recarga completa en memoria** desde los bytes de `lopdf` (más simple; < 10 ms en 1 000 páginas). Si un documento muy grande lo hiciera notar, pasar a recargar desde los bytes de A' (base + sección incremental).
- Mantener los bytes originales intactos y acumular las ediciones como secciones incrementales cuando sea posible: guardado rápido, deshacer barato y compatible con documentos firmados.
- Comprobar en los tests que PDFium ve cada cambio aplicado con `lopdf` (como hace este bench).
- Plan B descartado por ahora; se reabre si la medición en otras plataformas superase los 100 ms.
