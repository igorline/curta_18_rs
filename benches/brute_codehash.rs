use criterion::{black_box, criterion_group, criterion_main, Criterion};
use curta_18_solver::brute_codehash::bruteforce_code_hash;

fn bench_brute(c: &mut Criterion) {
    let initial_bytecode = black_box("636cbf043d60e01b5f5260205ff3");
    let start_code_hash: [u8; 3] = [0, 0x90, 0x1d];
    c.bench_function("brute_codehash", |b| {
        b.iter(|| bruteforce_code_hash(black_box(&start_code_hash), initial_bytecode))
    });
}

criterion_group!(benches, bench_brute);
criterion_main!(benches);
