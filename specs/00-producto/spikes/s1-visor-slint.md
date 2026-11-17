# Spike S1 · Visor en Slint — informe

> **Tarea:** `tasks.md` S1 · **Decisión afectada:** ADR-02 (Slint) · **Estado:** parcial — falta la medición en Windows con NVDA
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

## Pendiente para cerrar S1

- [ ] Prueba en Windows (equipo del autor) con la checklist `spikes/s1-visor-slint/PRUEBA.md`: fps, peor frame, memoria, NVDA.
- [ ] Decisión: mantener ADR-02 (Slint) o activar el plan B (widget de render propio con wgpu dentro de Slint).
