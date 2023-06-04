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

    for size in [1e+5, 1e+6, 1e+7, 1e+8] {
        let mut rng = rand::thread_rng();

        let v: Vec<_> = (0..size as usize)
            .map(|_| {
                Point10D {
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
                }
            })
            .collect();

        group.bench_with_input(BenchmarkId::new("vec", size), &v, |b, i| {
            b.iter(|| {
                let sum = i.iter().map(|x| x.a + x.b).sum::<f64>();
                let _sum2 = black_box(sum + 7.0);
            })
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

    for size in [1e+5, 1e+6, 1e+7, 1e+8] {
        let mut rng = rand::thread_rng();

        let v = (0..size as usize)
            .map(|_| {
                OPoint10D {
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
                }
            })
            .collect::<Vec<_>>()
            .into_ortho();

        group.bench_with_input(BenchmarkId::new("ortho vec", size), &v, |b, i| {
            b.iter(|| {
                let sum = i.iter().map(|x| x.a + x.b).sum::<f64>();
                let _sum2 = black_box(sum + 7.0);
            })
        });
    }
}

criterion_group!(benches, regular_vec_benchmark, ortho_vec_benchmark);
criterion_main!(benches);
