//! Spike S1 de Demi: visor de PDF en Slint (código desechable).
//!
//! Mide si Slint permite: desplazamiento continuo fluido (objetivo 60 fps) con un PDF
//! de 500 páginas, zoom con Ctrl+rueda, una capa vectorial de 50 anotaciones
//! arrastrables y un árbol de accesibilidad con el texto de cada página.
//!
//! Uso: `demi-s1 [archivo.pdf]`. Sin argumento genera un PDF de 500 páginas.
//! PDFium se busca en `DEMI_PDFIUM_DIR` o junto al ejecutable.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod sample;

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use demi_engine::pdfium_lib;
use demi_engine::pdfium_render::prelude::*;
use slint::{Image, Model, ModelRc, Rgba8Pixel, SharedPixelBuffer, VecModel};

slint::include_modules!();

/// Separación vertical entre páginas (debe coincidir con visor.slint: 8 px arriba + 8 abajo).
const GAP_PX: f32 = 16.0;
/// Páginas extra que se mantienen por encima y por debajo de las visibles.
const MARGIN_PAGES: usize = 2;
/// Páginas renderizadas que se conservan en caché alrededor de las visibles.
const CACHE_PAGES: usize = 6;

struct RenderRequest {
    pages: Vec<usize>,
    zoom: f32,
    scale: f32,
}

struct Rendered {
    page: usize,
    zoom: f32,
    buffer: SharedPixelBuffer<Rgba8Pixel>,
    text: Option<String>,
    millis: f32,
}

#[derive(Default)]
struct Stats {
    frames: u32,
    worst_frame: Duration,
    last_frame: Option<Instant>,
    render_ms_total: f32,
    render_count: u32,
}

/// Estado del visor en el hilo de la interfaz.
struct Viewer {
    sizes: Vec<(f32, f32)>,
    zoom: f32,
    /// Posición superior de cada página (px lógicos) al zoom actual; último = altura total.
    offsets: Vec<f32>,
    images: HashMap<usize, (Image, f32)>, // imagen y zoom con el que se renderizó
    texts: Vec<Option<String>>,
    shown: Vec<usize>,
    model: Rc<VecModel<PageData>>,
}

impl Viewer {
    fn new(sizes: Vec<(f32, f32)>, model: Rc<VecModel<PageData>>) -> Self {
        let n = sizes.len();
        let mut v = Self {
            sizes,
            zoom: 1.0,
            offsets: Vec::new(),
            images: HashMap::new(),
            texts: vec![None; n],
            shown: Vec::new(),
            model,
        };
        v.relayout();
        v
    }

    fn relayout(&mut self) {
        self.offsets.clear();
        let mut y = 0.0;
        for &(_, h) in &self.sizes {
            self.offsets.push(y);
            y += h * self.zoom + GAP_PX;
        }
        self.offsets.push(y);
    }

    fn total_height(&self) -> f32 {
        self.offsets.last().copied().unwrap_or(0.0)
    }

    fn max_width(&self) -> f32 {
        self.sizes.iter().map(|&(w, _)| w).fold(0.0, f32::max) * self.zoom
    }

    /// Páginas que intersectan `[top, bottom]`.
    fn visible(&self, top: f32, bottom: f32) -> (usize, usize) {
        let n = self.sizes.len();
        if n == 0 {
            return (0, 0);
        }
        let first = self.offsets[1..]
            .partition_point(|&end| end < top)
            .min(n - 1);
        let last = self.offsets[..n]
            .partition_point(|&start| start <= bottom)
            .saturating_sub(1);
        (first, last.max(first))
    }

    fn row(&self, i: usize) -> PageData {
        let (w, h) = self.sizes[i];
        PageData {
            index: i as i32,
            y: self.offsets[i],
            image: self
                .images
                .get(&i)
                .map(|(img, _)| img.clone())
                .unwrap_or_default(),
            width_pt: w,
            height_pt: h,
            text: self.texts[i]
                .clone()
                .unwrap_or_else(|| "texto pendiente".into())
                .into(),
            annots: annotations_for(i, w, h),
        }
    }

    /// Actualiza el modelo con las páginas visibles y devuelve las que hay que renderizar.
    fn update(&mut self, top: f32, height: f32) -> Vec<usize> {
        let (first, last) = self.visible(top, top + height);
        let n = self.sizes.len();
        let lo = first.saturating_sub(MARGIN_PAGES);
        let hi = (last + MARGIN_PAGES).min(n.saturating_sub(1));
        let wanted: Vec<usize> = (lo..=hi).collect();
        if wanted != self.shown {
            self.shown = wanted;
            let rows: Vec<PageData> = self.shown.iter().map(|&i| self.row(i)).collect();
            self.model.set_vec(rows);
        }
        // Liberar imágenes lejanas.
        let keep_lo = lo.saturating_sub(CACHE_PAGES);
        let keep_hi = hi + CACHE_PAGES;
        self.images.retain(|&i, _| i >= keep_lo && i <= keep_hi);

        // Visibles primero, luego el margen.
        let mut order: Vec<usize> = (first..=last).collect();
        order.extend((lo..first).rev().chain(last + 1..=hi));
        order
            .into_iter()
            .filter(|i| !self.images.get(i).is_some_and(|(_, z)| *z == self.zoom))
            .collect()
    }

