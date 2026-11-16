# Spec de producto — Fase 1: Descubrimiento de requisitos (BORRADOR)

> Proyecto: visor/editor PDF open source, multiplataforma (Windows, macOS, Linux), escrito en Rust.
> Nombre provisional: `pdf-tool`.
> Estado: **borrador para discusión**. Cuando se apruebe ("Proceder"), se generará `requirements.md` con notación EARS.

---

## 0. Visión y principios

**Visión:** una herramienta de escritorio para leer, revisar, editar, organizar, firmar y proteger PDFs, sin paywalls, sin cuenta, sin nube obligatoria y sin funciones de IA impuestas.

**Principios del producto (no negociables):**

| # | Principio | Qué significa en la práctica |
|---|-----------|------------------------------|
| P-1 | **Todo gratis, todo local** | Ninguna función se bloquea tras pago. Los documentos nunca salen del equipo salvo acción explícita del usuario. |
| P-2 | **Sin telemetría por defecto** | Cero datos de uso enviados. Si algún día existe, será opt-in y documentada. |
| P-3 | **No corromper nunca un documento** | Guardado atómico, copia de respaldo, deshacer/rehacer y opción de guardado incremental. |
| P-4 | **Rápido y ligero** | Arranque y apertura en tiempos cortos aun con documentos grandes. |
| P-5 | **Sin IA impuesta** | Nada de asistentes, resúmenes ni "chatea con tu PDF" integrados. Si alguien los quiere, que sea vía plugin externo. |
| P-6 | **Respeto por los estándares** | Lo que se produce debe abrirse bien en Acrobat, navegadores, Preview, etc. |
| P-7 | **Automatizable** | Todo lo que se pueda hacer en la UI con un documento debería poder hacerse por CLI sobre muchos. |

**Usuarios tipo (personas):**

- **Ana, profesional de oficina:** recibe contratos y formularios, necesita llenarlos, firmarlos y devolverlos.
- **Luis, estudiante / investigador:** lee papers y libros largos, subraya, toma notas, busca y exporta citas.
- **Marta, administrativa / contable:** une facturas, divide estados de cuenta, comprime para enviar por correo, escanea documentos.
- **Diego, abogado / RR. HH.:** necesita tachar datos sensibles de verdad (redacción), proteger con contraseña, comparar versiones y verificar firmas.
- **Sofía, desarrolladora / power user:** procesa cientos de PDFs con scripts, quiere CLI y atajos de teclado.

---

## 1. Visualización y lectura

**Problemas comunes:**
- Los lectores "gratis" son lentos con documentos de cientos de páginas o planos grandes.
- Fuentes que no se ven, documentos escaneados borrosos, colores raros.
- Leer de noche cansa la vista; no hay modo oscuro real del *contenido*.
- Perder el lugar en el que ibas leyendo al cerrar el archivo.

| ID | Caso de uso | Prioridad sugerida |
|----|-------------|--------------------|
| UC-VIS-01 | Abrir un PDF desde el menú, arrastrando el archivo, desde la línea de comandos o con doble clic (asociación de archivos). | P0 |
| UC-VIS-02 | Ver páginas con zoom (ajustar a ancho, ajustar a página, tamaño real, porcentaje libre, zoom con Ctrl+rueda / pellizco en trackpad). | P0 |
| UC-VIS-03 | Modos de vista: una página, continuo, dos páginas (libro, con portada separada o no), desplazamiento horizontal. | P0 |
| UC-VIS-04 | Rotar la vista de una página o de todo el documento (sin modificar el archivo). | P0 |
| UC-VIS-05 | Abrir varios documentos en pestañas y en ventanas separadas; vista dividida (dos documentos o dos partes del mismo documento lado a lado). | P1 |
| UC-VIS-06 | Recordar la última posición, el zoom y el modo de vista de cada documento al reabrirlo. | P0 |
| UC-VIS-07 | Modo lectura / pantalla completa sin distracciones; modo presentación (página a página, con flechas, clic y puntero láser). | P1 |
| UC-VIS-08 | Modo oscuro de la interfaz **y** modos de color del contenido: invertido inteligente (respetando imágenes), sepia, alto contraste, colores personalizados. | P1 |
| UC-VIS-09 | Abrir PDFs protegidos con contraseña de apertura. | P0 |
| UC-VIS-10 | Abrir PDFs dañados o mal formados, intentando repararlos y avisando al usuario, en lugar de fallar. | P0 |
| UC-VIS-11 | Ver capas opcionales (OCG), mostrar u ocultar capas (común en planos y mapas). | P2 |
| UC-VIS-12 | Ver adjuntos incrustados en el PDF y extraerlos. | P1 |
| UC-VIS-13 | Ver propiedades del documento: título, autor, productor, versión PDF, tamaño de página, fuentes usadas, seguridad, si está firmado o es PDF/A. | P1 |
| UC-VIS-14 | Reproducir contenido multimedia incrustado (video/audio) o al menos informar que existe y permitir extraerlo. | P2 |
| UC-VIS-15 | Herramienta lupa / zoom a una región seleccionada; herramienta de mano para desplazar. | P1 |
| UC-VIS-16 | Herramientas de medición (distancia, perímetro, área) con escala configurable, para planos. | P2 |
| UC-VIS-17 | Recargar automáticamente el documento si cambia en disco (útil para quien genera PDFs con LaTeX/Typst). | P1 |
| UC-VIS-18 | Abrir otros formatos de documento comunes en solo lectura: imágenes, EPUB, XPS/CBZ (opcional). | P2 |

