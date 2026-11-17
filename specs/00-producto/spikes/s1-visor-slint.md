# Spike S1 · Visor en Slint — informe

> **Tarea:** `tasks.md` S1 · **Decisión afectada:** ADR-02 (Slint) · **Estado:** cerrado (2026-11-17) — **se mantiene Slint**
> **Código:** `spikes/s1-visor-slint/` (desechable) · **Artefacto Windows:** workflow `Spike S1 · visor Slint`

## Pregunta

¿Puede Slint mostrar un PDF de 500 páginas con desplazamiento continuo fluido, zoom, una capa vectorial de anotaciones editable y un árbol de accesibilidad con el texto de cada página?

**Criterio de éxito:** 60 fps al desplazarse (peor frame < 33 ms) en el equipo de referencia; NVDA y VoiceOver anuncian los botones y leen el texto de una página.

## Qué se construyó

- Slint 1.19 + `demi-engine` (PDFium chromium/8125) en el mismo proceso (sin worker; eso lo mide S2).
- **Virtualización propia:** Rust entrega a Slint solo las páginas visibles ± 2, cada una con su posición `y`; el `Flickable` conoce la altura total. Caché de imágenes de ± 6 páginas.
- Hilo de render dedicado que atiende solo la petición más reciente y abandona la anterior si llega otra (cancelación).
- Zoom: Ctrl + rueda, Ctrl +/-/0 y botones. Mientras llega el render nuevo se muestra la imagen anterior escalada.
- Capa vectorial: 50 rectángulos arrastrables (5 en cada una de las 10 primeras páginas) dibujados en Slint, encima del bitmap.
- Accesibilidad: cada página es un nodo con rol `text` y etiqueta "Página N. <texto extraído>"; botones con nombre; anotaciones como botones con nombre.
- Medidor en pantalla: fps, peor frame, ms de render por página y páginas en memoria.

## Resultados en Linux (Xvfb, renderizador por software)

| Prueba | Resultado |
|--------|-----------|
| Abrir y mostrar el PDF de 500 páginas | ✅ |
| Desplazamiento continuo con rueda (150 pasos) | ✅ páginas correctas, 6–12 páginas en memoria |
| Render por página (A4 de texto, 100 %, release) | ✅ 2–5 ms |
| Zoom con Ctrl + rueda → texto nítido al nuevo zoom | ✅ |
| Arrastrar anotaciones | ✅ (tras desactivar el arrastre-para-desplazar, ver hallazgo 3) |
| fps / peor frame | ⏳ el medidor no funciona con el renderizador por software; se mide en Windows |
| NVDA / Narrador | ⏳ pendiente (Windows) |
| VoiceOver | ⏳ pendiente (sin Mac disponible) |

## Hallazgos que cambian el diseño

1. **El `ListView` estándar de Slint no sirve para el visor:** no expone `interactive`. Se usa un `Flickable` con virtualización controlada desde Rust, que además encaja con `design.md` §6.2 (Rust decide qué páginas existen).
2. **El `Flickable` se queda con la rueda ~800 ms después de desplazarse**, así que Ctrl + rueda no llegaba a las páginas. Solución: un `FocusScope` raíz con `capture-key-pressed` detecta Ctrl y desactiva `interactive` mientras está pulsado.
3. **Arrastrar con el ratón desplazaba el documento** en lugar de mover la anotación. Solución: `mouse-drag-pan-enabled: false` (en escritorio el desplazamiento con arrastre será la herramienta Mano).
4. **`pdfium-render` entrega RGBA, no BGRA**, porque activa `FPDF_REVERSE_BYTE_ORDER` por defecto. Se corrige el arnés de regresión visual (`Gray::from_rgba`) y se regeneran sus referencias.
5. La propiedad `viewport-*` del `Flickable` está obsoleta en Slint 1.19: usar `content-*`.

## Resultados en Windows (equipo del autor, 2026-11-17)

Ejecutable del workflow `Spike S1 · visor Slint` (renderizador por defecto de Slint), checklist `PRUEBA.md`.

| Prueba | Objetivo | Resultado |
|--------|----------|-----------|
| fps al desplazarse rápido con la rueda | 60 fps | ✅ **189 fps** |
| Peor frame | < 33 ms | ✅ **11 ms** |
| Páginas en blanco antes de dibujarse | mínimo | ✅ no se perciben |
| Memoria tras recorrer las 500 páginas | acotada | ✅ **92 MB** |
| Zoom (Ctrl + rueda, Ctrl +/-/0, botones) y nitidez tras el zoom | funciona | ✅ |
| Arrastrar anotaciones, también tras el zoom | fluido | ✅ |
| NVDA: botones, texto de cada página, anotaciones | se anuncian | ✅ (ver seguimiento 3) |
| PDF real del autor (tesis) | se abre y navega bien | ✅ |
| Barra de desplazamiento | se puede arrastrar | ⚠️ el prototipo no la tiene (seguimiento 1) |
| VoiceOver (macOS) | se anuncian | ⏳ sin Mac disponible; se verificará en T3.4 |

## Decisión

**Se mantiene ADR-02 (Slint).** El criterio de éxito se cumple con amplio margen en el equipo del autor (189 fps frente a 60; peor frame de 11 ms frente a 33), con memoria acotada y un árbol de accesibilidad que NVDA recorre. No se activa el plan B (render propio con wgpu).

## Seguimiento para el visor de producción (T3.1, T3.4)

1. **Barra de desplazamiento** propia, accesible con teclado y lector de pantalla (el `Flickable` no trae una).
2. **Botones con los componentes propios de Demi** (T3.1): los de estilo por defecto de Slint parecen deshabilitados y tienen poco contraste.
3. **El nombre accesible debe contener el texto visible** (WCAG 2.5.3, *label in name*): el botón "100 %" se anunciaba como "Tamaño real". Regla general para todos los controles: p. ej. "100 % (tamaño real)".
4. Mantener la virtualización controlada desde Rust, el `FocusScope` raíz para Ctrl + rueda y `mouse-drag-pan-enabled: false` (hallazgos 1–3).
5. Medir de nuevo con PDFs escaneados e imágenes grandes cuando el render pase por el worker (S2) y con teselas (T3.4).
