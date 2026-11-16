# Política de seguridad

Demi procesa documentos PDF que pueden venir de cualquier parte, así que tomamos en serio los fallos de seguridad.

## Reportar una vulnerabilidad

**No abras un issue público.** Usa el reporte privado de GitHub:
pestaña **Security** del repositorio → **Report a vulnerability**.

Incluye, si puedes:
- versión de Demi y sistema operativo;
- un PDF o pasos mínimos para reproducir (puedes adjuntarlo de forma privada);
- impacto observado (cierre inesperado, lectura o escritura fuera de límites, ejecución de código, fuga de datos…).

Responderemos en un plazo de 7 días y coordinaremos la publicación del arreglo contigo.

## Alcance

- `demi-worker` y todo el código que analiza o renderiza PDFs.
- El protocolo app ↔ worker (`demi-ipc`).
- El almacenamiento de firmas y contraseñas (`demi-store`, `demi-sign`).
- Vulnerabilidades en PDFium que afecten a Demi: las reportamos también al proyecto PDFium.

## Versiones con soporte

Mientras Demi esté en 0.x, solo la última versión recibe correcciones de seguridad.
