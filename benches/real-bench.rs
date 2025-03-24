use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use grav::{app::App, Orbital};

fn bench_real(c: &mut Criterion) {
    let mut app = Orbital::new();
    app.load_preset(1);
    app.start();
    println!(
        "{} * {} = {}",
        app.dt,
        app.num_ticks,
        app.dt * app.num_ticks as f32
    );

    c.bench_function("real_app_bench", |b| {
        b.iter(|| {
            app.run();
        })
    });

    println!("final: {}", app.t());
}

fn bench_batched(c: &mut Criterion) {
    c.bench_function("real_app_bench_batched", |b| {
        b.iter_batched_ref(
            || {
                let mut app = Orbital::new();
                app.load_preset(1);
                app.start();
                app
            },
            |app| app.run(),
            BatchSize::SmallInput,
        )
    });
}

// fn draw_bench(c: &mut Criterion) {
//     c.bench_function("draw_bench", |b| {
//         b.iter_batched_ref(
//             || {
//                 let mut app = Orbital::new();
//                 app.load_preset(1);
//                 app.start();
//                 app.run();
//                 app.run();
//                 app
//             },
//             |app| app.run(),
//             BatchSize::SmallInput,
//         )
//     });
// }

criterion_group!(benches, bench_real, bench_batched);
criterion_main!(benches);
