//! `s2-worker`: proceso aislado que abre el PDF y escribe teselas en memoria compartida.
//!
//! Uso: `s2-worker <socket-id> <token> <shm> <pdf> <pdfium-dir>`

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::type_complexity)]

use std::fs::OpenOptions;
use std::path::PathBuf;
use std::time::Instant;

use demi_engine::pdfium_lib;
use demi_engine::pdfium_render::prelude::*;
use demi_spike_s2::{Request, Response, SLOT_BYTES, TILE, checksum, recv, send, socket_name};
use interprocess::local_socket::{Stream, prelude::*};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, sock_id, token, shm, pdf, lib] = args.as_slice() else {
        eprintln!("uso: s2-worker <socket-id> <token> <shm> <pdf> <pdfium-dir>");
        std::process::exit(2);
    };
    let token: u64 = token.parse().expect("token");

    let mut conn = Stream::connect(socket_name(sock_id).expect("nombre")).expect("conectar");

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(shm)
        .expect("shm");
    // SAFETY: archivo creado por la app solo para este worker; nadie más lo trunca.
    #[allow(unsafe_code)]
    let mut map = unsafe { memmap2::MmapMut::map_mut(&file).expect("mmap") };

    let pdfium = pdfium_lib::bind(&PathBuf::from(lib)).expect("PDFium");
    let doc = pdfium.load_pdf_from_file(pdf, None).expect("abrir PDF");
    let pages = doc.pages();
    send(
        &mut conn,
        &Response::Hello {
            token,
            pages: pages.len() as u32,
        },
    )
    .expect("hello");

    // Caché de la última página renderizada: (página, ancho) -> (rgba, ancho, alto).
    let mut cache: Option<((u32, u32), Vec<u8>, u32, u32)> = None;

    loop {
        let req: Request = match recv(&mut conn) {
            Ok(r) => r,
            Err(_) => return, // la app cerró el canal
        };
        match req {
            Request::Shutdown => return,
            Request::Crash => std::process::abort(),
            Request::Tile {
                id,
                page,
                tx,
                ty,
                page_width_px,
                slot,
            } => {
                let mut render_us = 0;
                if cache.as_ref().map(|c| c.0) != Some((page, page_width_px)) {
                    let t = Instant::now();
                    let Ok(p) = pages.get(page as PdfPageIndex) else {
                        let _ = send(
                            &mut conn,
                            &Response::Error {
                                id,
                                message: "página".into(),
                            },
                        );
                        continue;
                    };
                    let bmp = p
                        .render_with_config(
                            &PdfRenderConfig::new().set_target_width(page_width_px as i32),
                        )
                        .expect("render");
                    cache = Some((
                        (page, page_width_px),
                        bmp.as_raw_bytes(),
                        bmp.width() as u32,
                        bmp.height() as u32,
                    ));
                    render_us = t.elapsed().as_micros() as u64;
                }
                let t = Instant::now();
                let (_, rgba, pw, ph) = cache.as_ref().expect("caché");
                let x0 = tx * TILE;
                let y0 = ty * TILE;
                let w = TILE.min(pw.saturating_sub(x0));
                let h = TILE.min(ph.saturating_sub(y0));
                let base = slot as usize * SLOT_BYTES;
                let dst = &mut map[base..base + SLOT_BYTES];
                for row in 0..h {
                    let src_off = (((y0 + row) * pw + x0) * 4) as usize;
                    let dst_off = (row * TILE * 4) as usize;
                    let len = (w * 4) as usize;
                    dst[dst_off..dst_off + len].copy_from_slice(&rgba[src_off..src_off + len]);
                }
                let sum = checksum(&dst[..(TILE * h * 4) as usize]);
                let copy_us = t.elapsed().as_micros() as u64;
                if send(
                    &mut conn,
                    &Response::TileReady {
                        id,
                        slot,
                        width: w,
                        height: h,
                        render_us,
                        copy_us,
                        checksum: sum,
                    },
                )
                .is_err()
                {
                    return;
                }
            }
        }
    }
}
