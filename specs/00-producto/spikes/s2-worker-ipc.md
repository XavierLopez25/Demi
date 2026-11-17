# Spike S2 · Worker aislado + IPC con memoria compartida — informe

> **Tarea:** `tasks.md` S2 · **Decisiones afectadas:** ADR-05 (worker aislado), `design.md` §5 · **Estado:** cerrado (2026-11-17) — **se confirma ADR-05 (worker aislado)**
> **Código:** `spikes/s2-worker-ipc/` (desechable) · **Medición multiplataforma:** workflow `Spike S2 · worker + IPC`

## Pregunta

¿Separar PDFium en su propio proceso (`demi-worker`) añade una latencia perceptible?

**Criterio de éxito:** tesela de 512² disponible en la app < 16 ms después del render; arranque del worker < 150 ms; si el worker muere, la app sigue viva.

## Qué se construyó

- `s2-worker`: proceso que carga PDFium, abre el PDF y renderiza teselas de 512×512 (RGBA) en un **anillo de 16 ranuras en memoria compartida** (archivo mapeado con `memmap2`, 16 MiB). Cachea la última página renderizada para servir sus 6 teselas con un solo render.
- Canal de control por **socket local** (`interprocess`: espacio de nombres en Windows/Linux, archivo en macOS) con mensajes `postcard` precedidos de su longitud (máximo 1 MiB: entrada no confiable).
- **Handshake con token** de un solo uso: el worker solo es aceptado si devuelve el token recibido al lanzarlo.
- `s2-bench`: hace de app; mide arranque, sobrecoste de IPC por tesela (secuencial y con 8 peticiones en vuelo), verifica cada tesela con una suma de comprobación del lado de la app y simula fallos (kill desde fuera, `abort()` dentro del worker).

## Resultados (workflow `Spike S2 · worker + IPC`, runners de GitHub, release)

Valores p50 (p95 entre paréntesis).

| Medida | Linux x64 | Windows x64 | macOS arm64 | macOS Intel | Objetivo |
|---|---|---|---|---|---|
| Arranque hasta conectar | 0,8 ms (1,0) | 5,1 ms (9,5) | 7,3 ms (10,9) | 10,3 ms (20,2) | |
| **Arranque hasta listo** (PDFium + PDF abierto) | 1,3 ms (1,6) | 11,5 ms (16,8) | 12,9 ms (47,2) | 16,0 ms (39,5) | < 150 ms ✅ |
| **Sobrecoste IPC por tesela** | 0,04 ms (0,07) | 0,05 ms (0,26) | 0,11 ms (0,25) | 0,15 ms (0,46) | < 16 ms ✅ |
| Render de página A4 a 1024 px | 5,3 ms | 4,3 ms | 2,0 ms | 5,0 ms | |
| Primer render de la sesión (máx.) | 1 340 ms | 44 ms | 489 ms | 789 ms | ver hallazgo 2 |
| Teselas/s con 8 peticiones en vuelo | 542 | 444 | 390 | 425 | |

Fallos simulados (en todas las plataformas **la app sigue viva** ✅):

| Fallo | Linux x64 | Windows x64 | macOS arm64 | macOS Intel |
|---|---|---|---|---|
| kill del proceso: detección / reinicio + primera tesela | 0,6 / 10 ms | 1,1 / 20 ms | 1,1 / 62 ms | 1,6 / 85 ms |
| `abort()` en el worker: detección / reinicio + primera tesela | 346 / 12 ms | 249 / 20 ms | 0,3 / 65 ms | 5,9 / 75 ms |

Medición previa en el contenedor de desarrollo (Linux): arranque 2,3 ms, sobrecoste 0,17 ms, 486 teselas/s.

## Decisión

**Se confirma ADR-05:** el worker aislado cumple con mucho margen en las cuatro plataformas (arranque ≤ 47 ms en p95 frente a 150; sobrecoste de IPC ≤ 0,5 ms frente a 16) y la app sobrevive a la muerte del worker. No hace falta el plan B (píxeles comprimidos por el socket ni *pool* precalentado por rendimiento).

## Hallazgos

1. El coste de IPC (≤ 0,2 ms) es despreciable frente al render (2–5 ms por página A4): **el worker aislado no penaliza la fluidez**.
2. **El primer render de cada worker es lento** (hasta 1,3 s): PDFium carga fuentes del sistema la primera vez. → En T2.4 el worker hará un render de calentamiento al arrancar, antes de anunciarse como listo, y la app lanzará el worker al abrir la ventana, no al abrir el primer documento.
3. **Tras un `abort()`, Linux y Windows tardan ~250–350 ms en cerrar el proceso** porque intervienen sus gestores de fallos (core dumps / Windows Error Reporting). → En T2.4 el worker desactivará el volcado de memoria (`RLIMIT_CORE = 0`) y el cuadro de error de Windows (`SetErrorMode`/`WerSetFlags`): además de acelerar la detección, evita que la usuaria vea un diálogo de "el programa dejó de funcionar" y que queden volcados con contenido de sus documentos en disco.
4. La muerte del worker se detecta por el cierre del socket, sin *heartbeat*; el *heartbeat* de `design.md` §5.3 sigue haciendo falta para un worker **colgado**.
5. Pasar los píxeles por memoria compartida evita copiar 1 MiB por tesela por el socket. En producción conviene memoria compartida anónima/nombrada en lugar de un archivo temporal (en Windows el archivo puede tocar disco).
