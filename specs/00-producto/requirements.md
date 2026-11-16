# Demi — Requisitos del producto (`requirements.md`)

> **Spec:** 00-producto · **Fase:** 1 (Requisitos) · **Estado:** aprobado para pasar a diseño
> **Fecha:** 2026-11-15
> **Fuentes:** `descubrimiento.md` (casos de uso UC-*), `../research/edicion-texto-existente.md`
> **Siguiente fase:** `design.md` (arquitectura, módulos, sistema de diseño visual)

---

## 1. Introducción

**Demi** es un visor y editor de PDF de escritorio, libre y de código abierto, para Windows, macOS y Linux, escrito en Rust. Permite leer, anotar, organizar, llenar, firmar, editar y proteger PDFs **sin paywalls, sin cuentas, sin nube obligatoria, sin telemetría y sin IA impuesta**.

### 1.1 Objetivos

| # | Objetivo | Indicador de éxito |
|---|----------|--------------------|
| O-1 | Que una persona no técnica pueda firmar un PDF sola, sin webs externas. | Firmar un documento con firma guardada en ≤ 4 pasos y < 30 s. |
| O-2 | Sustituir a lectores comerciales en el uso diario de lectura, anotación y organización. | Todos los requisitos MVP cumplidos; compatibilidad de salida con Acrobat, navegadores y Preview. |
| O-3 | Corregir texto existente en PDFs sin herramientas de pago. | Edición de una palabra/línea (N1) con resultado visualmente idéntico fuera de la zona editada. |
| O-4 | No corromper nunca un documento. | Cero archivos corruptos en la suite de pruebas de guardado (incluye cortes simulados). |

### 1.2 Decisiones de producto

| Tema | Decisión |
|------|----------|
| Nombre | **Demi** |
| Licencias | Aplicación: **GPL-3.0**. Crate núcleo (`demi-core` o similar): **MIT OR Apache-2.0**. |
| Lenguaje | Rust. Motor de PDF basado en PDFium (`pdfium-render`) + `lopdf`; el diseño detallado se decide en `design.md`. |
| Idiomas de interfaz | Español e inglés desde el MVP. |
| Estilo visual | Editorial ("papelería"), con la paleta definida en §12. |
| Firma digital | Certificados en archivo `.p12/.pfx` (v1.0). Tokens PKCS#11 y firma en la nube: posterior o plugin. |
| Edición de texto existente | N0 + N1 en MVP; N2 en v1.0; N3 posterior. |
| CLI | Deseable (Could), v1.0 si hay capacidad. |
| Plugins | Posterior; el núcleo debe diseñarse con un catálogo de acciones reutilizable. |

### 1.3 Versiones y prioridades

| Etiqueta | Significado |
|----------|-------------|
| **MVP** | Primera versión usable a diario. Must. |
| **v1.0** | Primera versión estable y completa. Should (CLI: Could). |
| **Posterior** | Roadmap tras v1.0. Could / Won't por ahora. |

### 1.4 Convención EARS

Cada criterio de aceptación sigue uno de los patrones EARS (Easy Approach to Requirements Syntax):

| Patrón | Forma |
|--------|-------|
| Ubicuo | **EL SISTEMA DEBERÁ** \<respuesta\> |
| Evento | **CUANDO** \<disparador\>, **EL SISTEMA DEBERÁ** \<respuesta\> |
| Estado | **MIENTRAS** \<estado\>, **EL SISTEMA DEBERÁ** \<respuesta\> |
| Opcional | **DONDE** \<característica presente\>, **EL SISTEMA DEBERÁ** \<respuesta\> |
| No deseado | **SI** \<condición no deseada\>, **ENTONCES EL SISTEMA DEBERÁ** \<respuesta\> |

Los IDs de requisito (`R-XXX-NN`) se trazan a los casos de uso (`UC-XXX-NN`) del documento de descubrimiento.

### 1.5 Usuarios tipo

- **Ana (oficina):** llena, firma y devuelve contratos y formularios.
- **Luis (estudiante/investigador):** lee documentos largos, subraya y exporta notas.
- **Marta (administración):** une, divide y comprime facturas y estados de cuenta.
- **Diego (legal/RR. HH.):** redacta datos sensibles, protege y verifica firmas.
- **Sofía (power user):** atajos, paleta de comandos, CLI.
- **Elena (usuaria no técnica, caso real):** firma documentos con frecuencia y hoy depende de otra persona o de webs.

---

## 2. Visualización y lectura

### R-VIS-01 Abrir documentos — MVP (UC-VIS-01, UC-SIS-02)
**Historia:** Como usuario, quiero abrir un PDF de la forma que me resulte natural, para empezar a trabajar sin pasos extra.

1. CUANDO el usuario elija "Abrir" y seleccione un archivo PDF, EL SISTEMA DEBERÁ mostrar el documento en una pestaña nueva.
2. CUANDO el usuario arrastre uno o más PDFs sobre la ventana, EL SISTEMA DEBERÁ abrir cada uno en su propia pestaña.
3. CUANDO Demi se ejecute con rutas de archivo como argumentos, EL SISTEMA DEBERÁ abrir esos archivos.
4. CUANDO el sistema operativo abra un PDF con Demi y Demi ya esté en ejecución, EL SISTEMA DEBERÁ abrir el documento en la instancia existente en lugar de lanzar otra.
5. SI el archivo ya está abierto, ENTONCES EL SISTEMA DEBERÁ enfocar la pestaña existente en lugar de abrir un duplicado.
6. SI el archivo no es un PDF válido ni reparable, ENTONCES EL SISTEMA DEBERÁ mostrar un mensaje comprensible con la causa y no cerrar otras pestañas.

### R-VIS-02 Zoom — MVP (UC-VIS-02)
1. EL SISTEMA DEBERÁ ofrecer los modos de zoom: ajustar al ancho, ajustar a la página, tamaño real (100 %) y porcentaje libre entre 10 % y 3200 %.
2. CUANDO el usuario use Ctrl/Cmd + rueda del ratón o el gesto de pellizcar, EL SISTEMA DEBERÁ hacer zoom centrado en la posición del puntero.
3. CUANDO cambie el zoom, EL SISTEMA DEBERÁ mostrar primero una versión escalada de la página y reemplazarla por la de alta resolución en ≤ 300 ms (página típica).

### R-VIS-03 Modos de vista — MVP (UC-VIS-03, UC-VIS-04)
1. EL SISTEMA DEBERÁ ofrecer los modos: una página, desplazamiento continuo y dos páginas (con y sin portada separada).
2. CUANDO el usuario rote la vista, EL SISTEMA DEBERÁ rotar la visualización 90° sin modificar el archivo.

### R-VIS-04 Recordar estado de lectura — MVP (UC-VIS-06)
1. CUANDO el usuario cierre un documento, EL SISTEMA DEBERÁ guardar localmente página, desplazamiento, zoom y modo de vista asociados a ese archivo.
2. CUANDO el usuario reabra ese documento, EL SISTEMA DEBERÁ restaurar ese estado.
3. EL SISTEMA DEBERÁ identificar el documento por un hash de contenido además de la ruta, para reconocerlo si se movió o renombró.

