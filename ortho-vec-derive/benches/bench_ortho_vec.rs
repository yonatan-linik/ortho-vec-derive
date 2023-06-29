#![allow(dead_code)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use ortho_vec_derive::prelude::*;
use rand::Rng;

struct Point10D {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    e: f64,
    f: f64,
    g: f64,
    h: f64,
    i: f64,
    j: f64,
}

pub fn regular_vec_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("summation");

    for size in [1e+2, 1e+3, 1e+4, 1e+5, 1e+6, 1e+7] {
        let mut rng = rand::thread_rng();

        group.bench_function(BenchmarkId::new("vec", size), |b| {
            b.iter_batched_ref(
                || {
                    (0..size as usize)
                        .map(|_| Point10D {
                            a: rng.gen(),
                            b: rng.gen(),
                            c: rng.gen(),
                            d: rng.gen(),
                            e: rng.gen(),
                            f: rng.gen(),
                            g: rng.gen(),
                            h: rng.gen(),
                            i: rng.gen(),
                            j: rng.gen(),
                        })
                        .collect::<Vec<_>>()
                },
                |v| {
                    v.iter_mut().for_each(|x| x.a += 3.0 * x.b + x.a);
                    let sum = v.iter().map(|x| x.a + x.b).sum::<f64>();
                    let _sum2 = black_box(sum + 7.0);
                },
                criterion::BatchSize::LargeInput,
            )
        });
    }
}

#[derive(OrthoVec)]
struct OPoint10D {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    e: f64,
    f: f64,
    g: f64,
    h: f64,
    i: f64,
    j: f64,
}

pub fn ortho_vec_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("summation");

    for size in [1e+2, 1e+3, 1e+4, 1e+5, 1e+6, 1e+7] {
        let mut rng = rand::thread_rng();

        group.bench_function(BenchmarkId::new("ortho_vec", size), |b| {
            b.iter_batched_ref(
                || {
                    (0..size as usize)
                        .map(|_| OPoint10D {
                            a: rng.gen(),
                            b: rng.gen(),
                            c: rng.gen(),
                            d: rng.gen(),
                            e: rng.gen(),
                            f: rng.gen(),
                            g: rng.gen(),
                            h: rng.gen(),
                            i: rng.gen(),
                            j: rng.gen(),
                        })
                        .collect::<Vec<_>>()
                        .into_ortho()
                },
                |v| {
                    v.iter_mut().for_each(|x| *x.a += 3.0 * *x.b + *x.a);
                    let sum = v.iter().map(|x| x.a + x.b).sum::<f64>();
                    let _sum2 = black_box(sum + 7.0);
                },
                criterion::BatchSize::LargeInput,
            )
        });
    }
}

criterion_group!(benches, regular_vec_benchmark, ortho_vec_benchmark);
criterion_main!(benches);
