# Spike S1 · Cómo probar el visor en Slint

Prototipo desechable para decidir si Slint sirve para el visor de Demi (tasks.md S1).

## Abrirlo

1. Descomprime la carpeta (deja `demi-s1.exe` y `pdfium.dll` juntos).
2. Doble clic en `demi-s1.exe`: abre un PDF de prueba de **500 páginas** generado al vuelo.
3. Para probar con un PDF tuyo, arrástralo sobre `demi-s1.exe` o ejecuta `demi-s1.exe "C:\ruta\archivo.pdf"`.

Arriba a la derecha verás: **fps · peor frame · ms de render por página · páginas en memoria**.

## Qué probar (anota lo que veas)

### A. Fluidez (objetivo: 60 fps, peor frame < 33 ms)
- [ ] Desplázate rápido con la rueda varios segundos: ¿cuántos **fps** y qué **peor frame** marca?
- [ ] Arrastra la barra de desplazamiento hasta la página ~400: ¿aparecen las páginas sin tirones?
- [ ] ¿Las páginas aparecen en blanco un instante antes de dibujarse? ¿Cuánto tarda?
- [ ] Repite con un PDF real pesado tuyo (escaneado, con imágenes, planos).

### B. Zoom
- [ ] **Ctrl + rueda** sobre una página acerca/aleja (también **Ctrl +**, **Ctrl -**, **Ctrl 0**).
- [ ] Tras el zoom, el texto se vuelve nítido en menos de medio segundo.
- [ ] Botones **Acercar / Alejar / 100 %**.

### C. Anotaciones (capa vectorial)
- [ ] En las 10 primeras páginas hay 5 rectángulos amarillos: **arrástralos** con el ratón.
- [ ] Se mueven suaves, sin retraso, y siguen en su sitio al hacer zoom.

### D. Lector de pantalla (NVDA)
1. Instala [NVDA](https://www.nvaccess.org/) (gratis) y ábrelo.
2. Con el visor abierto:
   - [ ] **Tab** recorre los botones y NVDA los anuncia ("Alejar, botón", "Acercar, botón"…).
   - [ ] Con el cursor de revisión de NVDA (**Insert + Numpad 8** / **Numpad 7/9**) o el modo de navegación, ¿lee "Página 1. Demi - pagina 1 de 500 Linea 1…"?
   - [ ] ¿Anuncia las anotaciones como "Anotación (arrastrable), botón"?
   - (Opcional) Prueba también con el **Narrador** de Windows (Ctrl + Win + Enter).

### E. Otros
- [ ] Uso de memoria en el Administrador de tareas tras recorrer todo el documento: ______ MB
- [ ] ¿Algo raro? (parpadeos, cierres, ventana en blanco, fuentes borrosas en pantalla HiDPI…)

## Envío de resultados

Responde en el chat con lo que marcaste y los números de fps / peor frame / memoria.
Si algo falla, una captura de pantalla ayuda mucho.