    fn set_rendered(&mut self, page: usize, image: Image, zoom: f32, text: Option<String>) {
        self.images.insert(page, (image, zoom));
        if text.is_some() {
            self.texts[page] = text;
        }
        if let Some(pos) = self.shown.iter().position(|&i| i == page) {
            self.model.set_row_data(pos, self.row(page));
        }
    }
}

fn main() {
    let path: PathBuf = match std::env::args_os().nth(1) {
        Some(p) => PathBuf::from(p),
        None => {
            let p = std::env::temp_dir().join("demi-s1-500-paginas.pdf");
            if !p.exists() {
                std::fs::write(&p, sample::generate(500)).expect("escribir PDF de prueba");
            }
            p
        }
    };
    let doc_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

    let lib_dir = pdfium_lib::default_library_dir().expect("directorio de PDFium");

    // Hilo de render: dueño de PDFium y del documento.
    let (req_tx, req_rx) = mpsc::channel::<RenderRequest>();
    let (sizes_tx, sizes_rx) = mpsc::channel::<Result<Vec<(f32, f32)>, String>>();
    let (done_tx, done_rx) = mpsc::channel::<Rendered>();
    std::thread::spawn(move || render_thread(lib_dir, path, req_rx, sizes_tx, done_tx));

    let sizes = match sizes_rx.recv().expect("hilo de render") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("No se pudo abrir el documento: {e}");
            std::process::exit(1);
        }
    };

    let ui = AppWindow::new().expect("crear ventana");
    ui.set_doc_name(format!("{doc_name} · {} páginas", sizes.len()).into());

    let model = Rc::new(VecModel::<PageData>::default());
    ui.set_pages(ModelRc::from(model.clone()));
    let viewer = Rc::new(RefCell::new(Viewer::new(sizes, model)));
    ui.set_total_height(viewer.borrow().total_height());
    ui.set_max_page_width(viewer.borrow().max_width());

    let stats = Rc::new(RefCell::new(Stats::default()));

    let refresh = {
        let ui_weak = ui.as_weak();
        let viewer = viewer.clone();
        move || {
            let Some(ui) = ui_weak.upgrade() else { return };
            let mut v = viewer.borrow_mut();
            let todo = v.update(-ui.get_scroll_y(), ui.get_visible_height());
            // El hilo de render solo atiende la petición más reciente: se envía siempre
            // la lista completa de páginas que aún no tienen imagen al zoom actual.
            if !todo.is_empty() {
                let _ = req_tx.send(RenderRequest {
                    pages: todo,
                    zoom: v.zoom,
                    scale: ui.window().scale_factor(),
                });
            }
        }
    };
    let refresh = Rc::new(refresh);

    {
        let r = refresh.clone();
        ui.on_viewport_changed(move || r());
    }
    {
        let r = refresh.clone();
        let ui_weak = ui.as_weak();
        let viewer = viewer.clone();
        ui.on_zoom_requested(move |z| {
            let Some(ui) = ui_weak.upgrade() else { return };
            let z = z.clamp(0.1, 8.0);
            {
                let mut v = viewer.borrow_mut();
                if (z - v.zoom).abs() < f32::EPSILON {
                    return;
                }
                // Mantener la misma posición relativa del documento.
                let ratio = z / v.zoom;
                v.zoom = z;
                v.relayout();
                v.shown.clear(); // forzar que se reconstruyan las filas con las nuevas posiciones
                ui.set_zoom(z);
                ui.set_total_height(v.total_height());
                ui.set_max_page_width(v.max_width());
                ui.set_scroll_y(ui.get_scroll_y() * ratio);
            }
            r();
        });
    }

    // Recoge las páginas renderizadas.
    let poll = slint::Timer::default();
    {
        let viewer = viewer.clone();
        let stats = stats.clone();
        poll.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(8),
            move || {
                while let Ok(r) = done_rx.try_recv() {
                    let mut v = viewer.borrow_mut();
                    if (r.zoom - v.zoom).abs() > f32::EPSILON {
                        continue; // render de un zoom anterior
                    }
                    v.set_rendered(r.page, Image::from_rgba8(r.buffer), r.zoom, r.text);
                    let mut s = stats.borrow_mut();
                    s.render_ms_total += r.millis;
                    s.render_count += 1;
                }
            },
        );
    }

    // Medición de fluidez: frames por segundo y peor intervalo entre frames.
    {
        let stats = stats.clone();
        ui.window()
            .set_rendering_notifier(move |state, _| {
                if let slint::RenderingState::AfterRendering = state {
                    let now = Instant::now();
                    let mut s = stats.borrow_mut();
                    if let Some(prev) = s.last_frame {
                        let dt = now - prev;
                        if dt < Duration::from_millis(250) && dt > s.worst_frame {
                            s.worst_frame = dt;
                        }
                    }
                    s.last_frame = Some(now);
                    s.frames += 1;
                }
            })
            .ok();
    }
    let stats_timer = slint::Timer::default();
    {
        let stats = stats.clone();
        let ui_weak = ui.as_weak();
        let viewer = viewer.clone();
        stats_timer.start(
            slint::TimerMode::Repeated,
            Duration::from_secs(1),
            move || {
                let Some(ui) = ui_weak.upgrade() else { return };
                let mut s = stats.borrow_mut();
                let avg = if s.render_count > 0 {
                    s.render_ms_total / s.render_count as f32
                } else {
                    0.0
                };
                ui.set_stats(
                    format!(
                        "{} fps · peor frame {} ms · render {:.0} ms/pág · {} págs en memoria",
                        s.frames,
                        s.worst_frame.as_millis(),
                        avg,
                        viewer.borrow().images.len()
                    )
                    .into(),
                );
                s.frames = 0;
                s.worst_frame = Duration::ZERO;
            },
        );
    }

    // Primer render cuando la ventana ya tiene tamaño.
    {
        let r = refresh.clone();
        slint::Timer::single_shot(Duration::from_millis(50), move || r());
    }

    ui.run().expect("ejecutar ventana");
}