### R-VIS-05 Documentos protegidos — MVP (UC-VIS-09)
1. CUANDO el usuario abra un PDF con contraseña de apertura, EL SISTEMA DEBERÁ solicitar la contraseña en un diálogo.
2. SI la contraseña es incorrecta, ENTONCES EL SISTEMA DEBERÁ indicarlo y permitir reintentar sin cerrar el diálogo.
3. EL SISTEMA NO DEBERÁ guardar contraseñas de documentos en disco ni en registros.

### R-VIS-06 Documentos dañados — MVP (UC-VIS-10)
1. SI el documento tiene la tabla xref dañada o datos basura antes de `%PDF`, ENTONCES EL SISTEMA DEBERÁ intentar repararlo en memoria y abrirlo.
2. CUANDO un documento se abra tras una reparación, EL SISTEMA DEBERÁ mostrar un aviso no intrusivo y no sobrescribir el original salvo que el usuario guarde.

### R-VIS-07 Pestañas, ventanas y vista dividida — v1.0 (UC-VIS-05)
1. EL SISTEMA DEBERÁ permitir mover una pestaña a una ventana nueva y entre ventanas.
2. EL SISTEMA DEBERÁ permitir ver dos documentos, o dos posiciones del mismo, lado a lado.

### R-VIS-08 Modo lectura y presentación — v1.0 (UC-VIS-07)
1. CUANDO el usuario active el modo lectura, EL SISTEMA DEBERÁ ocultar paneles y barras, dejando solo el documento.
2. CUANDO el usuario active el modo presentación, EL SISTEMA DEBERÁ mostrar una página por pantalla completa y avanzar con flechas, clic, espacio o retroceso.

### R-VIS-09 Temas y colores del contenido — v1.0 (UC-VIS-08)
1. EL SISTEMA DEBERÁ ofrecer tema claro y oscuro de la interfaz, y seguir por defecto el tema del sistema.
2. EL SISTEMA DEBERÁ ofrecer modos de color del contenido: original, sepia, invertido inteligente (sin invertir imágenes) y alto contraste.
3. EL SISTEMA DEBERÁ aplicar el modo de color solo a la visualización, nunca al archivo ni a la impresión.

### R-VIS-10 Adjuntos y propiedades — v1.0 (UC-VIS-12, UC-VIS-13)
1. DONDE el documento contenga archivos adjuntos, EL SISTEMA DEBERÁ listarlos y permitir guardarlos en disco.
2. EL SISTEMA DEBERÁ mostrar las propiedades: título, autor, productor, fechas, versión PDF, número de páginas, tamaño de página, fuentes, cifrado, firmas, conformidad PDF/A y si contiene JavaScript.

### R-VIS-11 Herramientas de vista — v1.0 (UC-VIS-15, UC-VIS-17)
1. EL SISTEMA DEBERÁ ofrecer herramienta de mano (desplazar) y zoom a una región seleccionada.
2. DONDE el usuario active "recargar automáticamente", CUANDO el archivo cambie en disco y no haya cambios sin guardar, EL SISTEMA DEBERÁ recargarlo conservando la posición.

### R-VIS-12 Funciones de vista posteriores — Posterior (UC-VIS-11, 14, 16, 18)
1. DONDE el documento tenga capas opcionales, EL SISTEMA DEBERÁ permitir mostrarlas u ocultarlas.
2. DONDE el documento contenga multimedia, EL SISTEMA DEBERÁ informarlo y permitir extraerla.
3. EL SISTEMA DEBERÁ ofrecer medición de distancia, perímetro y área con escala configurable.
4. EL SISTEMA DEBERÁ poder abrir imágenes y EPUB en modo solo lectura.

---

## 3. Navegación y búsqueda

### R-NAV-01 Miniaturas y marcadores — MVP (UC-NAV-01, UC-NAV-02)
1. EL SISTEMA DEBERÁ mostrar un panel lateral de miniaturas con tamaño ajustable y la página actual resaltada.
2. DONDE el documento tenga marcadores (outline), EL SISTEMA DEBERÁ mostrarlos como un árbol plegable y navegar al destino al hacer clic.
3. MIENTRAS el usuario se desplaza, EL SISTEMA DEBERÁ resaltar en el árbol el marcador de la sección actual.

### R-NAV-02 Ir a página e historial — MVP (UC-NAV-03, UC-NAV-04)
1. CUANDO el usuario escriba un número o una etiqueta de página (p. ej. "iv"), EL SISTEMA DEBERÁ navegar a esa página.
2. CUANDO el usuario siga un enlace interno, EL SISTEMA DEBERÁ registrar la posición anterior en un historial atrás/adelante (Alt+←/→).

### R-NAV-03 Enlaces — MVP (UC-NAV-05)
1. CUANDO el usuario active un enlace interno, EL SISTEMA DEBERÁ navegar al destino.
2. CUANDO el usuario active un enlace externo, EL SISTEMA DEBERÁ mostrar la URL completa y pedir confirmación antes de abrir el navegador.

### R-NAV-04 Búsqueda en documento — MVP (UC-NAV-06)
1. CUANDO el usuario escriba en la barra de búsqueda (Ctrl/Cmd+F), EL SISTEMA DEBERÁ resaltar todas las coincidencias y mostrar "n de N".
2. EL SISTEMA DEBERÁ encontrar coincidencias a través de ligaduras (fi, fl) y de palabras partidas por guion al final de línea.
3. MIENTRAS la búsqueda recorre un documento grande, EL SISTEMA DEBERÁ mostrar resultados parciales sin bloquear la interfaz.
4. SI el documento no tiene texto extraíble, ENTONCES EL SISTEMA DEBERÁ indicarlo (y, desde v1.0, sugerir OCR).

### R-NAV-05 Opciones y panel de búsqueda — v1.0 (UC-NAV-07, UC-NAV-08)
1. EL SISTEMA DEBERÁ ofrecer opciones: distinguir mayúsculas, palabra completa, ignorar acentos (activado por defecto) y expresión regular.
2. EL SISTEMA DEBERÁ mostrar un panel con cada resultado, su página y un fragmento de contexto.

### R-NAV-06 Recientes — MVP (UC-NAV-12)
1. EL SISTEMA DEBERÁ mostrar en la pantalla de inicio los documentos recientes con miniatura, nombre y fecha.
2. EL SISTEMA DEBERÁ permitir quitar elementos o borrar todo el historial de recientes.
3. SI un archivo reciente ya no existe, ENTONCES EL SISTEMA DEBERÁ marcarlo como no disponible.

### R-NAV-07 Marcadores propios — v1.0 (UC-NAV-10)
1. EL SISTEMA DEBERÁ permitir crear, renombrar, anidar, reordenar y borrar marcadores del documento y guardarlos en el PDF.

### R-NAV-08 Navegación posterior — Posterior (UC-NAV-09, UC-NAV-11)
1. EL SISTEMA DEBERÁ permitir buscar texto en todos los PDFs de una carpeta.
2. EL SISTEMA DEBERÁ ofrecer marcadores personales del lector que no modifican el PDF.

---

## 4. Selección y extracción

### R-EXT-01 Selección y copia de texto — MVP (UC-EXT-01)
1. CUANDO el usuario seleccione texto con el ratón, EL SISTEMA DEBERÁ resaltar la selección siguiendo el orden de lectura.
2. CUANDO el usuario copie la selección, EL SISTEMA DEBERÁ unir las líneas de un mismo párrafo, eliminar guiones de corte y conservar los saltos entre párrafos.
3. SI el texto no tiene mapeo Unicode fiable, ENTONCES EL SISTEMA DEBERÁ advertir que el texto copiado puede ser incorrecto.