---

## 2. Navegación y búsqueda

**Problemas comunes:**
- Documentos largos sin forma rápida de saltar a una sección.
- La búsqueda falla con acentos, ligaduras ("fi", "fl") o palabras cortadas por guion.
- No se puede buscar en varios PDFs a la vez.

| ID | Caso de uso | Prioridad |
|----|-------------|-----------|
| UC-NAV-01 | Panel de miniaturas de páginas con tamaño ajustable. | P0 |
| UC-NAV-02 | Panel de marcadores/índice (outline) del PDF, navegable y plegable. | P0 |
| UC-NAV-03 | Ir a página por número o por etiqueta de página (p. ej. "iv", "A-3"). | P0 |
| UC-NAV-04 | Historial de navegación atrás/adelante (como un navegador) tras seguir enlaces internos. | P0 |
| UC-NAV-05 | Seguir enlaces internos y externos; los externos piden confirmación antes de abrir el navegador. | P0 |
| UC-NAV-06 | Buscar texto con resaltado de todas las coincidencias, contador "3 de 27", siguiente/anterior. | P0 |
| UC-NAV-07 | Opciones de búsqueda: mayúsculas/minúsculas, palabra completa, **insensible a acentos**, expresiones regulares. | P1 |
| UC-NAV-08 | Panel de resultados con contexto (fragmento de texto alrededor de cada coincidencia). | P1 |
| UC-NAV-09 | Buscar en todos los PDFs de una carpeta (búsqueda multi-documento). | P2 |
| UC-NAV-10 | Crear, editar y reordenar marcadores propios (y guardarlos en el PDF). | P1 |
| UC-NAV-11 | Marcadores personales del lector (no se guardan en el PDF), tipo "punto de lectura". | P2 |
| UC-NAV-12 | Lista de documentos recientes con miniatura, buscable, con opción de borrar historial. | P0 |

---

## 3. Selección, copia y extracción

**Problemas comunes:**
- Al copiar texto salen saltos de línea en cada renglón, columnas mezcladas o caracteres basura.
- No se pueden copiar tablas.
- Extraer imágenes de un PDF requiere herramientas extra.

| ID | Caso de uso | Prioridad |
|----|-------------|-----------|
| UC-EXT-01 | Seleccionar y copiar texto respetando el orden de lectura y uniendo líneas de un mismo párrafo. | P0 |
| UC-EXT-02 | Selección rectangular / por columnas (Alt+arrastrar). | P1 |
| UC-EXT-03 | Copiar una región como imagen (captura de área) al portapapeles. | P1 |
| UC-EXT-04 | Extraer todas las imágenes de un documento o de páginas seleccionadas en su resolución original. | P1 |
| UC-EXT-05 | Exportar texto completo a TXT / Markdown. | P1 |
| UC-EXT-06 | Detectar y exportar tablas a CSV. | P2 |
| UC-EXT-07 | Exportar páginas como imágenes (PNG/JPEG/WebP/TIFF) a DPI elegido. | P1 |
| UC-EXT-08 | Exportar anotaciones/notas a texto o Markdown (para estudiantes e investigadores). | P1 |

---

## 4. Anotación y revisión

**Problemas comunes:**
- En lectores gratuitos, las anotaciones vienen limitadas o se guardan en formatos propios que otros programas no ven.
- No se pueden revisar documentos en equipo (comentarios, respuestas, estados).
- Las anotaciones se pierden si se "aplana" mal.

