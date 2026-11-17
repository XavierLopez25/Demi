//! `s2-bench`: hace de app, lanza `s2-worker` y mide (design.md §14, S2):
//! 1. Arranque del worker (hasta conectar y hasta estar listo con el PDF abierto).
//! 2. Sobrecoste de IPC por tesela (ida y vuelta − render − copia), secuencial y en paralelo.
//! 3. Recuperación cuando el worker muere (kill desde fuera y abort desde dentro).
//!
//! Criterios: tesela disponible < 16 ms tras el render; arranque < 150 ms; la app sobrevive.
//! Uso: `s2-bench` (PDFium en `DEMI_PDFIUM_DIR` o `vendor/pdfium/<plataforma>/…`).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use demi_engine::pdfium_lib;
use demi_spike_s2::{
    Request, Response, SLOT_BYTES, SLOTS, TILE, checksum, recv, send, socket_name,
};
use interprocess::local_socket::{ListenerOptions, Stream, prelude::*};

struct Worker {
    child: Child,
    conn: Stream,
    connect_ms: f64,
    ready_ms: f64,
}

struct Env {
    worker_exe: PathBuf,
    shm: PathBuf,
    pdf: PathBuf,
    lib: PathBuf,
}

fn spawn(env: &Env, n: u32) -> Worker {
    let sock_id = format!("{}-{n}", std::process::id());
    let token: u64 = 0x5eed_0000 + n as u64;
    let listener = ListenerOptions::new()
        .name(socket_name(&sock_id).unwrap())
        .create_sync()
        .expect("listener");
    let start = Instant::now();
    let child = Command::new(&env.worker_exe)
        .args([
            sock_id.as_str(),
            &token.to_string(),
            env.shm.to_str().unwrap(),
            env.pdf.to_str().unwrap(),
            env.lib.to_str().unwrap(),
        ])
        .spawn()
        .expect("lanzar worker");
    // accept() en un hilo para poder aplicar un tiempo límite.
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(listener.accept());
    });
    let mut conn = rx
        .recv_timeout(Duration::from_secs(10))
        .expect("el worker no se conectó")
        .expect("accept");
    let connect_ms = ms(start.elapsed());
    match recv::<Response, _>(&mut conn).expect("hello") {
        Response::Hello { token: t, .. } if t == token => {}
        other => panic!("handshake inválido: {other:?}"),
    }
    let ready_ms = ms(start.elapsed());
    Worker {
        child,
        conn,
        connect_ms,
        ready_ms,
    }
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn pct(v: &mut [f64], p: f64) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[((v.len() - 1) as f64 * p).round() as usize]
}

