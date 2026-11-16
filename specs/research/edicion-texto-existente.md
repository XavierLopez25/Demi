# UC-EDI-02 — Editar texto existente en un PDF: investigación y recomendación

*Fecha: noviembre de 2026. Stack objetivo: Rust, `pdfium-render` + `lopdf` + `krilla`/`pdf-writer`, `hayro` como alternativa.*

**Convención de marcado:**
- **[V]**: verificado en una fuente consultada en esta investigación (ver "Fuentes").
- **[I]**: inferencia o recomendación propia, no verificada en una fuente.
- **[C]**: conocimiento común del sector que no se volvió a verificar en esta sesión.

---

## Resumen ejecutivo

1. Un PDF no guarda párrafos. Guarda glifos posicionados en content streams, con codificaciones específicas de cada fuente y fuentes que suelen estar *subset* (incrustadas solo con los glifos usados). Por eso "editar texto" es en realidad un problema de reconstrucción de layout más reescritura de operadores **[V]**.
2. Ninguna biblioteca libre compatible con MIT/Apache ofrece edición de párrafos lista para usar. PDFium tiene las piezas de bajo nivel: `FPDFText_SetText` reencodifica con la fuente del objeto, `FPDFText_LoadFont` incrusta fuentes nuevas, `FPDFPage_GenerateContent` regenera la página y existe el flag experimental `FPDF_SUBSET_NEW_FONTS`. No hace detección de párrafos ni *reflow* **[V]**.
3. **Recomendación: un enfoque por niveles con un modelo propio de "bloques de texto" en el crate núcleo.**
   - **N0 (fallback, S–M):** eliminar de verdad los glifos originales y superponer texto nuevo, con un aviso claro.
   - **N1 (M):** editar una línea o palabra dentro de un mismo objeto de texto, reutilizando la fuente incrustada solo si todos los glifos existen. Primero con PDFium (`SetText` + `GenerateContent`) y después con un parche quirúrgico del content stream vía `lopdf`.
   - **N2 (L):** sustituir la fuente incrustando una nueva (fuente del sistema con el mismo nombre si su licencia lo permite, si no Liberation/Croscore/Noto con métricas compatibles) como Type0/Identity-H, subset con `subsetter` o `skera`.
   - **N3 (XL):** edición de párrafo con *reflow*, usando `parley` (shaping con `harfrust`) y reemitiendo las líneas como `TJ`.
4. **Pruebas:** corpus público (pdf.js, PDFBox, el de hayro), SSIM con máscara fuera de la zona editada (≈1,0), *round-trip* de extracción de texto, `qpdf --check`, y doble renderizador (PDFium + hayro).
5. **Licencias:** MuPDF/PyMuPDF e iText son AGPL, así que solo sirven como referencia de comportamiento, sin copiar código. Poppler es GPL, así que nunca va en el crate núcleo MIT/Apache. Las fuentes de sustitución recomendadas (Liberation ≥2.0 y Noto bajo OFL, Croscore bajo Apache-2.0) son redistribuibles **[V]**.

---

## 1. Por qué es difícil

### 1.1 Modelo de contenido
- **[V]** Un PDF guarda caracteres y sus posiciones, no palabras, líneas ni párrafos. Las herramientas de extracción reconstruyen esa estructura con heurísticas geométricas, y hasta los espacios entre palabras son inferidos (pdfminer.six). La evaluación de Bast y Korzen (JCDL 2017) cuantifica errores de límites de palabra, orden de lectura en dos columnas, guiones, ligaduras y párrafos partidos por figuras.
- **[C]** El texto se dibuja dentro de `BT … ET` con estos operadores:
  - `Tf` fija fuente y tamaño. `Tm`, `Td`, `TD` y `T*` fijan la matriz de texto. `Tc` (espaciado entre caracteres), `Tw` (entre palabras), `Tz` (escala horizontal), `TL` (interlineado) y `Ts` (*rise*) modifican el estado.
  - `Tj`, `'` y `"` muestran cadenas. `TJ` muestra arrays que mezclan cadenas y ajustes numéricos de posición, en milésimas de em.
  - El CTM (`cm`) y los `q/Q` anidados se componen con la matriz de texto.
  - Un generador puede emitir un `Tj` por glifo, partir palabras entre varios `TJ`, o dibujar las líneas en un orden distinto al de lectura.
- **[V]** El kerning no se lee de la fuente: el creador del PDF lo hornea en línea dentro de `TJ`. GSUB/GPOS (ligaduras, posicionamiento contextual) los resuelve quien genera el PDF, no el visor (nibblestew, 2023). Al editar, **nosotros** tenemos que hacer el shaping.

### 1.2 Fuentes
- **[V] Subsets.** Normalmente solo se incrustan los glifos usados, marcados con un prefijo de 6 letras. Si la palabra nueva necesita un glifo ausente, no se puede dibujar. Esto incluye diferencias de mayúsculas y minúsculas: puede existir la "a" y no la "A" (Apryse, Qoppa PDF Studio, Master PDF Editor, artículo "PDF font hell").
- **[V] Codificación.** El content stream contiene códigos de byte, no Unicode. El mismo byte cambia de significado según la codificación (WinAnsi, MacRoman, `/Differences`, Identity-H). `ToUnicode` es opcional, así que convertir Unicode → código no siempre es posible ("PDF font hell"; Apryse menciona codificaciones propias).
- **[V] Anchos.** Las fuentes simples usan `/Widths` y las CID usan `/W` disperso. Ambos se fijan al crear el PDF, y una fuente sustituta con otros avances hace que el texto se desplace o desborde.
- **[V] Métricas.** Ascent, descent, CapHeight y BBox del descriptor también afectan.
- **[V] Fuentes simples y CID.** Las simples usan códigos de 1 byte (máximo 256 por objeto de fuente). Type0/CID usa una CMap con una capa extra código → CID → GID. Type0/Identity-H con códigos de 2 bytes y una CMap ToUnicode es la forma práctica de incrustar fuentes nuevas (nibblestew, "PDF font hell").
- **[V] Type3.** Los glifos son procedimientos de dibujo, no un programa de fuente. Stirling-PDF los marca como "no soportados" salvo que encuentre una fuente equivalente en su biblioteca. Bast y Korzen los citan como fuente de errores de extracción.
- **[I] Fuentes no incrustadas** (Base-14 o referenciadas por nombre). Editarlas es "fácil" en codificación, pero el visor sustituye la fuente. Si añadimos glifos fuera de WinAnsi, hay que pasar a una fuente incrustada.
- **[V] Licencia de la fuente.** Los permisos de incrustación (OS/2 `fsType`) pueden impedir incrustar o editar (Adobe Community, 2025; PDF-XChange marcaba como no incrustables fuentes con metadatos incompletos).