| ID | Caso de uso | Prioridad |
|----|-------------|-----------|
| UC-ANO-01 | Resaltar, subrayar, tachar y subrayado ondulado sobre texto, con colores configurables. | P0 |
| UC-ANO-02 | Notas adhesivas (sticky notes) con texto, autor y fecha. | P0 |
| UC-ANO-03 | Cuadros de texto libres (FreeText) con fuente, tamaño, color, borde y fondo. | P0 |
| UC-ANO-04 | Dibujo a mano alzada (lápiz/tinta) con grosor y color; soporte de lápiz con presión (tableta). | P1 |
| UC-ANO-05 | Formas: rectángulo, elipse, línea, flecha, polilínea, polígono, nube de revisión. | P1 |
| UC-ANO-06 | Sellos: predefinidos ("Aprobado", "Borrador", "Confidencial", "Recibido" con fecha) y personalizados a partir de imagen. | P1 |
| UC-ANO-07 | Llamadas (callouts) con flecha que apunta a una zona. | P2 |
| UC-ANO-08 | Panel de comentarios: lista de todas las anotaciones, filtrar por tipo, autor, color, página; saltar a cada una. | P0 |
| UC-ANO-09 | Responder a comentarios (hilos) y marcar estado (aceptado, rechazado, resuelto). | P1 |
| UC-ANO-10 | Editar, mover, redimensionar, cambiar color y borrar cualquier anotación; selección múltiple. | P0 |
| UC-ANO-11 | Guardar anotaciones como objetos PDF estándar, legibles en Acrobat, navegadores, Preview, etc. | P0 |
| UC-ANO-12 | Aplanar anotaciones (convertirlas en contenido fijo de la página) de forma opcional. | P1 |
| UC-ANO-13 | Importar/exportar anotaciones en XFDF/FDF (intercambio con otros lectores). | P2 |
| UC-ANO-14 | Nombre de autor configurable para las anotaciones (por defecto sin datos personales). | P0 |
| UC-ANO-15 | Herramientas favoritas / paleta rápida con atajos de teclado (p. ej. `H` para resaltar). | P1 |

---

## 5. Edición del contenido

**Problemas comunes:**
- Corregir una errata en un PDF es "función premium" en casi todos lados.
- Agregar un logo, una imagen o un texto nuevo requiere convertir a Word y perder formato.
- Las marcas de agua y la numeración de páginas son de pago.

| ID | Caso de uso | Prioridad |
|----|-------------|-----------|
| UC-EDI-01 | Agregar texto nuevo a una página como contenido (no anotación), con fuente, tamaño y color. | P1 |
| UC-EDI-02 | Editar texto existente en línea (corregir una palabra o frase), conservando fuente si está disponible; avisar si la fuente está incompleta (subset) y ofrecer sustituta. | P2 (difícil) |
| UC-EDI-03 | Insertar, mover, redimensionar, recortar, reemplazar y borrar imágenes. | P1 |
| UC-EDI-04 | Mover/borrar objetos de la página (texto, imagen, trazos) mediante selección de objetos. | P2 |
| UC-EDI-05 | Agregar marcas de agua de texto o imagen (opacidad, rotación, posición, rango de páginas). | P1 |
| UC-EDI-06 | Agregar encabezados, pies de página y numeración de páginas (formatos: "1", "Página 1 de 10", romanos), numeración Bates para legal. | P1 |
| UC-EDI-07 | Agregar o editar enlaces e hipervínculos. | P2 |
| UC-EDI-08 | Editar metadatos del documento (título, autor, asunto, palabras clave). | P1 |
| UC-EDI-09 | Cambiar fondo de páginas (color o imagen). | P2 |
| UC-EDI-10 | Deshacer/rehacer ilimitado durante la sesión para todas las ediciones. | P0 |

---

## 6. Organización de páginas

**Problemas comunes:**
- Unir varios PDFs o sacar unas cuantas páginas obliga a subir archivos privados a sitios web desconocidos.
- Escaneos con páginas al revés o en desorden.
- Archivos demasiado pesados para enviar por correo.