fn main() {
    let dir = std::env::temp_dir().join(format!("demi-s2-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let pdf = dir.join("100-paginas.pdf");
    std::fs::write(&pdf, demi_spike_s2::sample_pdf(100)).unwrap();
    let shm = dir.join("tiles.shm");
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&shm)
        .unwrap();
    file.set_len((SLOTS * SLOT_BYTES) as u64).unwrap();
    // SAFETY: archivo propio de este proceso y de su worker.
    #[allow(unsafe_code)]
    let map = unsafe { memmap2::Mmap::map(&file).unwrap() };

    let lib = pdfium_lib::default_library_dir()
        .filter(|d| pdfium_lib::library_path(d).exists())
        .unwrap_or_else(vendor_pdfium);
    let worker_exe = std::env::current_exe()
        .unwrap()
        .with_file_name(format!("s2-worker{}", std::env::consts::EXE_SUFFIX));
    let env = Env {
        worker_exe,
        shm,
        pdf,
        lib,
    };
    let mut report = String::new();
    let os = format!("{} {}", std::env::consts::OS, std::env::consts::ARCH);
    report.push_str(&format!("## Spike S2 · worker + IPC ({os})\n\n"));

    // 1. Arranque.
    let (mut connect, mut ready) = (Vec::new(), Vec::new());
    for n in 0..10 {
        let mut w = spawn(&env, n);
        connect.push(w.connect_ms);
        ready.push(w.ready_ms);
        send(&mut w.conn, &Request::Shutdown).unwrap();
        w.child.wait().unwrap();
    }
    report.push_str("| Medida | p50 | p95 | máx | Objetivo |\n|---|---|---|---|---|\n");
    let row = |name: &str, v: &mut Vec<f64>, goal: &str| {
        let max = v.iter().cloned().fold(0.0, f64::max);
        format!(
            "| {name} | {:.2} ms | {:.2} ms | {:.2} ms | {goal} |\n",
            pct(v, 0.5),
            pct(v, 0.95),
            max
        )
    };
    report.push_str(&row("Arranque hasta conectar", &mut connect, ""));
    report.push_str(&row(
        "Arranque hasta listo (PDFium + PDF abierto)",
        &mut ready,
        "< 150 ms",
    ));

    // 2. Latencia por tesela (secuencial): 50 páginas × 6 teselas (página a 1024 px).
    let mut w = spawn(&env, 100);
    let (mut overhead, mut total, mut render, mut verify) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut id = 0u64;
    for page in 0..50u32 {
        for ty in 0..3 {
            for tx in 0..2 {
                id += 1;
                let slot = (id % SLOTS as u64) as u16;
                let t = Instant::now();
                send(
                    &mut w.conn,
                    &Request::Tile {
                        id,
                        page,
                        tx,
                        ty,
                        page_width_px: 1024,
                        slot,
                    },
                )
                .unwrap();
                let resp: Response = recv(&mut w.conn).unwrap();
                let Response::TileReady {
                    id: rid,
                    slot: s,
                    height,
                    render_us,
                    copy_us,
                    checksum: sum,
                    ..
                } = resp
                else {
                    panic!("respuesta inesperada")
                };
                assert_eq!(rid, id);
                let elapsed = ms(t.elapsed());
                // La app "recibe" la tesela: la lee de la memoria compartida y la verifica
                // (la verificación es solo del spike; se mide aparte).
                let tv = Instant::now();
                let base = s as usize * SLOT_BYTES;
                let tile = &map[base..base + (TILE * height * 4) as usize];
                assert_eq!(checksum(tile), sum, "la tesela no coincide entre procesos");
                verify.push(ms(tv.elapsed()));
                total.push(elapsed);
                if render_us > 0 {
                    render.push(render_us as f64 / 1000.0);
                }
                overhead.push(elapsed - (render_us + copy_us) as f64 / 1000.0);
            }
        }
    }
    report.push_str(&row(
        "Sobrecoste IPC por tesela (ida y vuelta − render − copia)",
        &mut overhead,
        "< 16 ms",
    ));
    report.push_str(&row(
        "Tesela completa (incl. render de página cuando toca)",
        &mut total,
        "",
    ));
    report.push_str(&row(
        "Render de página A4 a 1024 px (en el worker)",
        &mut render,
        "",
    ));
    report.push_str(&row(
        "Verificación de la tesela en la app (suma FNV, solo spike)",
        &mut verify,
        "",
    ));

    // 2b. En paralelo: 8 peticiones en vuelo.
    let t = Instant::now();
    let mut in_flight = 0;
    let mut done = 0;
    let total_tiles = 300;
    let mut next = 0u32;
    while done < total_tiles {
        while in_flight < 8 && next < total_tiles {
            let page = 50 + next / 6;
            let k = next % 6;
            id += 1;
            send(
                &mut w.conn,
                &Request::Tile {
                    id,
                    page,
                    tx: k % 2,
                    ty: k / 2,
                    page_width_px: 1024,
                    slot: (id % SLOTS as u64) as u16,
                },
            )
            .unwrap();
            in_flight += 1;
            next += 1;
        }
        let _: Response = recv(&mut w.conn).unwrap();
        in_flight -= 1;
        done += 1;
    }
    let secs = t.elapsed().as_secs_f64();
    report.push_str(&format!(
        "\nRendimiento con 8 peticiones en vuelo: **{:.0} teselas/s** ({total_tiles} teselas, 50 páginas, en {:.2} s).\n",
        total_tiles as f64 / secs,
        secs
    ));
    send(&mut w.conn, &Request::Shutdown).unwrap();
    w.child.wait().unwrap();

    // 3. Fallos: kill desde fuera y abort dentro del worker.
    report.push_str("\n| Fallo simulado | Detección | Reinicio hasta listo | La app sigue viva |\n|---|---|---|---|\n");
    for (name, crash) in [
        ("kill del proceso", false),
        ("abort() dentro del worker", true),
    ] {
        let mut w = spawn(&env, if crash { 201 } else { 200 });
        send(
            &mut w.conn,
            &Request::Tile {
                id: 1,
                page: 0,
                tx: 0,
                ty: 0,
                page_width_px: 1024,
                slot: 0,
            },
        )
        .unwrap();
        let _: Response = recv(&mut w.conn).unwrap();
        let t = Instant::now();
        if crash {
            send(&mut w.conn, &Request::Crash).unwrap();
        } else {
            w.child.kill().unwrap();
        }
        let detected = recv::<Response, _>(&mut w.conn).is_err();
        let detect_ms = ms(t.elapsed());
        let _ = w.child.wait();
        let t = Instant::now();
        let mut w2 = spawn(&env, if crash { 301 } else { 300 });
        send(
            &mut w2.conn,
            &Request::Tile {
                id: 2,
                page: 1,
                tx: 0,
                ty: 0,
                page_width_px: 1024,
                slot: 1,
            },
        )
        .unwrap();
        let ok = matches!(
            recv::<Response, _>(&mut w2.conn),
            Ok(Response::TileReady { .. })
        );
        let restart_ms = ms(t.elapsed());
        send(&mut w2.conn, &Request::Shutdown).unwrap();
        let _ = w2.child.wait();
        report.push_str(&format!(
            "| {name} | {} en {detect_ms:.1} ms | {restart_ms:.1} ms (incl. primera tesela) | {} |\n",
            if detected { "sí" } else { "NO" },
            if ok { "sí" } else { "NO" }
        ));
    }

    println!("{report}");
    if let Some(path) = std::env::var_os("GITHUB_STEP_SUMMARY") {
        let mut f = OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .unwrap();
        let _ = writeln!(f, "{report}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

fn vendor_pdfium() -> PathBuf {
    let platform = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "linux-x64",
        ("linux", "aarch64") => "linux-arm64",
        ("windows", "x86_64") => "win-x64",
        ("windows", "aarch64") => "win-arm64",
        ("macos", "aarch64") => "mac-arm64",
        _ => "mac-x64",
    };
    let sub = if platform.starts_with("win") {
        "bin"
    } else {
        "lib"
    };
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/pdfium")
        .join(platform)
        .join(sub)
}