### R-EXT-02 Extracción avanzada — v1.0 (UC-EXT-02, 03, 04, 05, 07, 08)
1. CUANDO el usuario arrastre con Alt presionado, EL SISTEMA DEBERÁ hacer una selección rectangular.
2. EL SISTEMA DEBERÁ permitir copiar una región como imagen al portapapeles.
3. EL SISTEMA DEBERÁ extraer las imágenes de páginas seleccionadas en su resolución y formato original.
4. EL SISTEMA DEBERÁ exportar el texto del documento a TXT y Markdown.
5. EL SISTEMA DEBERÁ exportar páginas como PNG, JPEG o WebP al DPI elegido (72–600).
6. EL SISTEMA DEBERÁ exportar las anotaciones a Markdown con página, tipo, texto resaltado y comentario.

### R-EXT-03 Tablas — Posterior (UC-EXT-06)
1. EL SISTEMA DEBERÁ detectar tablas en una región seleccionada y exportarlas a CSV.

---

## 5. Anotación y revisión

### R-ANO-01 Marcado de texto — MVP (UC-ANO-01)
1. CUANDO el usuario seleccione texto con la herramienta de resaltado, subrayado o tachado activa, EL SISTEMA DEBERÁ crear la anotación correspondiente sobre ese texto.
2. EL SISTEMA DEBERÁ ofrecer al menos 6 colores predefinidos derivados de la paleta (§12) y un selector libre.
3. EL SISTEMA DEBERÁ recordar el último color usado por herramienta.

### R-ANO-02 Notas y cuadros de texto — MVP (UC-ANO-02, UC-ANO-03)
1. CUANDO el usuario haga clic con la herramienta de nota, EL SISTEMA DEBERÁ crear una nota adhesiva con autor y fecha.
2. CUANDO el usuario haga clic o arrastre con la herramienta de texto, EL SISTEMA DEBERÁ crear un cuadro de texto libre editable con fuente, tamaño, color, borde y fondo configurables.

### R-ANO-03 Gestión de anotaciones — MVP (UC-ANO-08, UC-ANO-10, UC-ANO-14)
1. EL SISTEMA DEBERÁ permitir seleccionar, mover, redimensionar, recolorear y borrar anotaciones, incluida la selección múltiple.
2. EL SISTEMA DEBERÁ mostrar un panel con todas las anotaciones, filtrable por tipo, color, autor y página, que navegue a cada una al hacer clic.
3. EL SISTEMA DEBERÁ usar como autor el nombre configurado por el usuario y, si no hay ninguno, dejar el autor vacío (nunca el nombre de usuario del sistema operativo sin consentimiento).

### R-ANO-04 Compatibilidad de anotaciones — MVP (UC-ANO-11)
1. EL SISTEMA DEBERÁ guardar todas las anotaciones como objetos de anotación PDF estándar (ISO 32000) con flujo de apariencia (`/AP`) generado.
2. Las anotaciones creadas por Demi DEBERÁN mostrarse correctamente en Adobe Acrobat Reader, Chrome, Firefox y Preview de macOS (verificado en la suite de compatibilidad).
3. CUANDO Demi abra un PDF con anotaciones creadas por otras aplicaciones, EL SISTEMA DEBERÁ mostrarlas y permitir editarlas sin perder datos que no entienda.

### R-ANO-05 Dibujo, formas y sellos — v1.0 (UC-ANO-04, 05, 06, 15)
1. EL SISTEMA DEBERÁ ofrecer dibujo a mano alzada con grosor y color, con suavizado del trazo.
2. DONDE haya un lápiz con presión (tableta), EL SISTEMA DEBERÁ variar el grosor según la presión.
3. EL SISTEMA DEBERÁ ofrecer formas: rectángulo, elipse, línea, flecha, polilínea, polígono y nube.
4. EL SISTEMA DEBERÁ ofrecer sellos predefinidos ("Aprobado", "Borrador", "Confidencial", "Recibido" con fecha) en español e inglés, y sellos personalizados desde imagen.
5. EL SISTEMA DEBERÁ asignar atajos de una tecla a las herramientas de anotación (p. ej. `H` resaltar, `N` nota, `T` texto).

### R-ANO-06 Revisión — v1.0 (UC-ANO-09, UC-ANO-12)
1. EL SISTEMA DEBERÁ permitir responder a anotaciones (hilos) y asignar estado: aceptado, rechazado, cancelado, completado.
2. CUANDO el usuario elija "Aplanar anotaciones", EL SISTEMA DEBERÁ incorporarlas al contenido de la página tras confirmación, en una copia o en el mismo archivo según elija.

### R-ANO-07 Anotación posterior — Posterior (UC-ANO-07, UC-ANO-13)
1. EL SISTEMA DEBERÁ ofrecer llamadas (callouts).
2. EL SISTEMA DEBERÁ importar y exportar anotaciones en XFDF/FDF.

---

## 6. Edición del contenido

### R-EDI-01 Deshacer y rehacer — MVP (UC-EDI-10)
1. EL SISTEMA DEBERÁ permitir deshacer y rehacer (Ctrl/Cmd+Z, Ctrl/Cmd+Shift+Z) todas las operaciones que modifican el documento durante la sesión, sin límite práctico de pasos.
2. CUANDO el usuario guarde, EL SISTEMA DEBERÁ conservar el historial de deshacer de la sesión abierta.

### R-EDI-02 Editar texto existente, nivel N0 (eliminar y superponer) — MVP (UC-EDI-02)
**Historia:** Como usuario, quiero cambiar un texto aunque no se pueda editar "de verdad", sabiendo que el texto original desaparece por completo.

1. CUANDO el usuario aplique una edición en modo N0, EL SISTEMA DEBERÁ eliminar del content stream los glifos originales de la zona editada y dibujar el texto nuevo en su lugar.
2. EL SISTEMA NO DEBERÁ limitarse a tapar el texto original con un rectángulo: tras guardar, el texto original no debe poder extraerse ni buscarse.
3. CUANDO se use N0, EL SISTEMA DEBERÁ informar al usuario de que el estilo podría no coincidir exactamente.

### R-EDI-03 Editar texto existente, nivel N1 (misma fuente) — MVP (UC-EDI-02)
**Historia:** Como usuario, quiero corregir una palabra, número, fecha o línea de un PDF y que se vea igual que el resto.

1. CUANDO el usuario active "Editar texto", EL SISTEMA DEBERÁ resaltar los bloques de texto editables de la página.
2. CUANDO el usuario haga clic en un bloque, EL SISTEMA DEBERÁ permitir editar su texto en línea, con cursor y selección.
3. CUANDO el usuario confirme una edición y todos los caracteres nuevos existan en la fuente incrustada, EL SISTEMA DEBERÁ reescribir el texto con la misma fuente, tamaño, color y posición inicial.
4. SI algún carácter nuevo no existe en la fuente incrustada, ENTONCES EL SISTEMA DEBERÁ indicarlo antes de aplicar y ofrecer N0 (MVP) o N2 (v1.0).
5. SI el texto nuevo es más ancho que el espacio original disponible en la línea, ENTONCES EL SISTEMA DEBERÁ advertirlo y ofrecer: ajustar el espaciado, reducir ligeramente el tamaño o aceptar el desborde.
6. Tras la edición, la página DEBERÁ ser visualmente idéntica fuera de la zona editada (SSIM ≥ 0,999 con máscara), y el texto nuevo DEBERÁ poder extraerse y buscarse.
7. SI el documento tiene firmas digitales, ENTONCES EL SISTEMA DEBERÁ advertir antes de editar que la firma quedará invalidada.
8. SI el texto pertenece a un contenido que el sistema no puede editar con seguridad (Type3, texto como contorno, imagen escaneada), ENTONCES EL SISTEMA DEBERÁ indicarlo y ofrecer N0 o, para escaneos, OCR (v1.0).