| ID | Caso de uso | Prioridad |
|----|-------------|-----------|
| UC-PAG-01 | Vista de organizador: cuadrícula de miniaturas grandes con arrastrar y soltar para reordenar. | P0 |
| UC-PAG-02 | Rotar páginas (90°, 180°) de forma permanente, una, varias o todas. | P0 |
| UC-PAG-03 | Eliminar páginas, duplicarlas e insertar páginas en blanco. | P0 |
| UC-PAG-04 | Unir varios PDFs en uno (arrastrando archivos a la vista de organizador o eligiendo una lista ordenada), conservando marcadores. | P0 |
| UC-PAG-05 | Insertar páginas de otro PDF o de imágenes en una posición. | P0 |
| UC-PAG-06 | Dividir un PDF: por rangos ("1-3, 5, 8-"), cada N páginas, por marcadores, por tamaño máximo de archivo, o en páginas sueltas. | P0 |
| UC-PAG-07 | Extraer páginas seleccionadas a un nuevo PDF. | P0 |
| UC-PAG-08 | Recortar páginas (crop box), quitar márgenes en blanco automáticamente. | P1 |
| UC-PAG-09 | Cambiar tamaño de página / escalar contenido (p. ej. A4 ↔ Carta). | P2 |
| UC-PAG-10 | Intercalar páginas de dos PDFs (escaneos a doble cara: impares en un archivo, pares en otro, en orden inverso). | P1 |
| UC-PAG-11 | Imposición: N páginas por hoja (2-up, 4-up), folleto (booklet). | P2 |
| UC-PAG-12 | Editar etiquetas de página (numeración lógica). | P2 |
| UC-PAG-13 | Comprimir / optimizar: reducir resolución de imágenes, recomprimir, quitar objetos sin uso, con perfiles (alta calidad, correo, mínima) y vista previa del tamaño resultante. | P1 |

---

## 7. Formularios

**Problemas comunes:**
- Formularios oficiales que no se pueden llenar o que, una vez llenos, se ven vacíos en otro programa.
- PDFs "planos" (escaneados) donde no hay campos y hay que escribir encima.
- Crear un formulario rellenable es función de pago.

| ID | Caso de uso | Prioridad |
|----|-------------|-----------|
| UC-FOR-01 | Llenar formularios AcroForm: texto, casillas, botones de opción, listas, combos, fechas. | P0 |
| UC-FOR-02 | Navegar entre campos con Tab / Shift+Tab; resaltar campos rellenables. | P0 |
| UC-FOR-03 | Validaciones y formatos básicos de campos (número, fecha, máscara) declarados en el PDF, sin ejecutar JavaScript arbitrario. | P1 |
| UC-FOR-04 | Guardar con apariencias generadas para que el formulario se vea igual en cualquier lector. | P0 |
| UC-FOR-05 | "Llenar y firmar" en PDFs sin campos: colocar texto, marcas ✓ / ✗, fecha y firma en cualquier lugar. | P0 |
| UC-FOR-06 | Aplanar un formulario lleno (convertirlo en no editable) para entregarlo. | P1 |
| UC-FOR-07 | Crear y editar formularios: agregar campos de cada tipo, nombre, valor por defecto, obligatorio, orden de tabulación. | P2 |
| UC-FOR-08 | Detección automática de campos en un PDF plano (líneas y casillas) para convertirlo en formulario. | P2 |
| UC-FOR-09 | Importar/exportar datos de formulario (FDF/XFDF/JSON/CSV) y llenado masivo desde CSV (una copia por fila). | P2 |
| UC-FOR-10 | Informar claramente cuando un formulario es XFA (formato heredado de Adobe) y no puede llenarse completamente. | P1 |
| UC-FOR-11 | Guardar perfiles de autocompletado locales (nombre, dirección, ID) para formularios recurrentes. | P2 |

---

## 8. Firmas

Es importante distinguir tres cosas distintas que la gente llama "firmar":

1. **Firma visual / electrónica simple:** una imagen o trazo de tu firma colocada sobre la página. No tiene validez criptográfica.
2. **Firma digital (criptográfica):** usa un certificado (archivo .p12/.pfx, token USB, tarjeta inteligente o DNI electrónico) y detecta cualquier modificación posterior. Estándar PAdES.
3. **Verificación:** comprobar quién firmó, si el documento cambió después y si el certificado es de confianza.

**Problemas comunes:**
- Firmar con certificado o validar una firma suele ser de pago o solo funciona en Acrobat.
- Firmar "pegando una imagen" y luego no saber si eso vale legalmente.
- Una firma visual que se puede copiar y pegar en otro documento.