/// 5 anotaciones en cada una de las 10 primeras páginas (50 en total).
fn annotations_for(page: usize, w: f32, h: f32) -> ModelRc<Annot> {
    if page >= 10 {
        return ModelRc::default();
    }
    let v: Vec<Annot> = (0..5)
        .map(|k| {
            let k = k as f32;
            Annot {
                x: w * (0.1 + 0.15 * k),
                y: h * (0.15 + 0.12 * k),
                w: w * 0.25,
                h: 18.0,
            }
        })
        .collect();
    ModelRc::new(VecModel::from(v))
}

fn render_thread(
    lib_dir: PathBuf,
    path: PathBuf,
    req_rx: mpsc::Receiver<RenderRequest>,
    sizes_tx: mpsc::Sender<Result<Vec<(f32, f32)>, String>>,
    done_tx: mpsc::Sender<Rendered>,
) {
    let pdfium = match pdfium_lib::bind(&lib_dir) {
        Ok(p) => p,
        Err(e) => {
            let _ = sizes_tx.send(Err(format!(
                "PDFium no encontrado en {} ({e:?})",
                lib_dir.display()
            )));
            return;
        }
    };
    let doc = match pdfium.load_pdf_from_file(&path, None) {
        Ok(d) => d,
        Err(e) => {
            let _ = sizes_tx.send(Err(format!("{e:?}")));
            return;
        }
    };
    let pages = doc.pages();
    let sizes: Vec<(f32, f32)> = pages
        .iter()
        .map(|p| (p.width().value, p.height().value))
        .collect();
    let _ = sizes_tx.send(Ok(sizes));
    let mut text_sent = vec![false; pages.len() as usize];

    let mut pending: Option<RenderRequest> = None;
    'outer: loop {
        let mut req = match pending.take() {
            Some(r) => r,
            None => match req_rx.recv() {
                Ok(r) => r,
                Err(_) => return,
            },
        };
        // Quedarse solo con la petición más reciente.
        while let Ok(newer) = req_rx.try_recv() {
            req = newer;
        }
        for &i in &req.pages {
            // Si llegó una petición nueva, abandonar esta (cancelación).
            if let Ok(newer) = req_rx.try_recv() {
                pending = Some(newer);
                continue 'outer;
            }
            let Ok(page) = pages.get(i as PdfPageIndex) else {
                continue;
            };
            let start = Instant::now();
            let target_w = (page.width().value * req.zoom * req.scale).round().max(1.0) as i32;
            let Ok(bitmap) =
                page.render_with_config(&PdfRenderConfig::new().set_target_width(target_w))
            else {
                continue;
            };
            let (w, h) = (bitmap.width() as u32, bitmap.height() as u32);
            // pdfium-render pide a PDFium el orden RGBA por defecto (FPDF_REVERSE_BYTE_ORDER).
            let buffer =
                SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(&bitmap.as_raw_bytes(), w, h);
            let text = if text_sent[i] {
                None
            } else {
                text_sent[i] = true;
                Some(
                    page.text()
                        .map(|t| t.all().chars().take(2000).collect::<String>())
                        .unwrap_or_default(),
                )
            };
            let millis = start.elapsed().as_secs_f32() * 1000.0;
            if done_tx
                .send(Rendered {
                    page: i,
                    zoom: req.zoom,
                    buffer,
                    text,
                    millis,
                })
                .is_err()
            {
                return;
            }
        }
    }
}
