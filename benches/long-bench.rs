mod common;

use common::load_preset;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use femtovg::Color;
use grav::{
    helpers::{init_canvas, wgpu::create_canvas},
    App,
};

// bench a call of the "run" function with many ticks already calculated
fn long_run_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("long-bench");
    group.sample_size(20);

    group.bench_function("long_run_bench", |b| {
        b.iter_batched_ref(
            || {
                let mut app = load_preset(1);
                app.num_ticks = 1e6 as i32;
                app.start();
                app.run();
                app.run();
                app.run();
                app
            },
            |app| app.run(),
            BatchSize::SmallInput,
        )
    });
}

fn long_draw_bench(c: &mut Criterion) {
    // Skip this benchmark in headless environments like CI
    if std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err() {
        println!("Skipping long_draw_bench: no display available");
        return;
    }

    let mut group = c.benchmark_group("long-bench");
    group.sample_size(60);

    let (mut canvas, _, _, _) = spin_on::spin_on(create_canvas(1600, 1000, "benching"));

    init_canvas(&mut canvas);

    group.bench_function("long_draw_bench", |b| {
        b.iter_batched_ref(
            || {
                let mut app = load_preset(1);
                app.start();
                for _ in 0..10 {
                    app.run();
                }
                app
            },
            |app| {
                canvas.clear_rect(0, 0, 1600, 1000, Color::black());
                app.draw(&mut canvas);
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, long_run_bench, long_draw_bench);
criterion_main!(benches);