| ID | Caso de uso | Prioridad |
|----|-------------|-----------|
| UC-FIR-01 | Crear una firma visual: dibujándola (mouse/trackpad/lápiz), escribiéndola con tipografía manuscrita, o importando una imagen (con eliminación de fondo blanco). | P0 |
| UC-FIR-02 | Guardar varias firmas e iniciales localmente (cifradas o protegidas) para reutilizarlas. | P0 |
| UC-FIR-03 | Colocar, redimensionar y fijar la firma visual; insertar fecha automática junto a ella; firmar iniciales en todas las páginas. | P0 |
| UC-FIR-04 | Firmar digitalmente con un certificado en archivo (.p12/.pfx) protegido con contraseña. | P1 |
| UC-FIR-05 | Firmar digitalmente usando el almacén de certificados del sistema (Windows Certificate Store, Llavero de macOS) y tokens/tarjetas vía PKCS#11 (incluye DNI/cédulas electrónicas). | P2 |
| UC-FIR-06 | Firma con apariencia visible (nombre, fecha, motivo, ubicación, imagen) o invisible. | P1 |
| UC-FIR-07 | Niveles PAdES: B-B (básica), B-T (con sello de tiempo de un servidor TSA), B-LT/B-LTA (validación a largo plazo). | P1 (B-B, B-T) / P2 (LT, LTA) |
| UC-FIR-08 | Validar firmas al abrir el documento: panel con firmante, fecha, integridad (¿se modificó después?), estado del certificado y cadena de confianza; mensajes claros (válida / con advertencias / inválida). | P1 |
| UC-FIR-09 | Varias firmas en un mismo documento (flujo de firmas sucesivas) sin invalidar las anteriores (guardado incremental). | P1 |
| UC-FIR-10 | Certificar un documento (firma de autor que define qué cambios se permiten después). | P2 |
| UC-FIR-11 | Crear un certificado autofirmado para uso personal (con advertencia de que no tiene validez legal ante terceros). | P2 |
| UC-FIR-12 | Avisar si el usuario intenta editar un documento ya firmado y explicar que eso invalidará o afectará la firma. | P1 |

---

## 9. Seguridad y privacidad

**Problemas comunes:**
- "Tachar" con un rectángulo negro que en realidad deja el texto debajo (filtraciones famosas).
- PDFs con metadatos ocultos (autor, rutas, software, historial).
- PDFs maliciosos con JavaScript o lanzadores de archivos.

| ID | Caso de uso | Prioridad |
|----|-------------|-----------|
| UC-SEG-01 | **Redacción real:** marcar texto o áreas y eliminar de verdad el contenido subyacente (texto, imágenes, vectores), no solo taparlo. | P1 |
| UC-SEG-02 | Redacción por búsqueda: buscar patrones (texto, regex, correos, teléfonos, números de identificación) y redactar todas las coincidencias tras revisarlas. | P1 |
| UC-SEG-03 | Verificar tras redactar que el texto eliminado ya no puede extraerse. | P1 |
| UC-SEG-04 | Proteger con contraseña de apertura (AES-256) y contraseña de permisos (impresión, copia, edición). | P1 |
| UC-SEG-05 | Quitar la contraseña de un PDF del que se conoce la contraseña. | P1 |
| UC-SEG-06 | "Limpiar documento": eliminar metadatos, XMP, adjuntos ocultos, JavaScript, capas ocultas, comentarios, miniaturas incrustadas, datos de formularios. | P1 |
| UC-SEG-07 | No ejecutar JavaScript de PDF por defecto; bloquear acciones de "lanzar archivo" y avisar de enlaces a URLs externas. | P0 |
| UC-SEG-08 | Abrir documentos no confiables de forma segura: el analizador/renderizador no debe poder comprometer el sistema (aislamiento de procesos si es viable). | P1 |
| UC-SEG-09 | Indicar visualmente cuando un documento contiene JavaScript, adjuntos, formularios o enlaces externos. | P1 |

---

## 10. Conversión, creación y exportación

**Problemas comunes:**
- Convertir imágenes o escaneos a PDF exige apps con anuncios o marcas de agua.
- Los organismos piden PDF/A y nadie sabe generarlo.

| ID | Caso de uso | Prioridad |
|----|-------------|-----------|
| UC-CON-01 | Crear PDF a partir de imágenes (JPG, PNG, TIFF multipágina, HEIC si es viable), con tamaño de página, márgenes y orientación. | P0 |
| UC-CON-02 | Crear PDF en blanco / desde portapapeles (imagen pegada). | P1 |
| UC-CON-03 | Exportar PDF a imágenes (ver UC-EXT-07). | P1 |
| UC-CON-04 | Convertir a PDF/A (archivo a largo plazo) y validar conformidad básica. | P2 |
| UC-CON-05 | Convertir a escala de grises / blanco y negro. | P2 |
| UC-CON-06 | Convertir documentos de oficina (DOCX, ODT, XLSX) a PDF usando LibreOffice si está instalado (integración opcional, no dependencia). | P2 |
| UC-CON-07 | Exportar a formatos de texto (TXT, Markdown, HTML simple). | P1 |

