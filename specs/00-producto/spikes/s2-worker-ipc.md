# Spike S2 · Worker aislado + IPC con memoria compartida — informe

> **Tarea:** `tasks.md` S2 · **Decisiones afectadas:** ADR-05 (worker aislado), `design.md` §5 · **Estado:** en curso — falta la medición en Windows y macOS (CI)
> **Código:** `spikes/s2-worker-ipc/` (desechable) · **Medición multiplataforma:** workflow `Spike S2 · worker + IPC`

## Pregunta

¿Separar PDFium en su propio proceso (`demi-worker`) añade una latencia perceptible?

**Criterio de éxito:** tesela de 512² disponible en la app < 16 ms después del render; arranque del worker < 150 ms; si el worker muere, la app sigue viva.

## Qué se construyó

- `s2-worker`: proceso que carga PDFium, abre el PDF y renderiza teselas de 512×512 (RGBA) en un **anillo de 16 ranuras en memoria compartida** (archivo mapeado con `memmap2`, 16 MiB). Cachea la última página renderizada para servir sus 6 teselas con un solo render.
- Canal de control por **socket local** (`interprocess`: espacio de nombres en Windows/Linux, archivo en macOS) con mensajes `postcard` precedidos de su longitud (máximo 1 MiB: entrada no confiable).
- **Handshake con token** de un solo uso: el worker solo es aceptado si devuelve el token recibido al lanzarlo.
- `s2-bench`: hace de app; mide arranque, sobrecoste de IPC por tesela (secuencial y con 8 peticiones en vuelo), verifica cada tesela con una suma de comprobación del lado de la app y simula fallos (kill desde fuera, `abort()` dentro del worker).

## Resultados en Linux x64 (contenedor de desarrollo, release)

| Medida | p50 | p95 | máx | Objetivo |
|---|---|---|---|---|
| Arranque hasta conectar | 1,69 ms | 2,33 ms | 2,33 ms | |
| Arranque hasta listo (PDFium + PDF abierto) | 2,32 ms | 3,21 ms | 3,21 ms | < 150 ms ✅ |
| **Sobrecoste IPC por tesela** (ida y vuelta − render − copia) | **0,17 ms** | 0,27 ms | 3,50 ms | < 16 ms ✅ |
| Tesela completa (incl. render de página cuando toca) | 1,79 ms | 6,04 ms | 16,18 ms | |
| Render de página A4 a 1024 px (en el worker) | 3,78 ms | 7,22 ms | 14,12 ms | |

Con 8 peticiones en vuelo: **486 teselas/s**.

| Fallo simulado | Detección | Reinicio hasta listo (incl. primera tesela) | La app sigue viva |
|---|---|---|---|
| kill del proceso | 0,6 ms | 14,3 ms | ✅ |
| `abort()` dentro del worker | 0,6 ms | 14,7 ms | ✅ |

## Resultados en Windows y macOS

⏳ Se completan con el resumen del workflow `Spike S2 · worker + IPC` (ver el PR).

## Hallazgos

1. El coste de IPC (≈ 0,2 ms) es despreciable frente al render (≈ 4 ms por página A4): **el worker aislado no penaliza la fluidez**.
2. La muerte del worker se detecta al instante por el cierre del socket (sin necesidad de *heartbeat* para ese caso); el *heartbeat* de `design.md` §5.3 sigue haciendo falta para el caso de un worker **colgado**.
3. Pasar los píxeles por memoria compartida evita copiar 1 MiB por tesela a través del socket.