### 1.3 Otros factores
- **[C] Form XObjects (`Do`).** Pueden contener texto con su propia matriz y recursos. Editar dentro de un XObject compartido por varias páginas cambia todas las páginas.
- **[V] PDFium.** `FPDFPageObj_GetMatrix` devuelve matrices relativas a la página o al Form XObject padre, según dónde esté el objeto.
- **[C] Recorte y máscaras.** Clip paths (`W n`), modos de renderizado de texto (`Tr` 7 = texto como recorte, 3 = invisible, el que usan las capas OCR) y transparencias.
- **[V] PDFium conserva el recorte.** Tiene tests explícitos de que el clip se mantiene tras `SetText` + regeneración (`SetTextKeepClippingPath`, `Bug1574`).
- **[C] Tagged PDF.** El texto va envuelto en `BDC /P <</MCID n>> … EMC`, enlazado con el árbol de estructura. También puede haber `/ActualText` (por ejemplo para ligaduras o guiones) y spans `/Span` con `/Lang`. Si se reescribe el contenido hay que conservar los MCID, o la accesibilidad se rompe.
- **[V] Marcas en PDFium.** Sus tests verifican que las marcas sobreviven a eliminar otros objetos y guardar (`MaintainMarkedObjects`).
- **[I] Firmas digitales.** Cualquier edición invalida la firma de la revisión firmada. El guardado incremental conserva la revisión firmada intacta, pero el documento deja de estar "firmado sin cambios". Hay que avisar al usuario.

---

## 2. Algoritmos y técnicas

### 2.1 Agrupar glifos en líneas, bloques y párrafos
- **[V] pdfminer.six (LAParams).** Agrupa en tres etapas:
  1. Caracteres en líneas: distancia horizontal menor que `char_margin` (relativo al ancho de carácter) y solapamiento vertical mayor que `line_overlap`. Inserta espacios cuando el hueco supera `word_margin`.
  2. Líneas en cajas: solapamiento horizontal y cercanía vertical según `line_margin`.
  3. Agrupación jerárquica de cajas por un criterio de área.

  Tiene `detect_vertical` para escritura vertical. Es heurístico y muy sensible a los parámetros.
- **[V] MuPDF/PyMuPDF (stext).** Jerarquía bloque → línea → span → char. Cada span lleva fuente, tamaño, flags (negrita, cursiva, serif, monoespaciada, superíndice), color, alpha, origen de la línea base, ascender/descender y `char_flags` (subrayado, tachado, sintético, recortado). Es un buen **modelo de datos** a imitar (el diseño, no el código, por ser AGPL).
- **[V] PDFium (FPDFText).** Desde el text page ofrece:
  - Por carácter: caja ajustada y caja "loose", origen, matriz, info de fuente, peso y color de relleno.
  - `FPDFText_IsGenerated`, para distinguir espacios y saltos insertados por PDFium.
  - `FPDFText_HasUnicodeMapError`.
  - `FPDFText_GetTextObject`, que da el objeto de texto que contiene un carácter. **Esto permite mapear selección → objeto editable.**

  No expone párrafos.