---

## 11. Escaneo y OCR

**Problemas comunes:**
- PDFs escaneados en los que no se puede buscar ni copiar texto.
- Escaneos torcidos, con bordes negros o fondos grises.

| ID | Caso de uso | Prioridad |
|----|-------------|-----------|
| UC-OCR-01 | Reconocer texto (OCR) en páginas escaneadas y agregar una capa de texto invisible para poder buscar y copiar. | P1 |
| UC-OCR-02 | Elegir idiomas de OCR (español e inglés incluidos; otros descargables), todo procesado localmente. | P1 |
| UC-OCR-03 | Detectar automáticamente qué páginas necesitan OCR (no tienen texto). | P1 |
| UC-OCR-04 | Limpieza de escaneo: enderezar (deskew), quitar bordes negros, ajustar contraste, eliminar páginas en blanco. | P2 |
| UC-OCR-05 | Escanear directamente desde un escáner del sistema (WIA/TWAIN, ImageCaptureCore, SANE). | P2 |

---

## 12. Comparación de documentos

| ID | Caso de uso | Prioridad |
|----|-------------|-----------|
| UC-CMP-01 | Comparar dos versiones de un PDF por texto: mostrar agregados, eliminados y cambios, lado a lado y sincronizados. | P2 |
| UC-CMP-02 | Comparación visual (diferencia de píxeles) para planos o diseños. | P2 |
| UC-CMP-03 | Exportar un informe de diferencias. | P2 |

---

## 13. Impresión

| ID | Caso de uso | Prioridad |
|----|-------------|-----------|
| UC-IMP-01 | Imprimir con el diálogo del sistema: rango de páginas, copias, doble cara, escalado (ajustar, tamaño real, personalizado). | P0 |
| UC-IMP-02 | Imprimir con o sin anotaciones / solo campos de formulario. | P1 |
| UC-IMP-03 | Imprimir varias páginas por hoja y folletos. | P2 |
| UC-IMP-04 | Vista previa de impresión fiel. | P1 |

---

## 14. Procesamiento por lotes y automatización (CLI)

**Problemas comunes:**
- Hacer lo mismo a 200 PDFs uno por uno.
- Herramientas de línea de comandos dispersas (qpdf, pdftk, ghostscript) con sintaxis distinta.

| ID | Caso de uso | Prioridad |
|----|-------------|-----------|
| UC-LOT-01 | CLI con subcomandos: `merge`, `split`, `rotate`, `extract`, `compress`, `encrypt`, `decrypt`, `watermark`, `number`, `ocr`, `sign`, `verify`, `info`, `render`, `text`, `sanitize`, `redact`. | P1 |
| UC-LOT-02 | Acciones por lotes desde la UI: elegir archivos/carpeta, una o varias acciones encadenadas, destino y patrón de nombre de salida. | P2 |
| UC-LOT-03 | Guardar secuencias de acciones como "recetas" reutilizables (UI y CLI comparten el mismo formato). | P2 |
| UC-LOT-04 | Salida legible por máquinas (JSON) en la CLI para integrarla en scripts. | P1 |

---

## 15. Integración con el sistema y experiencia de uso

| ID | Caso de uso | Prioridad |
|----|-------------|-----------|
| UC-SIS-01 | Instalar en Windows (MSI/winget), macOS (DMG firmado, Apple Silicon e Intel) y Linux (AppImage, .deb, .rpm, Flatpak). | P0 |
| UC-SIS-02 | Ofrecer ser el lector PDF predeterminado; abrir archivos desde el explorador. | P0 |
| UC-SIS-03 | Arrastrar y soltar archivos en la ventana, y páginas entre ventanas. | P1 |
| UC-SIS-04 | Atajos de teclado completos y configurables (incluido un modo con atajos estilo Vim para power users). | P1 |
| UC-SIS-05 | Paleta de comandos (Ctrl/Cmd+K) para buscar cualquier acción por nombre. | P1 |
| UC-SIS-06 | Interfaz en español e inglés desde el inicio, preparada para más idiomas. | P0 |
| UC-SIS-07 | Actualizaciones: aviso de nueva versión (opt-in, sin enviar datos), o delegar en el gestor de paquetes del sistema. | P1 |
| UC-SIS-08 | Funcionar 100% sin conexión. | P0 |
| UC-SIS-09 | Recuperación tras cierre inesperado: restaurar pestañas y ediciones no guardadas. | P1 |
| UC-SIS-10 | Sistema de plugins/extensiones (para que funciones extra, incluida IA si alguien la quiere, vivan fuera del núcleo). | P2 |
| UC-SIS-11 | Configuración portátil (posibilidad de ejecutar desde USB con la configuración junto al ejecutable). | P2 |