### R-EDI-04 Editar texto existente, nivel N2 (sustitución de fuente) — v1.0
1. SI faltan glifos en la fuente original, ENTONCES EL SISTEMA DEBERÁ proponer una fuente sustituta en este orden: fuente del sistema con el mismo nombre (si su licencia permite incrustarla), fuente métricamente compatible incluida (Liberation, Carlito, Caladea, Croscore), fuente Noto de la misma clase.
2. EL SISTEMA DEBERÁ mostrar una vista previa con la sustituta antes de aplicar.
3. EL SISTEMA DEBERÁ incrustar solo el subconjunto de glifos necesario.
4. EL SISTEMA NO DEBERÁ incrustar fuentes cuyo campo `fsType` lo prohíba.

### R-EDI-05 Editar texto existente, nivel N3 (párrafo con reflujo) — Posterior
1. CUANDO el usuario edite un párrafo detectado, EL SISTEMA DEBERÁ recomponer las líneas del párrafo respetando su ancho, alineación e interlineado.

### R-EDI-06 Agregar contenido — v1.0 (UC-EDI-01, 03, 05, 06, 08)
1. EL SISTEMA DEBERÁ permitir agregar texto nuevo como contenido de página con fuente, tamaño y color.
2. EL SISTEMA DEBERÁ permitir insertar, mover, redimensionar, recortar, reemplazar y borrar imágenes.
3. EL SISTEMA DEBERÁ permitir agregar marcas de agua de texto o imagen con opacidad, rotación, posición y rango de páginas, con vista previa.
4. EL SISTEMA DEBERÁ permitir agregar encabezados, pies y numeración de páginas en formatos "1", "Página 1 de N", romanos y Bates.
5. EL SISTEMA DEBERÁ permitir editar los metadatos título, autor, asunto y palabras clave.

### R-EDI-07 Edición posterior — Posterior (UC-EDI-04, 07, 09)
1. EL SISTEMA DEBERÁ permitir seleccionar, mover y borrar objetos de página individuales.
2. EL SISTEMA DEBERÁ permitir crear y editar enlaces.
3. EL SISTEMA DEBERÁ permitir cambiar el fondo de las páginas.

---

## 7. Organización de páginas

### R-PAG-01 Vista de organizador — MVP (UC-PAG-01)
1. CUANDO el usuario abra "Organizar", EL SISTEMA DEBERÁ mostrar todas las páginas como miniaturas en una cuadrícula con tamaño ajustable.
2. EL SISTEMA DEBERÁ permitir seleccionar páginas con clic, Ctrl/Cmd+clic, Shift+clic y rangos escritos ("1-3, 7").
3. CUANDO el usuario arrastre páginas a otra posición, EL SISTEMA DEBERÁ reordenarlas mostrando el punto de inserción.

### R-PAG-02 Operaciones básicas — MVP (UC-PAG-02, 03, 05, 07)
1. EL SISTEMA DEBERÁ rotar de forma permanente las páginas seleccionadas 90° a izquierda/derecha o 180°.
2. EL SISTEMA DEBERÁ eliminar, duplicar e insertar páginas en blanco (tamaño igual a la página adyacente).
3. EL SISTEMA DEBERÁ insertar en una posición las páginas de otro PDF o imágenes.
4. EL SISTEMA DEBERÁ extraer las páginas seleccionadas a un PDF nuevo.
5. CUANDO se eliminen páginas, EL SISTEMA DEBERÁ eliminar o redirigir los marcadores y enlaces que apunten a ellas.

### R-PAG-03 Unir documentos — MVP (UC-PAG-04)
1. CUANDO el usuario elija "Unir" con una lista ordenable de PDFs, EL SISTEMA DEBERÁ crear un documento con todas sus páginas en ese orden.
2. EL SISTEMA DEBERÁ conservar los marcadores de cada documento, agrupados bajo un marcador con su nombre de archivo.
3. SI algún documento está cifrado, ENTONCES EL SISTEMA DEBERÁ pedir su contraseña antes de unir.

### R-PAG-04 Dividir documentos — MVP (UC-PAG-06)
1. EL SISTEMA DEBERÁ dividir un documento: por rangos, cada N páginas, por marcadores de primer nivel y en páginas sueltas.
2. EL SISTEMA DEBERÁ mostrar una vista previa de los archivos resultantes y su patrón de nombre antes de ejecutar.
3. (v1.0) EL SISTEMA DEBERÁ dividir por tamaño máximo de archivo.

### R-PAG-05 Recorte, intercalado y compresión — v1.0 (UC-PAG-08, 10, 13)
1. EL SISTEMA DEBERÁ permitir recortar páginas manualmente y recortar márgenes en blanco de forma automática.
2. EL SISTEMA DEBERÁ intercalar dos documentos (impares + pares), con opción de invertir el orden del segundo.
3. EL SISTEMA DEBERÁ comprimir con perfiles (alta calidad, correo, mínima) y mostrar el tamaño estimado antes de guardar.
4. SI la compresión no reduce el tamaño, ENTONCES EL SISTEMA DEBERÁ informarlo y no reemplazar el archivo.

### R-PAG-06 Organización posterior — Posterior (UC-PAG-09, 11, 12)
1. EL SISTEMA DEBERÁ cambiar el tamaño de página o escalar su contenido.
2. EL SISTEMA DEBERÁ generar imposiciones N-up y folletos.
3. EL SISTEMA DEBERÁ editar las etiquetas de página.

---

## 8. Formularios

### R-FOR-01 Llenar formularios — MVP (UC-FOR-01, UC-FOR-02)
1. DONDE el documento tenga campos AcroForm, EL SISTEMA DEBERÁ resaltarlos y permitir llenarlos: texto, multilínea, casillas, botones de opción, listas y combos.
2. CUANDO el usuario presione Tab o Shift+Tab, EL SISTEMA DEBERÁ moverse al campo siguiente o anterior según el orden del documento.
3. EL SISTEMA DEBERÁ respetar las propiedades de solo lectura, longitud máxima y requerido.

### R-FOR-02 Apariencias al guardar — MVP (UC-FOR-04)
1. CUANDO el usuario guarde un formulario, EL SISTEMA DEBERÁ generar las apariencias de cada campo modificado para que se vea igual en cualquier lector.
2. Los formularios llenados con Demi DEBERÁN mostrar sus valores en Acrobat Reader, Chrome, Firefox y Preview.

### R-FOR-03 Llenar y firmar PDFs sin campos — MVP (UC-FOR-05)
1. EL SISTEMA DEBERÁ permitir colocar en cualquier punto de la página: texto, ✓, ✗, punto, fecha (con formato configurable) e iniciales o firma.
2. MIENTRAS el usuario coloca texto, EL SISTEMA DEBERÁ ofrecer alineación con líneas detectadas de la página (ajuste magnético opcional).

