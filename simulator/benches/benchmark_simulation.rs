use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use rand::{SeedableRng, rngs::StdRng};
use simulator::{Simulation, SimulationResult, SimulationTemplate};
use std::hint::black_box;
use toml;

const TEMPLATE_STR: &str = include_str!("simulations/complex.toml");
const SEED: u64 = 0x3a3f7af8085da7a2;

fn load_simulation() -> (Simulation, u64) {
    let template = toml::from_str::<SimulationTemplate>(&TEMPLATE_STR).expect("bad TOML");
    let seed = template.seed.unwrap_or(SEED);
    let sim: Simulation = template.try_into().expect("invalid template");
    (sim, seed)
}

fn bench_run_once(c: &mut Criterion) {
    let (sim, seed) = load_simulation();
    let mut group = c.benchmark_group("simulation");
    group.throughput(Throughput::Elements(1));

    group.bench_function("run_once_with_rng", |b| {
        b.iter(|| {
            let rng = StdRng::seed_from_u64(black_box(seed));
            let out: SimulationResult = black_box(sim.clone().run_once_with_rng(rng));
            black_box(out);
        })
    });

    group.finish();
}

criterion_group!(benches, bench_run_once);
criterion_main!(benches);