- **[V] Literatura de segmentación** (recopilada en BobLd/DocumentLayoutAnalysis, con implementaciones en PdfPig, que es Apache-2.0):
  - Recursive XY-Cut: descendente, por perfiles de proyección.
  - Docstrum (O'Gorman): ascendente, por vecinos más cercanos.
  - Voronoi (Kise).
  - *Whitespace cover* de Breuel: rectángulos de blanco como separadores de columna.
- **[I] Párrafos para editar** (recomendación). Basta una versión acotada en tres pasos:
  1. Líneas por línea base común y dirección (usando la matriz de cada char).
  2. Líneas en bloques por interlineado constante (±10 %), solape horizontal y estilo dominante compatible.
  3. Detección de alineación (izquierda, derecha, centrada, justificada) comparando los extremos x de las líneas, de la sangría de primera línea, y de si la última línea es corta y el resto justificadas.

  El usuario debe poder **corregir la agrupación** (fusionar o partir bloques), como piden los usuarios de LibreOffice Draw, que importa cada línea como un cuadro separado (bug 125838 / 32249) **[V]**.

### 2.2 Reflow tras edición
- **[I]** Para reflow hacen falta cuatro cosas:
  1. Runs de estilo (fuente, tamaño, color, *rise*).
  2. Una caja (ancho = extensión del bloque, posición de la primera línea base).
  3. Interlineado medido.
  4. Alineación.

  Con eso se hace shaping (harfrust) + *line breaking* (UAX#14) + justificación, y se reemiten las líneas. La justificación se puede reproducir con `Tw` (solo afecta al byte 32 en fuentes simples) o, de forma más robusta, con ajustes en `TJ` por hueco. Las líneas que excedan la caja deben avisar (desbordamiento), no empujar contenido que no es nuestro.
- **[V]** PyMuPDF `insert_htmlbox` es la referencia de "relleno de caja con HTML/CSS y shaping HarfBuzz". Reduce la escala (`scale_low`) si el texto no cabe, que es una estrategia útil como opción.

### 2.3 Glifos ausentes en subsets
- **[I]** Hay cuatro estrategias, en orden de fidelidad:
  - **(a)** Reutilizar la fuente si todos los glifos existen. Para comprobarlo: codificar Unicode → código vía ToUnicode inverso o codificación, y verificar que el GID tiene contorno, con `skrifa` sobre los bytes de `FPDFFont_GetFontData`, o con `FPDFFont_GetGlyphPath`.
  - **(b)** Si la fuente **completa** está instalada (mismo PostScript name sin prefijo de subset) y su `fsType` lo permite, incrustar un subset nuevo de esa fuente para el texto editado. Es lo que hace Master PDF Editor con "cambiar fuente automáticamente si está instalada" **[V]**.
  - **(c)** Sustituir por una fuente métricamente compatible (Liberation/Arimo para Arial/Helvetica, Tinos para Times, Cousine para Courier, Carlito para Calibri, Caladea para Cambria) **[V]**, o por una de la misma clase (serif/sans/mono, peso, cursiva) según flags del descriptor y `PdfFont::is_serif()` y similares. La clasificación por familia y peso es exactamente lo que hizo Stirling-PDF con Liberation, DejaVu y Noto **[V]**.
  - **(d)** "Extender" el subset existente, añadiendo glifos al programa de fuente incrustado. Solo es viable si hay una fuente completa idéntica de la que copiar los contornos. Es frágil por la renumeración de GIDs, `/W`, CIDToGIDMap y tablas hint, así que **se recomienda no hacerlo** y usar (b).
- **[V]** Apryse recomienda en su foro reemplazar el *run* entero con una fuente nueva parecida en vez de parchear dentro de un subset.

### 2.4 Reescribir el stream o tapar y superponer
- **[V]** PyMuPDF/Artifex documentan dos vías:
  - Rectángulo blanco + `insert_text`: solo oculta, y el texto original sigue siendo extraíble.
  - Redacción (`add_redact_annot` + `apply_redactions`): elimina de verdad los caracteres cuya caja solapa, y luego inserta el texto de reemplazo con una fuente Base-14 o CJK, reduciendo el tamaño hasta 4 pt si no cabe.

  El propio artículo de Artifex reconoce que no hay reflow y que la coincidencia de fuente es imperfecta.
- **[V] Stirling-PDF.** Su pipeline intenta primero **reescribir los tokens existentes** del stream y, si no puede, regenera la página completa. Las fuentes candidatas siguen este orden: biblioteca → payload incrustado → *fallback* para glifos ausentes.
- **[I] Conclusión.** La reescritura del stream es la meta. "Whiteout" sin eliminación es inaceptable para un editor serio porque deja el texto viejo extraíble y buscable, lo que es un riesgo de privacidad. El fallback debe **eliminar** los glifos del stream y luego superponer.

### 2.5 Deshacer y guardado
- **[I] Undo.** Patrón *command* sobre el modelo propio (bloque editado: texto anterior y nuevo, rango de operadores afectados, recursos añadidos). Conviene guardar el content stream original de la página para un undo trivial (snapshot por página editada) hasta guardar.
- **[V] Guardado incremental.** PDFium tiene `FPDF_INCREMENTAL` en `FPDF_SaveAsCopy` y `FPDF_SUBSET_NEW_FONTS` (experimental), que hace subset de las fuentes nuevas al guardar. `lopdf` permite `Document::new_from_prev` para actualizaciones incrementales.

---

## 3. Cómo lo hacen las herramientas existentes

- **Adobe Acrobat Pro.**
  - **[V]** La ayuda describe cambio de fuente, tamaño, color, alineación, interlineado, espaciado de párrafo y carácter, y escala horizontal. Recomienda fuentes incrustadas y advierte que los layouts complejos requieren ajustes manuales.
  - **[V]** Según el foro (2025), no se puede cambiar o editar una fuente no incrustada o con restricciones de licencia de incrustación.
  - **[C]** Acrobat detecta párrafos y hace reflow dentro del cuadro. Si la fuente es subset y no está instalada, sustituye.
  - No hay documentación pública del algoritmo.
- **Foxit PDF Editor.**
  - **[V]** "Edit Text" por párrafo con ajuste automático de línea al llegar al margen, Enter para línea nueva, rotación del cuadro y panel de formato.
  - **[V]** Desde la v12 tiene la opción "incrustar fuentes automáticamente cuando el texto se actualiza". Las versiones anteriores convertían el texto editado a trazos.
- **PDF-XChange Editor.**
  - **[V]** El hilo consultado trata errores "glyph is missing in embedded font" al guardar PDF/A. El problema estaba en los permisos y metadatos de la fuente, y los mensajes de error eran engañosos.
  - Lección: validar las fuentes antes de incrustar y dar mensajes claros.
- **Master PDF Editor.**
  - **[V]** Falla con "This font doesn't contain these characters". Ofrece cambiar de familia o "cambiar fuente automáticamente" si el original está instalado.
  - **[V]** No permite estilos mixtos dentro de un objeto de texto sin partirlo.
- **Qoppa PDF Studio.**
  - **[V]** Avisa "Some characters are not supported by this font and may be lost" con subsets y ofrece fuentes estándar o de documento.
  - Lección de UX: avisar **antes** de perder caracteres.
- **Stirling-PDF 2.x.**
  - **[V]** Editor de texto en alfa (dic. 2025), inicialmente para usuarios de pago. El modelo es open-core.
  - **[V]** Convierte el PDF a JSON (páginas, fuentes, elementos con `charCodes`), el frontend edita y el backend reescribe tokens o regenera la página.
  - **[V]** Su backlog pendiente: Type3 (sintetizar TTF), avisar de glifos de fallback, preservar color y tratar bien los espacios.
  - **[C]** Es Java y usa PDFBox. No se verificó en esta sesión qué partes son MIT y cuáles propietarias.
- **LibreOffice Draw.**
  - **[V]** Importa el PDF dividiendo cada línea en un cuadro de texto. Al combinar cuadros se pierde la editabilidad (bug 125838, duplicado de 32249).
  - Contraejemplo útil: sin agrupación en párrafos la edición es penosa.
- **Inkscape.**
  - **[V]** Con "Internal import" y "Substitute missing fonts" el texto es editable. Cairo import y "Draw missing fonts" convierten el texto en trazos. Es por página y la fidelidad tipográfica es aproximada.
- **pdf.js / Firefox.**
  - **[V]** El editor de Firefox añade anotaciones (FreeText, tinta, resaltado, firma). No documenta la edición del texto existente.
  - Sirve de referencia para la UX de FreeText (nuestro UC de "añadir texto"), no para UC-EDI-02.
- **PyMuPDF / MuPDF (AGPL-3.0 o comercial).**
  - **[V]** Ofrece redacción real (opciones de texto, imagen y gráficos), `insert_htmlbox` (HTML/CSS, HarfBuzz, encoge si no cabe) y el modelo stext (dict/rawdict).
  - **[V]** El changelog de MuPDF 1.25–1.27 no anuncia edición de texto. Esa función no existe en MuPDF, solo se aproxima con redact + insert.
- **iText pdfSweep / pdfCleanUp.**
  - **[V]** Add-on para eliminar contenido dentro de regiones o redacciones.
  - **[V]/[C]** Licencia AGPL o comercial según el pie de iText. Solo es referencia.
- **Apache PDFBox (Apache-2.0 [C]).**
  - **[V]** Sus mantenedores (Tilman Hausherr, mkl) dicen que reemplazar `Tj` "normalmente no funciona". Solo funciona con fuentes WinAnsi no subset, líneas dibujadas enteras y sin kerning. Para PDFs arbitrarios es "extremadamente difícil o imposible".
  - Ofrece las piezas de parseo y escritura de streams, pero no edición de alto nivel.
- **qpdf / pikepdf (Apache-2.0 [C]).**
  - **[V]** `TokenFilter` reescribe content streams token a token y deja intacto lo que no cambia.
  - Es el patrón ideal para un parche quirúrgico: modificar solo los tokens de la palabra editada.
- **Okular / Poppler (GPL [C]).** Solo anotaciones, sin edición del texto del contenido.
- **Apryse / PDFTron (comercial).** **[V]** Recomiendan reemplazar el run completo con `ElementBuilder` y una fuente nueva de métricas parecidas.
- **PDFDancer (comercial, 2026).** **[V]** Material promocional que afirma edición "sin overlays": reconstruye líneas y párrafos, reutiliza la fuente incrustada si se puede y, si no, usa una fuente OFL similar con reflow. No verificable. Confirma que es la dirección del mercado.

---

## 4. Estado del ecosistema Rust (crates.io, consultado el 15-11-2026)

| Crate | Versión (fecha) | Licencia | Papel en UC-EDI-02 |
|---|---|---|---|
| `pdfium-render` | 0.9.5 (2026-11-01) | MIT OR Apache-2.0 | Extracción de chars con caja y matriz, char → objeto, `PdfPageTextObject::set_text`, fuentes (`is_embedded`, `data`, `glyphs`, flags serif y demás), `load_true_type_from_bytes(.., is_cid_font)`, estrategias de regeneración de contenido |
| `lopdf` | 0.47.0 (2026-11-04) | MIT | `Content::decode/encode` de operaciones, `change_page_content`, `new_from_prev` (incremental). Tiene `replace_text`/`replace_partial_text` sin documentación de límites: no fiables con subsets ni codificaciones [I] |
| `hayro` / `hayro-syntax` / `hayro-interpret` / `hayro-write` | 0.9.0 (2026-11-02) | Apache-2.0 OR MIT | Intérprete puro Rust con `Device` abstracto (útil para el mapeo operador ↔ glifo) y reescritura de páginas. Experimental. No soporta CID no incrustadas |
| `krilla` | 0.8.2 (2026-06-04) | MIT OR Apache-2.0 | Crea PDF y hace subsets de TTF/CFF. No edita PDFs existentes |
| `pdf-writer` | 0.15.0 (2026-05-27) | MIT OR Apache-2.0 | Escritura de bajo nivel de diccionarios (Type0, CIDFontType2, ToUnicode CMap, `/W`), útil para incrustar fuentes en un PDF existente vía `lopdf` [I] |
| `subsetter` (typst) | 0.2.6 (2026-06-04) | MIT OR Apache-2.0 | Subset TrueType/CFF solo para incrustar en PDF. Sin instanciado de fuentes variables |
| `skera` (fontations) | 0.10.0 (2026-11-10) | MIT OR Apache-2.0 | Subsetter de Google Fonts, más general. Lo usa HarfRust |
| `skrifa` / `read-fonts` / `write-fonts` | 0.50 / 0.47 / 0.56 (2026-11-10) | MIT OR Apache-2.0 | Lectura de métricas, cmap y contornos, y escritura de fuentes |
| `ttf-parser` | 0.25.1 (2024-11-29) | MIT OR Apache-2.0 | Maduro pero sin releases desde 2024. Preferir fontations [I] |
| `harfrust` | 0.16.0 (2026-11-12) | MIT | Port de HarfBuzz (iguala HarfBuzz 14.3.1) sobre read-fonts. Sucede de facto a rustybuzz |
| `rustybuzz` | 0.20.1 (2024-11-12) | MIT | Sin releases desde 2024. Evitar en código nuevo [I] |
| `parley` (+ `fontique`) | 0.13.0 (2026-11-12) | Apache-2.0 OR MIT | Layout de texto enriquecido: line breaking, bidi, shaping con HarfRust, fallback con Fontique, ICU4X, utilidades de edición y selección |
| `cosmic-text` | 0.19.0 (2026-04-22) | MIT OR Apache-2.0 | Alternativa a parley (multi-línea, edición) |
| `swash` | 0.2.10 (2026-07-17) | Apache-2.0 OR MIT | Introspección y shaping, alternativa |
| `allsorts` | 0.17.0 (2026-05-13) | Apache-2.0 | Parser, shaping y subsetter (de YesLogic/Prince), alternativa robusta |
| `mupdf` (mupdf-rs) | 0.8.0 (2026-06-22) | **AGPL-3.0** | **No usar** en el núcleo MIT/Apache. Solo para experimentos comparativos fuera del repositorio [I] |

Notas sobre PDFium:
- **[V] `FPDFText_SetText`.** Convierte cada carácter con `CharCodeFromUnicode` de la **fuente del objeto** y solo falla si el resultado queda vacío. No comprueba si el glifo existe. **Nosotros** debemos validar la cobertura antes de llamarla.
- **[V] `FPDFText_LoadFont`.** Incrusta el programa de fuente entero y genera la ToUnicode. `FPDF_SUBSET_NEW_FONTS` (experimental) hace subset al guardar.
- **[V] `FPDFText_SetCharcodes` y `FPDFText_SetPositions`** (experimentales) permiten fijar códigos y posiciones por carácter. `SetPositions` es útil para conservar kerning o justificación tras `SetText`. Hay que comprobar qué versión de pdfium-render las expone.
- **[V] Regeneración.** `FPDFPage_GenerateContent` regenera el contenido de la página y, sin llamarlo, se pierden los cambios. pdfium-render lo automatiza con `PdfPageContentRegenerationStrategy` (`AutomaticOnEveryChange` por defecto, `AutomaticOnDrop`, `Manual`).
- **[V] Bug abierto.** Los tests de PDFium documentan un TODO (crbug 42270554, `Bug1549`): tras eliminar un objeto y regenerar, el color resultante es incorrecto.
- **[I] Riesgo general.** La regeneración global puede alterar operadores que no tocamos. Hay que compararla visualmente siempre.
- **[V] pdfium-render 0.9.2** añadió `PdfiumCustomFontProvider`, que controla la búsqueda de fuentes del sistema y sirve para fijar las fuentes de sustitución empaquetadas.

---

## 5. Investigación académica y artículos técnicos

- **[V]** Bast & Korzen, *A Benchmark and Evaluation for Text Extraction from PDF*, JCDL 2017: 12.098 artículos de arXiv con ground truth desde TeX. Errores típicos: palabras, párrafos, orden y saltos de línea. Útil como metodología para medir nuestra agrupación en párrafos.
- **[V]** Segmentación de página: Docstrum (O'Gorman), Recursive XY-Cut (Ha, Haralick y Phillips), Voronoi (Kise), *Two Geometric Algorithms for Layout Analysis* (Breuel), y la comparación de seis algoritmos de Shafait, Keysers y Breuel. Hay implementaciones Apache-2.0 en PdfPig (C#) como referencia de diseño.
- **[V]** Hay un benchmark multi-tarea de extracción de información en PDF académicos (arXiv 2303.09957), que no se revisó en detalle.
- **[I]** No se encontró literatura académica específica sobre *editar* texto de PDF con reflow. Es un problema de ingeniería documentado sobre todo en blogs, foros y productos comerciales.

---

## 6. Recomendación por niveles

**Arquitectura común (crate núcleo, MIT/Apache) [I]:**
- **`TextModel` por página.** Chars de PDFium (Unicode, caja, origen, matriz, fuente, tamaño, color, `IsGenerated`) → líneas → bloques. Cada char se enlaza a su objeto de texto (`FPDFText_GetTextObject`) y, en el modo quirúrgico, a su **operador e índice de glifo** en el content stream.
- **`ContentLocator`.** Intérprete mínimo propio sobre `lopdf::Content` que sigue el estado gráfico y de texto (`q/Q`, `cm`, `BT/ET`, `Tf`, `Tm/Td/TD/T*`, `Tc/Tw/Tz/TL/Ts`, `Tj/TJ/'/"`, `Do` de Form XObjects) y genera glifos con posición. Se empareja con los chars de PDFium por posición (tolerancia de 0,5 pt) y por código. La alternativa es `hayro-interpret` con un `Device` propio.
- **`FontCoverage`.** Dada una fuente del PDF, dice si un `String` es codificable y dibujable: codificación o ToUnicode inverso, más existencia del GID con contorno vía `skrifa` sobre `PdfFont::data()`.
- **`EditCommand`.** Comandos con undo y redo, snapshot del stream original por página y log de recursos añadidos.

### N0 — Fallback "eliminar + superponer" (S–M)
- **Algoritmo:**
  1. Eliminar del stream los glifos del bloque. Si el `TJ` mezcla glifos dentro y fuera de la zona, se parte. Equivale a una redacción con texto conservado fuera de la zona.
  2. Insertar un objeto de texto nuevo (`PdfPageTextObject::new` con fuente de sustitución) o una FreeText aplanada.
  3. Mostrar el aviso "edición aproximada: la fuente o el diseño pueden diferir".
- **Crates:** pdfium-render (eliminar o crear objetos) o lopdf; fuentes empaquetadas.
- **Riesgos:** si solo se tapa con blanco, el texto viejo queda extraíble **[V]**. Prohibir ese modo salvo opción explícita.
- **Casos borde:** fondo no blanco (no pintar un rectángulo, eliminar de verdad), texto con `Tr 7` usado como recorte, capas OCR invisibles (`Tr 3`).

### N1 — Editar dentro de un objeto o línea con la misma fuente (M; prototipo S)
- **Precondición:** `FontCoverage` acepta todos los caracteres nuevos y el estilo no cambia.
- **Fase A (prototipo):** `set_text()` de pdfium-render sobre el objeto de texto + regeneración + `FPDFText_SetPositions` para recolocar glifos (kerning y anchos con `/Widths`). Es rápido, pero regenera la página entera.
- **Fase B (objetivo):** parche quirúrgico estilo `TokenFilter`, que solo sustituye las cadenas y números del operador afectado:
  1. Codificar el texto nuevo con la codificación de la fuente (1 byte si es simple, 2 bytes si es Identity-H).
  2. Calcular el delta de ancho Δ = Σ anchos nuevos − Σ anchos viejos.
  3. Si la línea está alineada a la izquierda, dejar que el resto de la línea se desplace si pertenece al mismo `TJ` y añadir una compensación `-Δ` antes del siguiente operador de la misma línea cuando esté posicionado de forma relativa. Si la línea es justificada, repartir Δ entre los huecos con números en `TJ`. Si Δ excede un umbral (por ejemplo, el texto invade el margen), proponer N3.
  4. Conservar `BDC/EMC` (MCID) y añadir `/ActualText` solo si hace falta.
- **Riesgos:**
  - Glifos repartidos entre varios operadores u objetos (habrá que reescribir varios).
  - Fuentes con `/Differences` sin ToUnicode.
  - Ligaduras (un glifo "fi" para dos caracteres).
  - Texto dentro de Form XObjects compartidos (clonar el XObject para esa página).
- **Pruebas:** SSIM con máscara fuera de la caja editada ≥ 0,999. Dentro de la caja, comparar con el render de referencia del texto esperado. La extracción posterior debe devolver exactamente el texto esperado. `qpdf --check` limpio.

### N2 — Sustitución de fuente con incrustación nueva (L)
- **Algoritmo:**
  1. Elegir la fuente en este orden:
     - (a) la fuente completa instalada con el mismo PostScript name sin prefijo de subset, si `fsType` permite incrustarla;
     - (b) una fuente empaquetada métricamente compatible (Liberation/Croscore/Carlito/Caladea) por nombre;
     - (c) una fuente por clase (flags del descriptor, peso, cursiva, serif) desde Noto, Liberation o DejaVu.
  2. Shaping con `harfrust`.
  3. Subset con `subsetter` o `skera`.
  4. Escribir Type0/Identity-H + CIDFontType2/CFF + `/W` + ToUnicode, con `pdf-writer` o a mano con `lopdf`. La alternativa es `load_true_type_from_bytes(.., true)` + `FPDF_SUBSET_NEW_FONTS`.
  5. Añadir la fuente a `/Resources /Font` con un nombre nuevo y emitir `Tf` para cambiar de fuente dentro de la línea, volviendo después a la original.
  6. Igualar el tamaño visual: escalar para igualar la altura x o la CapHeight, y opcionalmente usar `Tz` para igualar el ancho medio (con un límite, por ejemplo ±5 %).
- **Crates:** fontique (descubrir fuentes del sistema), skrifa (OS/2, métricas), harfrust, subsetter o skera, pdf-writer y lopdf.
- **Riesgos:**
  - Diferencia visual apreciable si no hay una fuente equivalente.
  - Licencias de fuentes del usuario.
  - PDF/A: debe incrustarse todo, y hay validadores estrictos.
  - Glifos CJK y RTL (shaping obligatorio).
- **UX:** avisar "se usará la fuente X en lugar de Y" antes de confirmar, como hacen PDF Studio y Stirling-PDF.

### N3 — Edición de párrafo con reflow (XL)
- **Algoritmo:**
  1. Bloque → runs de estilo → texto lógico (con los guiones de fin de línea detectados y marcados como blandos).
  2. Editor de texto enriquecido en la UI (las utilidades de edición de parley).
  3. Layout con parley usando el ancho del bloque, el interlineado medido y la alineación detectada.
  4. Eliminar los glifos originales del bloque (con la maquinaria de N0) y emitir una línea por `Td` + `TJ`, usando la fuente original (N1) o la sustituta (N2) por run.
  5. Conservar MCID: un `BDC` que envuelva el párrafo. Si había varios MCID, fusionarlos y actualizar el ParentTree, o no reescribir.
  6. Si el texto desborda la caja: avisar y ofrecer reducir el tamaño (como `scale_low` [V]) o ampliar la caja si debajo hay espacio libre (detección de colisión con otros bloques).
- **Riesgos:**
  - Detección de párrafos errónea (columnas, tablas, listas).
  - Partición de palabras: parley no hace guionado. Se necesitaría un crate de guionado [I, no verificado].
  - Bidi y escrituras complejas.
  - Notas o enlaces anclados a la posición antigua (anotaciones `/Link` que quedan desplazadas).
- **Pruebas:**
  - Corpus con párrafos de una y dos columnas, justificados, listas y tablas.
  - Métrica de agrupación al estilo Bast y Korzen sobre PDFs generados desde fuentes conocidas (LaTeX o Typst, donde conocemos los párrafos).
  - SSIM fuera del bloque.
  - Round-trip de extracción.
  - Revisión visual humana de una muestra.

### Plan de pruebas común [I]
- **Corpus:** tests de pdf.js y PDFBox (ya usados por hayro, más de 1.400 PDFs [V]), PDFs propios generados con LibreOffice, Word, LaTeX, Typst, Chrome e InDesign, escaneados con capa OCR, y Type3 (LaTeX antiguo con fuentes bitmap).
- **Visual:** renderizar con PDFium **y** con hayro (dos implementaciones independientes) antes y después. Calcular SSIM por región (fuera de la máscara ≈1,0, dentro según tolerancia) y el diff de píxeles.
- **Texto:** la extracción posterior (PDFium y pdfminer como oráculo externo) debe ser igual al texto esperado, y el texto de fuera de la región debe quedar intacto.
- **Estructura:** `qpdf --check`, veraPDF para PDF/A y PDF/UA si aplica, conservación del número de MCID y del árbol de estructura, y fuzzing del `ContentLocator` (cargo-fuzz).
- **Regresión:** snapshot tests de streams reescritos (insta) y tests dorados de render.

### Complejidad relativa
| Nivel | Complejidad | Dependencias principales |
|---|---|---|
| N0 eliminar + overlay | S–M | pdfium-render, lopdf |
| N1-A (PDFium SetText) | S | pdfium-render |
| N1-B (parche quirúrgico) | M | lopdf, intérprete propio o hayro-interpret, skrifa |
| N2 sustitución de fuente | L | harfrust, subsetter o skera, pdf-writer, fontique |
| N3 reflow de párrafo | XL | parley y todo lo anterior |

**Orden sugerido [I]:** N0 → N1-A (para validar UX y pruebas) → N1-B → N2 → N3.

---

## 7. Riesgos legales y de licencias

- **MuPDF, PyMuPDF y mupdf-rs son AGPL-3.0** **[V]** (crates.io para mupdf-rs; PyMuPDF/MuPDF son AGPL o comerciales [C]). No copiar código, ni portarlo línea a línea, ni enlazarlo en el núcleo MIT/Apache. Consultar la documentación para entender comportamientos está bien. Evitar que un contribuidor "traduzca" funciones de MuPDF.
- **iText (pdfSweep) es AGPL o comercial** **[V]/[C]**. Mismo criterio.
- **Poppler es GPL-2/3 [C].** Compatible con la app GPL-3.0, pero incompatible con el crate núcleo MIT/Apache. Mantenerlo fuera del núcleo.
- **PDFium es BSD-3 [C]**, y pdfium-render es MIT/Apache **[V]**. Al distribuir los binarios de PDFium hay que incluir sus avisos de licencia (incluidas las de terceros: FreeType, libjpeg, etc.) [C].
- **PDFBox y qpdf (Apache-2.0)** [C] y **PdfPig (Apache-2.0)** [C] se pueden usar como referencia e incluso portar con atribución y NOTICE.
- **Fuentes de sustitución** (todas redistribuibles y empaquetables):
  - **Liberation ≥ 2.0** bajo SIL OFL (las 1.x eran GPL con excepción) **[V]**, métricamente compatible con Arial, Arial Narrow, Times New Roman y Courier New.
  - **Croscore** (Arimo, Tinos, Cousine) bajo Apache-2.0 **[V]**.
  - **Carlito** (Calibri) bajo OFL 1.1 **[V]** y **Caladea** (Cambria) **[V]** por métricas. Verificar la licencia de Caladea en su repositorio [C: OFL].
  - **Noto** bajo OFL [C], para cobertura amplia de escrituras.
  - **DejaVu** (licencia Bitstream Vera, libre) [C].
  - La OFL permite empaquetarlas con software y incrustarlas en documentos. No permite venderlas solas, y las versiones modificadas no pueden usar el "Reserved Font Name" [C].
- **Fuentes del usuario (sistema).** Respetar `fsType` de OS/2 (restringida, *preview & print*, editable, instalable) al incrustar. Si es restringida, no incrustarla y usar N2(b o c) con aviso. Ver el caso de Market Sans en Adobe Community **[V]**.
- **Firma digital.** Editar invalida la firma de la revisión. Avisar y usar guardado incremental para no destruir la revisión firmada.
- **Privacidad.** Si el modo overlay no elimina el original, se filtra el texto (PyMuPDF/Artifex lo advierte **[V]**). Por defecto, eliminar siempre.

---

## Tabla comparativa de herramientas y bibliotecas

| Nombre | Licencia | Qué hace respecto a editar texto existente | ¿Reutilizable en nuestro stack? |
|---|---|---|---|
| PDFium (+ pdfium-render) | BSD-3 [C] / MIT-Apache [V] | Chars con geometría, char → objeto, `SetText`, `SetCharcodes`, `SetPositions`, cargar fuentes, regenerar contenido, guardado incremental, subset de fuentes nuevas | **Sí** (motor base) |
| lopdf | MIT [V] | Parseo y codificación de content streams, cambio de contenido, actualización incremental | **Sí** (parche quirúrgico) |
| hayro (syntax/interpret/write) | Apache-2.0/MIT [V] | Intérprete con `Device`, reescritura de páginas, segundo renderizador | **Sí** (localizador y pruebas), experimental |
| krilla / pdf-writer | MIT/Apache [V] | Generación de PDF y escritura de diccionarios de fuentes | **Sí** (pdf-writer para objetos de fuente) |
| subsetter / skera | MIT/Apache [V] | Subset de fuentes | **Sí** |
| harfrust / parley / fontique / skrifa | MIT / Apache-MIT [V] | Shaping, layout, fallback y métricas | **Sí** |
| Adobe Acrobat Pro | Propietaria | Edición de párrafo con reflow y sustitución de fuentes | No (solo referencia de UX) |
| Foxit PDF Editor | Propietaria | Edición por párrafo, autoincrustación de fuentes (v12+) | No (referencia de UX) |
| PDF-XChange Editor | Propietaria | Edición de contenido, validación de fuentes | No |
| Master PDF Editor | Propietaria | Edición por objeto, cambio automático de fuente si está instalada | No (referencia de UX) |
| Qoppa PDF Studio | Propietaria | Aviso de subset antes de perder glifos | No (referencia de UX) |
| Stirling-PDF 2.x | Open-core (MIT + partes propietarias [C]) | Editor alfa: PDF ↔ JSON, reescritura de tokens o regeneración, fallback de fuentes Liberation/DejaVu/Noto | Ideas sí. Código no (Java, y verificar licencia por carpeta) |
| LibreOffice Draw | MPL-2.0 [C] | Importa cada línea como un cuadro, sin párrafos | No (contraejemplo) |
| Inkscape | GPL [C] | Importación por página con texto editable y sustitución de fuentes | No |
| pdf.js / Firefox | Apache-2.0 [C] | Solo anotaciones (FreeText y otras), no edita el contenido | Ideas de UX de FreeText |
| MuPDF / PyMuPDF | **AGPL-3.0** o comercial | stext, redacción real, `insert_htmlbox` | **No** (solo documentación como referencia) |
| iText pdfSweep | **AGPL** o comercial | Eliminar contenido por regiones | **No** |
| Apache PDFBox | Apache-2.0 [C] | Parseo y escritura, sin edición de alto nivel | Ideas y port con atribución |
| qpdf / pikepdf | Apache-2.0 / MPL-2.0 [C] | `TokenFilter` para reescritura token a token | Patrón de diseño; qpdf para validar (`--check`) |
| pdfminer.six | MIT [C] | Análisis de layout (LAParams) | Algoritmo (reimplementar) y oráculo de tests |
| PdfPig | Apache-2.0 [C] | Docstrum, XY-Cut y whitespace cover | Algoritmos (port con atribución) |
| Poppler / Okular | GPL [C] | Renderizado y anotaciones | No en el núcleo |
| Apryse / PDFTron, PDFDancer | Comercial | Reemplazo de run con fuente nueva; "edición sin overlays" (afirmación de marketing) | No |

---

## Fuentes

- [fpdf_edit.h (PDFium, main)](https://pdfium.googlesource.com/pdfium/+/refs/heads/main/public/fpdf_edit.h)
- [fpdf_edittext.cpp (PDFium, main)](https://pdfium.googlesource.com/pdfium/+/main/fpdfsdk/fpdf_edittext.cpp)
- [fpdf_edit_embeddertest.cpp (PDFium, main)](https://pdfium.googlesource.com/pdfium/+/main/fpdfsdk/fpdf_edit_embeddertest.cpp)
- [fpdf_text.h (PDFium, main)](https://pdfium.googlesource.com/pdfium/+/refs/heads/main/public/fpdf_text.h)
- [fpdf_save.h (PDFium, main)](https://pdfium.googlesource.com/pdfium/+/refs/heads/main/public/fpdf_save.h)
- [pdfium-render (GitHub)](https://github.com/ajrcarey/pdfium-render)
- [PdfPageTextObject — docs.rs pdfium-render](https://docs.rs/pdfium-render/latest/pdfium_render/prelude/struct.PdfPageTextObject.html)
- [PdfPageContentRegenerationStrategy — docs.rs](https://docs.rs/pdfium-render/latest/pdfium_render/prelude/enum.PdfPageContentRegenerationStrategy.html)
- [PdfFont — docs.rs](https://docs.rs/pdfium-render/latest/pdfium_render/prelude/struct.PdfFont.html)
- [PdfFonts — docs.rs](https://docs.rs/pdfium-render/latest/pdfium_render/prelude/struct.PdfFonts.html)
- [lopdf Document — docs.rs](https://docs.rs/lopdf/latest/lopdf/struct.Document.html)
- [lopdf Content — docs.rs](https://docs.rs/lopdf/latest/lopdf/content/struct.Content.html)
- [crates.io API (versiones y licencias consultadas el 15-11-2026)](https://crates.io/)
- [krilla (GitHub)](https://github.com/LaurenzV/krilla)
- [hayro (GitHub)](https://github.com/LaurenzV/hayro)
- [subsetter (GitHub)](https://github.com/typst/subsetter)
- [HarfRust (GitHub)](https://github.com/harfbuzz/harfrust)
- [Parley (GitHub)](https://github.com/linebender/parley)
- [How to Search and Replace Text in PDFs Using PyMuPDF (Artifex, 2025)](https://artifex.com/blog/how-to-search-and-replace-text-in-pdfs-using-pymupdf)
- [PyMuPDF Page API (insert_htmlbox, apply_redactions)](https://pymupdf.readthedocs.io/en/latest/page.html)
- [PyMuPDF TextPage](https://pymupdf.readthedocs.io/en/latest/textpage.html)
- [MuPDF Change Log 1.27.1](https://mupdf.readthedocs.io/en/1.27.1/other/changes.html)
- [pdfminer.six — Converting a PDF file to text (layout analysis)](https://pdfminersix.readthedocs.io/en/latest/topic/converting_pdf_to_text.html)
- [PDF text and fonts, design by devil (nibblestew, 2023)](https://nibblestew.blogspot.com/2023/01/pdf-text-and-fonts-design-by-devil.html)
- [PDF Font Hell: why swapping fonts in PDF breaks everything (DEV)](https://dev.to/michael_lahr/pdf-font-hell-why-swapping-fonts-in-pdf-breaks-everything-1n16)
- [Why do I get weird results after editing existing PDF text? (Apryse community)](https://community.apryse.com/t/why-do-i-get-weird-results-after-editing-existing-pdf-text/451)
- [Apache PDFBox: how to replace text data by keys (Stack Overflow, archivo)](https://kaweah.freedombox.rocks/kiwix/content/stackoverflow.com_en_all_2023-11/questions/60618730/apache-pdfbox-how-to-replace-text-data-by-keys)
- [Stirling-PDF — pdf_text_edit_flow.md](https://git.laurivan.com/Mirrors/Stirling-PDF/src/branch/demo/docs/pdf_text_edit_flow.md)
- [Stirling-PDF — pdf-text-editor-backlog.md](https://git.laurivan.com/Mirrors/Stirling-PDF/raw/branch/demo/docs/pdf-text-editor-backlog.md)
- [Stirling PDF 2.0 brings text editing… (OpenSourceForU, dic. 2025)](https://www.opensourceforu.com/2025/12/stirling-pdf-2-0-brings-text-editing-and-enterprise-tools-to-open-source/)
- [Stirling-PDF (GitHub)](https://github.com/Stirling-Tools/Stirling-PDF)
- [LibreOffice Bug 125838](https://bugs.documentfoundation.org/show_bug.cgi?id=125838)
- [Inkscape forum — simple PDF text edit](https://inkscape.org/forums/questions/simple-pdf-text-edit)
- [Firefox 150 PDF Editor Explained (Techwalla)](https://www.techwalla.com/13779414/firefox-150-pdf-editor-explained-what-it-does-and-doesnt-do)
- [Adobe Acrobat (web) — Edit text](https://helpx.adobe.com/acrobat/web/edit-pdfs/edit-text-images-and-objects/edit-text.html)
- [Adobe Community — I can't change the font (Acrobat Pro 2020), 2025](https://community.adobe.com/t5/acrobat-discussions/i-can-t-change-the-font-adobe-acrobat-pro-2020/m-p/15270479)
- [Foxit PDF Editor — Paragraph Editing](https://resource-cms.foxit.com/foxit-pdf-editor-for-windows/paragraph-editing)
- [Foxit blog — Control how fonts are handled in your PDFs (2023)](https://www.foxit.com/blog/control-how-fonts-are-handled-in-your-pdfs/)
- [Master PDF Editor — Possible issues with text](https://code-industry.net/masterpdfeditor-help/possible-issues-with-text)
- [Qoppa PDF Studio — Text content editing with subsetted fonts](https://kbpdfstudio.qoppa.com/text-content-editing-with-subsetted-fonts/)
- [PDF-XChange forum — missing glyph embedding](https://forum.pdf-xchange.com/viewtopic.php?p=169692)
- [iText — Removing content with pdfSweep](https://kb.itextpdf.com/itext/removing-content-with-pdfsweep)
- [pikepdf — Content streams / TokenFilter](https://pikepdf.readthedocs.io/en/stable/api/filters.html)
- [PDFDancer — How to edit text inside any PDF without overlays](https://pdfdancer-www.pages.dev/how-to/edit-text-without-overlays)
- [Bast & Korzen — A Benchmark and Evaluation for Text Extraction from PDF (JCDL 2017)](https://ad-publications.informatik.uni-freiburg.de/benchmark.pdf)
- [A Benchmark of PDF Information Extraction Tools… (arXiv 2303.09957)](https://arxiv.org/pdf/2303.09957)
- [BobLd/DocumentLayoutAnalysis (Docstrum, XY-Cut, Voronoi, whitespace cover)](https://github.com/BobLd/DocumentLayoutAnalysis)
- [Liberation fonts (Wikipedia)](https://en.wikipedia.org/wiki/Liberation_fonts)
- [Croscore fonts (Wikipedia)](https://ipfs.ssi.eecc.de/ipns/en.wikipedia-on-ipfs.org/wiki/Croscore_fonts)
- [Carlito — Font Squirrel (SIL OFL 1.1)](https://www.fontsquirrel.com/fonts/carlito)
- [PDF Association — No-cost access to ISO 32000-2 (PDF 2.0)](https://pdfa.org/announcing-no-cost-access-to-iso-32000-2-pdf-2-0/)