### R-FOR-04 Validación, aplanado y XFA — v1.0 (UC-FOR-03, 06, 10)
1. EL SISTEMA DEBERÁ aplicar los formatos y validaciones estándar de campo (número, fecha, porcentaje, especial) declarados en el PDF sin ejecutar JavaScript arbitrario.
2. CUANDO el usuario elija "Aplanar formulario", EL SISTEMA DEBERÁ convertir los campos en contenido no editable, preferentemente en una copia.
3. SI el formulario es XFA, ENTONCES EL SISTEMA DEBERÁ indicarlo claramente y mostrar la versión AcroForm de respaldo si existe.

### R-FOR-05 Formularios posteriores — Posterior (UC-FOR-07, 08, 09, 11)
1. EL SISTEMA DEBERÁ permitir crear y editar campos de formulario de todos los tipos y su orden de tabulación.
2. EL SISTEMA DEBERÁ detectar posibles campos en PDFs planos.
3. EL SISTEMA DEBERÁ importar/exportar datos (FDF, XFDF, JSON, CSV) y llenar en lote desde CSV.
4. EL SISTEMA DEBERÁ guardar perfiles de autocompletado locales.

---

## 9. Firmas

### R-FIR-01 Crear firma visual — MVP (UC-FIR-01, UC-FIR-14)
**Historia:** Como Elena, quiero tener mi firma guardada en mi computadora para no depender de nadie ni de páginas web.

1. EL SISTEMA DEBERÁ permitir crear una firma: dibujándola (ratón, trackpad, lápiz), escribiéndola con una tipografía manuscrita incluida o importando una imagen.
2. CUANDO el usuario importe una foto de su firma en papel, EL SISTEMA DEBERÁ recortarla automáticamente, eliminar el fondo, aumentar el contraste del trazo y mostrar el resultado para confirmar o ajustar.
3. EL SISTEMA DEBERÁ permitir elegir el color de la tinta (negro, azul, `#204243`).
4. EL SISTEMA DEBERÁ guardar la firma como imagen con transparencia y, cuando sea dibujada, como trazo vectorial.

### R-FIR-02 Firmas guardadas — MVP (UC-FIR-02, UC-FIR-15)
1. EL SISTEMA DEBERÁ permitir guardar varias firmas e iniciales con nombre (p. ej. "Mi firma", "Iniciales").
2. EL SISTEMA DEBERÁ almacenar las firmas en el perfil local del usuario, protegidas con el almacén seguro del sistema operativo (Windows DPAPI / Credential Manager, Llavero de macOS, Secret Service en Linux) o cifradas en disco.
3. EL SISTEMA NO DEBERÁ incluir las firmas guardadas en registros, informes de error ni archivos temporales sin cifrar.

### R-FIR-03 Firmar rápido — MVP (UC-FIR-03, UC-FIR-13)
**Historia:** Como Elena, quiero firmar en pocos pasos, sin buscar en menús.

1. EL SISTEMA DEBERÁ mostrar un botón **Firmar** siempre visible en la barra principal.
2. CUANDO el usuario pulse Firmar y tenga firmas guardadas, EL SISTEMA DEBERÁ mostrarlas para elegir con un clic; SI no tiene ninguna, ENTONCES EL SISTEMA DEBERÁ abrir directamente el asistente de creación.
3. CUANDO el usuario elija una firma, EL SISTEMA DEBERÁ mostrarla siguiendo al puntero y colocarla con un clic.
4. EL SISTEMA DEBERÁ permitir mover y redimensionar la firma colocada manteniendo la proporción.
5. EL SISTEMA DEBERÁ ofrecer agregar la fecha junto a la firma con un clic.
6. EL SISTEMA DEBERÁ permitir colocar iniciales en todas las páginas o en un rango con una sola acción.
7. Una usuaria sin experiencia previa DEBERÁ poder firmar un documento con una firma guardada en ≤ 4 pasos (Abrir → Firmar → Elegir → Colocar) y guardarlo (prueba de usabilidad con ≥ 3 personas no técnicas).
8. CUANDO el usuario guarde un documento con firma visual, EL SISTEMA DEBERÁ incorporar la firma al contenido de la página (no como anotación editable), para que no pueda moverse ni copiarse fácilmente en otro lector.

### R-FIR-04 Firma digital con certificado .p12/.pfx — v1.0 (UC-FIR-04, 06, 07, 09)
1. CUANDO el usuario elija "Firmar con certificado" y seleccione un archivo `.p12/.pfx` con su contraseña, EL SISTEMA DEBERÁ firmar el documento conforme a PAdES B-B.
2. DONDE el usuario configure un servidor de sello de tiempo (TSA), EL SISTEMA DEBERÁ producir firmas PAdES B-T.
3. EL SISTEMA DEBERÁ permitir firma visible (nombre, fecha, motivo, ubicación e imagen opcional) o invisible.
4. EL SISTEMA DEBERÁ firmar mediante guardado incremental, de modo que las firmas previas sigan siendo válidas.
5. EL SISTEMA NO DEBERÁ guardar la contraseña del certificado ni la clave privada fuera del almacén seguro, y DEBERÁ borrarlas de memoria tras su uso.
6. SI el certificado está caducado o aún no es válido, ENTONCES EL SISTEMA DEBERÁ advertirlo antes de firmar.

### R-FIR-05 Validación de firmas — v1.0 (UC-FIR-08, UC-FIR-12)
1. CUANDO se abra un documento firmado, EL SISTEMA DEBERÁ mostrar una barra con el estado global: válida, válida con advertencias o inválida.
2. EL SISTEMA DEBERÁ mostrar por cada firma: firmante, fecha, sello de tiempo, si el documento cambió después y qué cambió (rango de bytes/revisión), y la cadena de certificados.
3. EL SISTEMA DEBERÁ permitir ver la revisión del documento tal como estaba en el momento de cada firma.
4. EL SISTEMA DEBERÁ usar el almacén de certificados de confianza del sistema operativo y permitir confiar manualmente en un certificado.
5. DONDE haya conexión y el usuario lo permita, EL SISTEMA DEBERÁ comprobar la revocación (OCSP/CRL); SI no puede, ENTONCES DEBERÁ indicarlo como advertencia, no como error.
6. CUANDO el usuario intente modificar un documento firmado, EL SISTEMA DEBERÁ advertir el efecto sobre las firmas antes de aplicar el cambio.

### R-FIR-06 Preparar documento para firmar — v1.0 (UC-FIR-16)
**Historia:** Como usuario, quiero marcar dónde debe firmar mi mamá, para que ella abra el documento y la app la guíe.

1. EL SISTEMA DEBERÁ permitir colocar campos de firma, iniciales y fecha (campos de firma PDF estándar), con nombre del firmante esperado.
2. CUANDO alguien abra un documento con campos de firma vacíos, EL SISTEMA DEBERÁ mostrar el aviso "Este documento necesita tu firma" y un botón para empezar.
3. MIENTRAS haya campos pendientes, EL SISTEMA DEBERÁ guiar de uno en uno ("Firma aquí · 1 de 3"), desplazando la vista hasta cada campo.
4. CUANDO se completen todos, EL SISTEMA DEBERÁ ofrecer guardar con un nombre sugerido (p. ej. `<nombre>-firmado.pdf`).