---

## 16. Requisitos no funcionales (propuestos)

### Rendimiento
- NFR-REN-01: Arranque en frío hasta ventana usable ≤ 1 s en un equipo de gama media.
- NFR-REN-02: Primera página visible de un PDF de 1 000 páginas en ≤ 1 s.
- NFR-REN-03: Desplazamiento fluido (objetivo 60 fps) con renderizado progresivo y caché de páginas; las páginas pesadas no bloquean la interfaz.
- NFR-REN-04: Uso de memoria acotado (caché con límite configurable), sin crecer sin control en documentos grandes.
- NFR-REN-05: Operaciones largas (OCR, compresión, unión de muchos archivos) en segundo plano con progreso y botón de cancelar.

### Fiabilidad e integridad
- NFR-FIA-01: Guardado atómico (escribir a archivo temporal y reemplazar), nunca dejar un archivo a medio escribir.
- NFR-FIA-02: Opción "Guardar" (incremental, conserva firmas) y "Guardar como" (reescritura completa optimizada).
- NFR-FIA-03: Copia de respaldo opcional del original antes de sobrescribir.
- NFR-FIA-04: Si otro programa modifica el archivo abierto, avisar antes de sobrescribir.
- NFR-FIA-05: El resultado de cualquier operación debe validarse (se puede reabrir sin errores).

### Seguridad
- NFR-SEG-01: Tratar todo PDF como entrada no confiable; pruebas de fuzzing sobre el analizador.
- NFR-SEG-02: Contraseñas y claves privadas nunca en texto plano en disco ni en logs.
- NFR-SEG-03: Sin conexiones de red salvo las solicitadas explícitamente (sello de tiempo, revocación de certificados, comprobación de actualizaciones opt-in).

### Compatibilidad
- NFR-COM-01: Leer PDF 1.0 a 2.0, incluidos objetos y referencias comprimidos, archivos linealizados y con actualizaciones incrementales.
- NFR-COM-02: Lo que se escribe (anotaciones, formularios, firmas) debe mostrarse correctamente en Acrobat Reader, Chrome/Edge/Firefox y Preview de macOS.
- NFR-COM-03: Windows 10+, macOS 12+, Linux con X11 y Wayland.

### Accesibilidad
- NFR-ACC-01: Toda la interfaz utilizable solo con teclado.
- NFR-ACC-02: Compatibilidad con lectores de pantalla (NVDA, VoiceOver, Orca) para la interfaz y, si es posible, para el texto del documento.
- NFR-ACC-03: Escalado de interfaz, alto contraste y respeto de la configuración del sistema (tema, tamaño de fuente).

### Licencia y comunidad
- NFR-LIC-01: Licencia del proyecto permisiva o copyleft (por decidir) y dependencias con licencias compatibles; sin dependencias con versiones comerciales "freemium".
- NFR-LIC-02: Compilación reproducible desde el código fuente con un solo comando.

---

## 17. Casos borde a contemplar

- PDF de 0 bytes, truncado, con xref roto o con un encabezado basura antes de `%PDF`.
- PDFs enormes (> 2 GB, > 10 000 páginas) o páginas gigantes (planos A0, mapas).
- Páginas con miles de objetos vectoriales (gráficos científicos) que tardan en renderizar.
- Fuentes no incrustadas, fuentes Type3, texto sin mapa Unicode (copia devuelve basura).
- Texto de derecha a izquierda (árabe, hebreo) y vertical (CJK).
- Documentos con contraseña de permisos pero sin contraseña de apertura.
- Abrir el mismo archivo dos veces; archivo en una unidad de red o en una carpeta sincronizada (OneDrive, Dropbox) que cambia mientras está abierto.
- Rutas con acentos, emojis o muy largas (Windows > 260 caracteres).
- Disco lleno o sin permisos al guardar.
- Formularios XFA, documentos con JavaScript de validación de campos.
- Firmas con certificados caducados, revocados o sin conexión para comprobar revocación.
- Editar un documento firmado o certificado.
- Pantallas con distintas densidades (HiDPI) y mover la ventana entre monitores.

