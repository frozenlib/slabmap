use criterion::{criterion_group, criterion_main, Bencher, Criterion};
use rand::prelude::*;
use slab_iter::Slab;

criterion_main!(benches);
criterion_group!(benches, criterion_benchmark);

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut g = c.benchmark_group("regression");
    g.bench_function("insert", insert);
    g.bench_function("remove", remove);
    g.bench_function("values", values);
    g.bench_function("optimize", optimize);
}

const COUNT: usize = 10000;

fn insert(b: &mut Bencher) {
    b.iter(|| {
        let mut s = Slab::new();
        for i in 0..COUNT {
            s.insert(i);
        }
        s
    });
}
fn remove(b: &mut Bencher) {
    let mut s = Slab::new();
    for i in 0..COUNT {
        s.insert(i);
    }
    let keys = make_random_keys(COUNT);
    b.iter(|| {
        let mut s = s.clone();
        for key in &keys {
            s.remove(*key);
        }
        s
    });
}
fn values(b: &mut Bencher) {
    let mut s = Slab::new();
    for i in 0..COUNT {
        s.insert(i);
    }
    b.iter(|| {
        let sum: usize = s.values().sum();
        sum
    });
}
fn optimize(b: &mut Bencher) {
    let mut s = Slab::new();
    for i in 0..COUNT {
        s.insert(i);
    }
    for i in 0..COUNT - 1 {
        s.remove(i);
    }
    b.iter(|| {
        s.optimize();
        s.len()
    });
}

fn make_rng() -> StdRng {
    let seed: [u8; 32] = [17; 32];
    rand::SeedableRng::from_seed(seed)
}
fn make_random_keys(n: usize) -> Vec<usize> {
    let mut rng = make_rng();
    let mut keys: Vec<usize> = (0..n).collect();
    keys.shuffle(&mut rng);
    keys
}