### R-FIR-07 Firmas posteriores — Posterior (UC-FIR-05, 07 LT/LTA, 10, 11, 17)
1. EL SISTEMA DEBERÁ firmar con tokens y tarjetas vía PKCS#11 y con los almacenes de certificados del sistema.
2. EL SISTEMA DEBERÁ producir firmas PAdES B-LT y B-LTA.
3. EL SISTEMA DEBERÁ permitir certificar documentos (firma de autor con permisos de cambio).
4. EL SISTEMA DEBERÁ crear certificados autofirmados con advertencia de validez limitada.
5. EL SISTEMA DEBERÁ exportar e importar firmas visuales propias en un archivo protegido con contraseña.

---

## 10. Seguridad y privacidad

### R-SEG-01 Contenido activo — MVP (UC-SEG-07)
1. EL SISTEMA NO DEBERÁ ejecutar JavaScript incrustado en documentos PDF.
2. EL SISTEMA NO DEBERÁ ejecutar acciones de tipo "lanzar archivo/aplicación" ni enviar formularios a URLs.
3. EL SISTEMA NO DEBERÁ cargar recursos remotos al abrir un documento.

### R-SEG-02 Redacción real — v1.0 (UC-SEG-01, 02, 03)
1. EL SISTEMA DEBERÁ permitir marcar áreas o texto para redactar y revisarlas antes de aplicar.
2. EL SISTEMA DEBERÁ permitir buscar patrones (texto, regex, correos, teléfonos, números de identificación) y marcar todas las coincidencias para revisión.
3. CUANDO el usuario aplique la redacción, EL SISTEMA DEBERÁ eliminar el texto, las porciones de imagen y los vectores subyacentes, y dibujar el relleno elegido.
4. Tras aplicar, el contenido redactado NO DEBERÁ poder extraerse por copia, búsqueda ni herramientas externas (verificado automáticamente tras guardar).
5. EL SISTEMA DEBERÁ guardar la redacción en un archivo nuevo por defecto.

### R-SEG-03 Contraseñas y permisos — v1.0 (UC-SEG-04, UC-SEG-05)
1. EL SISTEMA DEBERÁ cifrar documentos con AES-256 y contraseña de apertura y/o de permisos (imprimir, copiar, editar, anotar, llenar formularios).
2. CUANDO el usuario conozca la contraseña, EL SISTEMA DEBERÁ permitir quitar el cifrado.
3. EL SISTEMA DEBERÁ mostrar un indicador de fortaleza de contraseña.

### R-SEG-04 Limpieza del documento — v1.0 (UC-SEG-06, UC-SEG-09)
1. EL SISTEMA DEBERÁ ofrecer "Limpiar documento" con opciones seleccionables: metadatos e XMP, adjuntos, JavaScript, capas ocultas, comentarios, miniaturas, datos de formulario, contenido fuera de página.
2. EL SISTEMA DEBERÁ mostrar en la barra de estado indicadores cuando el documento contenga JavaScript, adjuntos, formularios o enlaces externos.

### R-SEG-05 Aislamiento — v1.0 (UC-SEG-08)
1. EL SISTEMA DEBERÁ analizar y renderizar los documentos en un proceso separado con privilegios reducidos, de modo que un fallo del motor no cierre la aplicación ni comprometa el sistema.
2. SI el proceso del motor falla, ENTONCES EL SISTEMA DEBERÁ reiniciarlo, conservar los cambios no guardados de otras pestañas e informar del documento afectado.

---

## 11. Conversión, OCR, comparación, impresión y CLI

### R-CON-01 Imágenes a PDF — MVP (UC-CON-01)
1. CUANDO el usuario arrastre o elija imágenes JPG, PNG o TIFF, EL SISTEMA DEBERÁ crear un PDF con una imagen por página, en el orden elegido.
2. EL SISTEMA DEBERÁ permitir elegir tamaño de página (A4, Carta, ajustado a la imagen), orientación y márgenes.
3. EL SISTEMA DEBERÁ respetar la orientación EXIF de las fotos.

### R-CON-02 Conversión v1.0 — v1.0 (UC-CON-02, 03, 07)
1. EL SISTEMA DEBERÁ crear un PDF en blanco y un PDF desde una imagen del portapapeles.
2. EL SISTEMA DEBERÁ exportar a TXT, Markdown e imágenes (ver R-EXT-02).

### R-CON-03 Conversión posterior — Posterior (UC-CON-04, 05, 06)
1. EL SISTEMA DEBERÁ convertir a PDF/A y validar la conformidad básica.
2. EL SISTEMA DEBERÁ convertir a escala de grises.
3. DONDE LibreOffice esté instalado, EL SISTEMA DEBERÁ convertir documentos de oficina a PDF.

### R-OCR-01 OCR local — v1.0 (UC-OCR-01, 02, 03)
1. CUANDO el usuario ejecute OCR, EL SISTEMA DEBERÁ agregar una capa de texto invisible alineada con el texto de la imagen, sin alterar la apariencia de la página.
2. EL SISTEMA DEBERÁ incluir español e inglés y permitir descargar otros idiomas solo con acción explícita del usuario.
3. EL SISTEMA DEBERÁ detectar las páginas sin texto y proponer OCR solo para ellas.
4. EL SISTEMA DEBERÁ procesar el OCR completamente en el equipo.

### R-OCR-02 Escaneo — Posterior (UC-OCR-04, UC-OCR-05)
1. EL SISTEMA DEBERÁ enderezar, recortar bordes, ajustar contraste y eliminar páginas en blanco.
2. EL SISTEMA DEBERÁ adquirir páginas desde escáneres del sistema (WIA/TWAIN, ImageCaptureCore, SANE).

### R-CMP-01 Comparación — Posterior (UC-CMP-01, 02, 03)
1. EL SISTEMA DEBERÁ comparar dos PDFs por texto y visualmente, mostrar las diferencias lado a lado con desplazamiento sincronizado y exportar un informe.

### R-IMP-01 Impresión — MVP (UC-IMP-01)
1. CUANDO el usuario imprima, EL SISTEMA DEBERÁ usar el diálogo de impresión del sistema con rango de páginas, copias, doble cara y escalado (ajustar, tamaño real, personalizado).
2. EL SISTEMA DEBERÁ imprimir el contenido original, sin aplicar los modos de color de visualización.

### R-IMP-02 Impresión avanzada — v1.0 / Posterior (UC-IMP-02, 03, 04)
1. (v1.0) EL SISTEMA DEBERÁ permitir imprimir con o sin anotaciones, y solo los campos de formulario.
2. (v1.0) EL SISTEMA DEBERÁ mostrar una vista previa fiel de impresión.
3. (Posterior) EL SISTEMA DEBERÁ imprimir varias páginas por hoja y folletos.

### R-LOT-01 Línea de comandos — v1.0, Could (UC-LOT-01, UC-LOT-04)
1. EL SISTEMA DEBERÁ ofrecer un ejecutable `demi` con subcomandos para las operaciones del núcleo: `info`, `merge`, `split`, `rotate`, `extract`, `render`, `text`, `compress`, `encrypt`, `decrypt`, `watermark`, `number`, `sanitize`, `redact`, `ocr`, `sign`, `verify`.
2. DONDE se indique `--json`, EL SISTEMA DEBERÁ producir salida JSON estable y documentada.
3. EL SISTEMA DEBERÁ devolver códigos de salida distintos de cero ante errores, con mensajes en stderr.

