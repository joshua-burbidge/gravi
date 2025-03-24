use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use femtovg::Color;
use grav::{
    helpers::{init_canvas, wgpu::create_canvas},
    App, Orbital,
};

// bench a call of the "run" function with many ticks already calculated
fn long_run_bench(c: &mut Criterion) {
    c.bench_function("long_run_bench", |b| {
        b.iter_batched_ref(
            || {
                let mut app = Orbital::new();
                app.load_preset(1);
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
    let (mut canvas, _, _, _) = spin_on::spin_on(create_canvas(1600, 1000, "benching"));

    init_canvas(&mut canvas);

    c.bench_function("long_draw_bench", |b| {
        b.iter_batched_ref(
            || {
                let mut app = Orbital::new();
                app.load_preset(1);
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