---

## 18. Fuera de alcance (propuesto)

- Funciones de IA integradas (resúmenes, chat, generación de texto).
- Almacenamiento en la nube propio, cuentas de usuario o sincronización obligatoria.
- Colaboración en tiempo real (varias personas editando a la vez).
- Editor de maquetación tipo InDesign / edición de texto de nivel procesador de textos.
- Aplicaciones móviles (al menos en la primera versión).
- Lectura/edición de XFA dinámico completo.

---

## 19. Decisiones tomadas (2026-11-15)

- **MVP:** visualización, navegación y búsqueda + **organizar páginas** + **anotar y comentar** + **llenar formularios y firma visual**.
- **Edición de texto existente (UC-EDI-02):** después del MVP.
- **Seguridad y OCR:** fuera del MVP (excepto UC-SEG-07, no ejecutar JavaScript, que es P0 por seguridad).
- **Firma digital:** certificado en archivo .p12/.pfx (firmar + validar) en v1.0; tokens PKCS#11 y firma en la nube quedan para después o como plugin.
- **Licencia:** GPL-3.0 para la aplicación; MIT/Apache-2.0 (doble) para el crate núcleo.
- **CLI:** "Could" (deseable, no bloquea el MVP).
- **Idiomas:** español e inglés.
- **Edición de texto existente:** el usuario la considera imprescindible; queda a la espera del informe de investigación para fijar alcance y versión.
- **Plugins:** a mediano plazo; pendiente de decidir el modelo.
- **Nombre de la app:** **Demi**.
- **Edición de texto existente (aprobado):** N0 (eliminar y superponer) + N1 (misma fuente) en el MVP; N2 (sustitución de fuente) en v1.0; N3 (párrafo con reflow) después. Ver `specs/research/edicion-texto-existente.md`.
- **Estilo visual:** editorial ("papelería") con la paleta del usuario (salvia, arena, rosa empolvado; `#2B5E62` acento y `#204243` como tinta). Detalle en `requirements.md` R-UI-01.
- **Fase 1 cerrada** → ver `requirements.md`.

### Dolor principal confirmado: firmar sin depender de webs ni de otra persona

Caso real: el usuario y su madre firman documentos con frecuencia. Hoy recurren a páginas web para pegar la imagen de la firma, o la madre tiene que enviarle el documento para que él coloque la firma. Casos de uso añadidos:

| ID | Caso de uso | Prioridad |
|----|-------------|-----------|
| UC-FIR-13 | Flujo "Firmar rápido" para usuarios no técnicos: abrir → botón **Firmar** → elegir firma guardada → clic donde va → guardar. Máximo 4 pasos, sin menús ocultos. | P0 |
| UC-FIR-14 | Capturar la firma desde una **foto** de la firma en papel (tomada con el celular), recortando y eliminando el fondo automáticamente. | P0 |
| UC-FIR-15 | Varias firmas guardadas con nombre (p. ej. "Mi firma", "Iniciales"), cada una de su propietario en su propio equipo. | P0 |
| UC-FIR-16 | **Preparar documento para firmar:** marcar en un PDF dónde debe firmar otra persona (campos de firma, fecha, iniciales) y enviárselo; al abrirlo, la app la guía de un campo al siguiente ("Firma aquí 1 de 3"). | P1 |
| UC-FIR-17 | Exportar/importar la firma propia de forma segura para usarla en otro equipo propio. | P2 |

## 20. Preguntas abiertas para cerrar la Fase 1

1. ¿Qué casos de uso usas **tú** más a menudo? (Servirá para ajustar el MVP.)
2. Firma digital con certificado: ¿necesitas firmar con un certificado/token de algún país o entidad en particular (p. ej. firma electrónica avanzada de Guatemala u otro)?
3. ¿La CLI forma parte del MVP o puede esperar?
4. Edición de texto existente (UC-EDI-02): es la función más difícil técnicamente. ¿Es imprescindible para ti o puede ir después?
5. Licencia del proyecto: ¿permisiva (MIT/Apache-2.0) o copyleft (GPL-3.0/MPL-2.0)?
6. ¿Un sistema de plugins es importante a medio plazo?
7. Idiomas de la interfaz: ¿español e inglés bastan para empezar?