### R-LOT-02 Lotes y recetas — Posterior (UC-LOT-02, UC-LOT-03)
1. EL SISTEMA DEBERÁ aplicar una o varias acciones encadenadas a una lista de archivos o una carpeta, con patrón de nombre de salida.
2. EL SISTEMA DEBERÁ guardar y reutilizar secuencias de acciones ("recetas") en un formato común a la interfaz y la CLI.

---

## 12. Interfaz, identidad visual e integración con el sistema

### R-UI-01 Identidad visual — MVP
1. EL SISTEMA DEBERÁ seguir la dirección visual **editorial** ("papelería"): superficies tipo papel, trazos finos, marca con tipografía serif.
2. EL SISTEMA DEBERÁ usar la paleta oficial con estos roles:

| Token | Color | Rol |
|-------|-------|-----|
| `paper` | `#FFFFFF` | Hoja del documento, superficies elevadas |
| `canvas` | `#EEF4F0` | Fondo de ventana alrededor del documento |
| `chrome` | `#E1E4DB` | Barras de herramientas y estado |
| `sand` | `#E2D2C3` | Superficies cálidas, paneles, resaltado suave |
| `line` | `#BEC0BB` | Bordes y separadores |
| `sage-mid` | `#779699` | Bordes de controles, iconos inactivos |
| `stone` | `#92938E` | Texto/elementos deshabilitados (no para texto legible) |
| `mist` | `#93A8AD` | Detalles decorativos |
| `rose` | `#B9A1A1` | Firmas pendientes, marcas, anotaciones |
| `accent` | `#2B5E62` | Acción principal, selección, enlaces, foco |
| `ink` | `#204243` | Texto principal (sustituye al negro) |

3. EL SISTEMA DEBERÁ usar como máximo un botón de acción principal (`accent`) por vista.
4. EL SISTEMA DEBERÁ ofrecer un tema oscuro derivado de la paleta (con `ink` como base de fondo), definido en `design.md`.

### R-UI-02 Accesibilidad — MVP
1. Todo texto legible DEBERÁ tener un contraste ≥ 4,5:1 (≥ 3:1 para texto grande, iconos y bordes de controles), según WCAG 2.2 AA. En particular, `stone`, `mist`, `rose` y `sage-mid` NO DEBERÁN usarse para texto que deba leerse.
2. EL SISTEMA DEBERÁ ser utilizable completamente con teclado, con foco visible.
3. EL SISTEMA DEBERÁ exponer la interfaz a los lectores de pantalla (NVDA/Narrador, VoiceOver, Orca).
4. EL SISTEMA DEBERÁ respetar la escala de interfaz y la preferencia de reducir movimiento del sistema operativo.
5. EL SISTEMA NO DEBERÁ transmitir información solo con color (p. ej. estado de firma: icono + texto + color).

### R-UI-03 Idiomas — MVP (UC-SIS-06)
1. EL SISTEMA DEBERÁ mostrar la interfaz en español o inglés según el idioma del sistema, con opción de cambiarlo.
2. EL SISTEMA DEBERÁ externalizar todos los textos de la interfaz en archivos de traducción para permitir más idiomas.

### R-UI-04 Modo compacto, atajos y paleta de comandos — v1.0 (UC-SIS-04, UC-SIS-05)
1. EL SISTEMA DEBERÁ ofrecer un modo de interfaz compacto (paneles e iconos más pequeños) además del modo normal con controles amplios.
2. EL SISTEMA DEBERÁ ofrecer atajos de teclado para todas las acciones frecuentes y permitir reasignarlos.
3. CUANDO el usuario pulse Ctrl/Cmd+K, EL SISTEMA DEBERÁ abrir una paleta de comandos que busque cualquier acción por nombre en el idioma activo.

### R-SIS-01 Instalación y plataformas — MVP (UC-SIS-01)
1. EL SISTEMA DEBERÁ distribuirse como: instalador MSI y paquete winget (Windows 10/11 x64 y ARM64), DMG firmado y notarizado (macOS 12+, Apple Silicon e Intel), AppImage y .deb (Linux x86_64). (v1.0: .rpm y Flatpak.)
2. EL SISTEMA DEBERÁ funcionar en Linux bajo X11 y Wayland.
3. EL SISTEMA DEBERÁ ofrecer, durante la instalación o el primer uso, convertirse en el lector PDF predeterminado, sin forzarlo.

### R-SIS-02 Funcionamiento sin conexión y privacidad — MVP (UC-SIS-08)
1. EL SISTEMA DEBERÁ funcionar con todas sus funciones (salvo sello de tiempo, revocación y descargas de idiomas de OCR) sin conexión a internet.
2. EL SISTEMA NO DEBERÁ enviar telemetría ni datos de uso.
3. EL SISTEMA NO DEBERÁ realizar conexiones de red no iniciadas explícitamente por el usuario o por una opción que él activó.

### R-SIS-03 Actualizaciones y recuperación — v1.0 (UC-SIS-07, UC-SIS-09, UC-SIS-03)
1. DONDE el usuario active la comprobación de actualizaciones, EL SISTEMA DEBERÁ consultar la última versión sin enviar datos identificativos, y nunca actualizar sin confirmación.
2. MIENTRAS haya cambios sin guardar, EL SISTEMA DEBERÁ guardar periódicamente (cada ≤ 60 s) un estado de recuperación local cifrado.
3. CUANDO Demi se inicie tras un cierre inesperado, EL SISTEMA DEBERÁ ofrecer restaurar las pestañas y los cambios no guardados.
4. EL SISTEMA DEBERÁ permitir arrastrar páginas entre ventanas de Demi.

### R-SIS-04 Extensibilidad — Posterior (UC-SIS-10, UC-SIS-11)
1. EL SISTEMA DEBERÁ exponer las operaciones del núcleo como un catálogo de acciones con nombre, parámetros tipados y resultado, compartido por la interfaz y la CLI (requisito de arquitectura desde el MVP; sin API pública de plugins hasta "Posterior").
2. (Posterior) EL SISTEMA DEBERÁ cargar plugins aislados (p. ej. WebAssembly) con permisos explícitos declarados (leer documento, modificar documento, red, sistema de archivos) que el usuario apruebe.
3. (Posterior) EL SISTEMA DEBERÁ ofrecer un modo portátil con la configuración junto al ejecutable.

---

## 13. Requisitos no funcionales

### Rendimiento (equipo de referencia: CPU de 4 núcleos ~2020, 8 GB RAM, SSD)
- **R-NFR-REN-01:** Arranque en frío hasta ventana usable en ≤ 1 s; con un documento como argumento, primera página visible en ≤ 1,5 s.
- **R-NFR-REN-02:** Primera página visible de un PDF de 1 000 páginas de texto en ≤ 1 s tras abrirlo.
- **R-NFR-REN-03:** MIENTRAS el usuario se desplaza, EL SISTEMA DEBERÁ mantener la interfaz respondiendo (sin bloqueos > 100 ms) y mostrar marcadores de posición para páginas aún no renderizadas.
- **R-NFR-REN-04:** El uso de memoria de la caché de páginas DEBERÁ estar limitado (por defecto 512 MB, configurable).
- **R-NFR-REN-05:** Las operaciones de más de 1 s (unir, comprimir, OCR, búsqueda) DEBERÁN ejecutarse en segundo plano con progreso y botón Cancelar; cancelar no deberá dejar archivos parciales.
- **R-NFR-REN-06:** Unir 100 PDFs de 10 páginas DEBERÁ completarse en ≤ 10 s.

### Integridad y fiabilidad
- **R-NFR-FIA-01:** EL SISTEMA DEBERÁ guardar de forma atómica: escribir a un archivo temporal en el mismo directorio, verificar que se puede reabrir y reemplazar el original.
- **R-NFR-FIA-02:** EL SISTEMA DEBERÁ ofrecer "Guardar" (incremental cuando el documento esté firmado o el usuario lo prefiera) y "Guardar como" (reescritura completa optimizada).
- **R-NFR-FIA-03:** DONDE el usuario lo active, EL SISTEMA DEBERÁ conservar una copia de respaldo del original antes de sobrescribirlo.
- **R-NFR-FIA-04:** SI el archivo abierto cambió en disco desde que se abrió, ENTONCES EL SISTEMA DEBERÁ advertir antes de sobrescribirlo y ofrecer guardar como copia.
- **R-NFR-FIA-05:** SI el disco está lleno o no hay permisos, ENTONCES EL SISTEMA DEBERÁ informarlo, conservar el original intacto y mantener los cambios en memoria.
- **R-NFR-FIA-06:** Todo documento producido por Demi DEBERÁ pasar `qpdf --check` sin errores en la suite de pruebas.

### Seguridad
- **R-NFR-SEG-01:** Todo PDF se tratará como entrada no confiable. El analizador y las operaciones del núcleo DEBERÁN tener pruebas de fuzzing continuas en CI.
- **R-NFR-SEG-02:** Contraseñas, claves privadas y firmas guardadas NO DEBERÁN aparecer en registros, volcados de error ni archivos temporales sin cifrar.
- **R-NFR-SEG-03:** Las dependencias DEBERÁN auditarse en CI (`cargo audit` / `cargo deny`) y el binario de PDFium DEBERÁ actualizarse al menos trimestralmente o ante avisos de seguridad.

### Compatibilidad
- **R-NFR-COM-01:** EL SISTEMA DEBERÁ leer PDF 1.0 a 2.0, incluidos object streams, xref streams, archivos linealizados y con revisiones incrementales.
- **R-NFR-COM-02:** EL SISTEMA DEBERÁ manejar rutas con caracteres no ASCII y rutas largas de Windows (> 260 caracteres).
- **R-NFR-COM-03:** EL SISTEMA DEBERÁ funcionar correctamente con pantallas HiDPI y al mover ventanas entre monitores de distinta densidad.
- **R-NFR-COM-04:** EL SISTEMA DEBERÁ mostrar y seleccionar correctamente texto de derecha a izquierda y vertical (CJK); su edición queda fuera de N1.

### Calidad y mantenimiento
- **R-NFR-CAL-01:** El crate núcleo NO DEBERÁ depender de la interfaz gráfica y DEBERÁ ser utilizable de forma independiente (licencia MIT OR Apache-2.0).
- **R-NFR-CAL-02:** Toda dependencia DEBERÁ tener licencia compatible con GPL-3.0 (aplicación) y con MIT/Apache-2.0 (núcleo); se prohíben dependencias AGPL o comerciales "freemium" en ambos.
- **R-NFR-CAL-03:** El proyecto DEBERÁ compilarse desde el código fuente con un solo comando documentado en las tres plataformas.
- **R-NFR-CAL-04:** CI DEBERÁ ejecutar en Windows, macOS y Linux: compilación, `clippy`, formato, tests unitarios, tests de regresión visual sobre un corpus de PDFs y la suite de compatibilidad de salida.
- **R-NFR-CAL-05:** Cada requisito de este documento DEBERÁ tener al menos una prueba automatizada o un caso de prueba manual trazable por su ID.

---

## 14. Casos borde obligatorios (criterios de prueba transversales)

EL SISTEMA DEBERÁ comportarse de forma definida (abrir, abrir con aviso o rechazar con mensaje claro, sin bloquearse ni cerrarse) ante:

1. Archivo de 0 bytes, truncado, con xref roto o con basura antes de `%PDF`.
2. Documentos > 2 GB o > 10 000 páginas; páginas de tamaño A0 o mayor.
3. Páginas con > 100 000 operadores vectoriales.
4. Fuentes no incrustadas, Type3, CID sin `ToUnicode`.
5. Texto RTL y vertical.
6. Documentos con contraseña de permisos pero sin contraseña de apertura.
7. Archivo en unidad de red o carpeta sincronizada que cambia mientras está abierto.
8. Rutas con acentos, emojis o > 260 caracteres.
9. Disco lleno o sin permisos al guardar.
10. Formularios XFA y campos con JavaScript de validación.
11. Firmas con certificados caducados, revocados o sin conexión para comprobar revocación.
12. Edición de documentos firmados o certificados.
13. Monitores HiDPI y cambio de monitor en caliente.

---

## 15. Fuera de alcance

- Funciones de IA integradas (resúmenes, chat, generación de texto). Podrán existir solo como plugins de terceros.
- Nube propia, cuentas de usuario o sincronización obligatoria.
- Colaboración en tiempo real.
- Maquetación tipo InDesign o edición de nivel procesador de textos.
- Aplicaciones móviles (al menos hasta después de v1.0).
- XFA dinámico completo.
- Firma en la nube de prestadores específicos (salvo vía plugin).

---

## 16. Resumen por versión

| Área | MVP | v1.0 | Posterior |
|------|-----|------|-----------|
| Visualización | R-VIS-01…06 | R-VIS-07…11 | R-VIS-12 |
| Navegación/búsqueda | R-NAV-01…04, 06 | R-NAV-05, 07 | R-NAV-08 |
| Extracción | R-EXT-01 | R-EXT-02 | R-EXT-03 |
| Anotación | R-ANO-01…04 | R-ANO-05, 06 | R-ANO-07 |
| Edición | R-EDI-01, 02 (N0), 03 (N1) | R-EDI-04 (N2), 06 | R-EDI-05 (N3), 07 |
| Páginas | R-PAG-01…04 | R-PAG-05 | R-PAG-06 |
| Formularios | R-FOR-01…03 | R-FOR-04 | R-FOR-05 |
| Firmas | R-FIR-01…03 | R-FIR-04…06 | R-FIR-07 |
| Seguridad | R-SEG-01 | R-SEG-02…05 | — |
| Conversión/OCR/impresión | R-CON-01, R-IMP-01 | R-CON-02, R-OCR-01, R-IMP-02 | R-CON-03, R-OCR-02, R-CMP-01 |
| CLI/lotes | — | R-LOT-01 (Could) | R-LOT-02 |
| Interfaz/sistema | R-UI-01…03, R-SIS-01, 02 | R-UI-04, R-SIS-03 | R-SIS-04 (plugins) |
| No funcionales | Todos (§13) | — | — |

---

## 17. Trazabilidad

Todos los casos de uso `UC-*` de `descubrimiento.md` están cubiertos por algún `R-*` de este documento o figuran en §15 (fuera de alcance). La matriz completa UC → R se mantendrá en `design.md` y en las pruebas.
