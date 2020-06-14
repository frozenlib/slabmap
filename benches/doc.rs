use criterion::{criterion_group, criterion_main, Bencher, BenchmarkId, Criterion, Throughput};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use std::collections::BTreeMap;
use std::collections::HashMap;

criterion_main!(benches);
criterion_group!(benches, criterion_benchmark);

fn inputs() -> Vec<usize> {
    //(1..100usize).map(|x| x * 100).collect()
    (1..20usize).map(|x| x * 500).collect()
}

pub fn criterion_benchmark(c: &mut Criterion) {
    {
        let mut g = c.benchmark_group("insert");
        for n in inputs() {
            g.throughput(Throughput::Elements(n as u64));
            g.bench_with_input(BenchmarkId::new("HashMap", n), &n, |b, n| {
                insert_hash_map(b, *n)
            });
            g.bench_with_input(BenchmarkId::new("BTreeMap", n), &n, |b, n| {
                insert_btree_map(b, *n)
            });
            g.bench_with_input(BenchmarkId::new("slab_iter::Slab", n), &n, |b, n| {
                insert_this(b, *n)
            });
            g.bench_with_input(BenchmarkId::new("Vec", n), &n, |b, n| insert_vec(b, *n));
        }
    }
    {
        let mut g = c.benchmark_group("remove(front)");
        for n in inputs() {
            g.throughput(Throughput::Elements(n as u64));
            g.bench_with_input(BenchmarkId::new("HashMap", n), &n, |b, n| {
                remove_front_hash_map(b, *n)
            });
            g.bench_with_input(BenchmarkId::new("BTreeMap", n), &n, |b, n| {
                remove_front_btree_map(b, *n)
            });
            g.bench_with_input(BenchmarkId::new("slab_iter::Slab", n), &n, |b, n| {
                remove_front_this(b, *n)
            });
            g.bench_with_input(BenchmarkId::new("Vec", n), &n, |b, n| {
                remove_front_vec(b, *n)
            });
        }
    }

    {
        let mut g = c.benchmark_group("remove(back)");
        for n in inputs() {
            g.throughput(Throughput::Elements(n as u64));
            g.bench_with_input(BenchmarkId::new("HashMap", n), &n, |b, n| {
                remove_back_hash_map(b, *n)
            });
            g.bench_with_input(BenchmarkId::new("BTreeMap", n), &n, |b, n| {
                remove_back_btree_map(b, *n)
            });
            g.bench_with_input(BenchmarkId::new("slab_iter::Slab", n), &n, |b, n| {
                remove_back_this(b, *n)
            });
            g.bench_with_input(BenchmarkId::new("Vec", n), &n, |b, n| {
                remove_back_vec(b, *n)
            });
        }
    }

    {
        let mut g = c.benchmark_group("remove(random)");
        for n in inputs() {
            g.throughput(Throughput::Elements(n as u64));
            g.bench_with_input(BenchmarkId::new("HashMap", n), &n, |b, n| {
                remove_random_hash_map(b, *n)
            });
            g.bench_with_input(BenchmarkId::new("BTreeMap", n), &n, |b, n| {
                remove_random_btree_map(b, *n)
            });
            g.bench_with_input(BenchmarkId::new("slab_iter::Slab", n), &n, |b, n| {
                remove_random_this(b, *n)
            });
        }
    }

    {
        let mut g = c.benchmark_group("get(sequential)");
        for n in inputs() {
            g.throughput(Throughput::Elements(n as u64));
            g.bench_with_input(BenchmarkId::new("HashMap", n), &n, |b, n| {
                get_seq_hash_map(b, *n)
            });
            g.bench_with_input(BenchmarkId::new("BTreeMap", n), &n, |b, n| {
                get_seq_btree_map(b, *n)
            });
            g.bench_with_input(BenchmarkId::new("slab_iter::Slab", n), &n, |b, n| {
                get_seq_this(b, *n)
            });
            g.bench_with_input(BenchmarkId::new("Vec", n), &n, |b, n| get_seq_vec(b, *n));
        }
    }

    {
        let mut g = c.benchmark_group("get(random)");
        for n in inputs() {
            g.throughput(Throughput::Elements(n as u64));
            g.bench_with_input(BenchmarkId::new("HashMap", n), &n, |b, n| {
                get_random_hash_map(b, *n)
            });
            g.bench_with_input(BenchmarkId::new("BTreeMap", n), &n, |b, n| {
                get_random_btree_map(b, *n)
            });
            g.bench_with_input(BenchmarkId::new("slab_iter::Slab", n), &n, |b, n| {
                get_random_this(b, *n)
            });
            g.bench_with_input(BenchmarkId::new("Vec", n), &n, |b, n| get_random_vec(b, *n));
        }
    }

    {
        let mut g = c.benchmark_group("iter");
        for n in inputs() {
            g.throughput(Throughput::Elements(n as u64));
            g.bench_with_input(BenchmarkId::new("HashMap", n), &n, |b, n| {
                iter_hash_map(b, *n)
            });
            g.bench_with_input(BenchmarkId::new("BTreeMap", n), &n, |b, n| {
                iter_btree_map(b, *n)
            });
            g.bench_with_input(BenchmarkId::new("slab_iter::Slab", n), &n, |b, n| {
                iter_this(b, *n)
            });
            g.bench_with_input(BenchmarkId::new("Vec", n), &n, |b, n| iter_vec(b, *n));
        }
    }

    {
        let mut g = c.benchmark_group("iter-insert-remove");
        for n in inputs() {
            g.throughput(Throughput::Elements(n as u64));
            g.bench_with_input(BenchmarkId::new("HashMap", n), &n, |b, n| {
                iter_insert_remove_hash_map(b, 10000, *n)
            });
            g.bench_with_input(BenchmarkId::new("BTreeMap", n), &n, |b, n| {
                iter_insert_remove_btree_map(b, 10000, *n)
            });
            g.bench_with_input(BenchmarkId::new("slab_iter::Slab", n), &n, |b, n| {
                iter_insert_remove_this(b, 10000, *n, false)
            });
            g.bench_with_input(
                BenchmarkId::new("slab_iter::Slab - optimized", n),
                &n,
                |b, n| iter_insert_remove_this(b, 10000, *n, true),
            );
            g.bench_with_input(BenchmarkId::new("slab::Slab", n), &n, |b, n| {
                iter_insert_remove_slab(b, 10000, *n)
            });
        }
    }
}

fn insert_hash_map(b: &mut Bencher, n: usize) {
    b.iter(|| {
        let mut c = HashMap::new();
        for i in 0..n {
            c.insert(i, i);
        }
        c
    })
}
fn insert_btree_map(b: &mut Bencher, n: usize) {
    b.iter(|| {
        let mut c = BTreeMap::new();
        for i in 0..n {
            c.insert(i, i);
        }
        c
    })
}
fn insert_this(b: &mut Bencher, n: usize) {
    b.iter(|| {
        let mut c = slab_iter::Slab::new();
        for i in 0..n {
            c.insert(i);
        }
        c
    })
}
fn insert_vec(b: &mut Bencher, n: usize) {
    b.iter(|| {
        let mut c = Vec::new();
        for i in 0..n {
            c.insert(i, i);
        }
        c
    })
}

fn remove_front_hash_map(b: &mut Bencher, n: usize) {
    let mut c = HashMap::new();
    for i in 0..n {
        c.insert(i, i);
    }
    b.iter(|| {
        let mut c = c.clone();
        for i in 0..n {
            c.remove(&i);
        }
        c
    })
}
fn remove_front_btree_map(b: &mut Bencher, n: usize) {
    let mut c = BTreeMap::new();
    for i in 0..n {
        c.insert(i, i);
    }
    b.iter(|| {
        let mut c = c.clone();
        for i in 0..n {
            c.remove(&i);
        }
        c
    })
}
fn remove_front_this(b: &mut Bencher, n: usize) {
    let mut c = slab_iter::Slab::new();
    for i in 0..n {
        c.insert(i);
    }
    b.iter(|| {
        let mut c = c.clone();
        for i in 0..n {
            c.remove(i);
        }
        c
    })
}
fn remove_front_vec(b: &mut Bencher, n: usize) {
    let mut c = Vec::new();
    for i in 0..n {
        c.insert(i, i);
    }
    b.iter(|| {
        let mut c = c.clone();
        for _ in 0..n {
            c.remove(0);
        }
        c
    })
}

fn remove_back_hash_map(b: &mut Bencher, n: usize) {
    let mut c = HashMap::new();
    for i in 0..n {
        c.insert(i, i);
    }
    b.iter(|| {
        let mut c = c.clone();
        for i in (0..n).rev() {
            c.remove(&i);
        }
        c
    })
}
fn remove_back_btree_map(b: &mut Bencher, n: usize) {
    let mut c = BTreeMap::new();
    for i in 0..n {
        c.insert(i, i);
    }
    b.iter(|| {
        let mut c = c.clone();
        for i in (0..n).rev() {
            c.remove(&i);
        }
        c
    })
}
fn remove_back_this(b: &mut Bencher, n: usize) {
    let mut c = slab_iter::Slab::new();
    for i in 0..n {
        c.insert(i);
    }
    b.iter(|| {
        let mut c = c.clone();
        for i in (0..n).rev() {
            c.remove(i);
        }
        c
    })
}
fn remove_back_vec(b: &mut Bencher, n: usize) {
    let mut c = Vec::new();
    for i in 0..n {
        c.insert(i, i);
    }
    b.iter(|| {
        let mut c = c.clone();
        for _ in 0..n {
            c.pop();
        }
        c
    })
}
fn remove_random_hash_map(b: &mut Bencher, n: usize) {
    let mut c = HashMap::new();
    for i in 0..n {
        c.insert(i, i);
    }
    let keys = make_random_keys(n);
    b.iter(|| {
        let mut c = c.clone();
        for i in &keys {
            c.remove(i);
        }
        c
    })
}
fn remove_random_btree_map(b: &mut Bencher, n: usize) {
    let mut c = BTreeMap::new();
    for i in 0..n {
        c.insert(i, i);
    }
    let keys = make_random_keys(n);
    b.iter(|| {
        let mut c = c.clone();
        for i in &keys {
            c.remove(i);
        }
        c
    })
}
fn remove_random_this(b: &mut Bencher, n: usize) {
    let mut c = slab_iter::Slab::new();
    for i in 0..n {
        c.insert(i);
    }
    let keys = make_random_keys(n);
    b.iter(|| {
        let mut c = c.clone();
        for i in &keys {
            c.remove(*i);
        }
        c
    })
}

fn get_seq_hash_map(b: &mut Bencher, n: usize) {
    let mut c = HashMap::new();
    for i in 0..n {
        c.insert(i, i);
    }
    b.iter(|| {
        let mut sum = 0;
        for i in 0..n {
            sum += *c.get(&i).unwrap()
        }
        sum
    })
}
fn get_seq_btree_map(b: &mut Bencher, n: usize) {
    let mut c = BTreeMap::new();
    for i in 0..n {
        c.insert(i, i);
    }
    b.iter(|| {
        let mut sum = 0;
        for i in 0..n {
            sum += *c.get(&i).unwrap()
        }
        sum
    })
}
fn get_seq_this(b: &mut Bencher, n: usize) {
    let mut c = slab_iter::Slab::new();
    for i in 0..n {
        c.insert(i);
    }
    b.iter(|| {
        let mut sum = 0;
        for i in 0..n {
            sum += *c.get(i).unwrap()
        }
        sum
    })
}
fn get_seq_vec(b: &mut Bencher, n: usize) {
    let mut c = Vec::new();
    for i in 0..n {
        c.insert(i, i);
    }
    b.iter(|| {
        let mut sum = 0;
        for i in 0..n {
            sum += *c.get(i).unwrap()
        }
        sum
    })
}

fn get_random_hash_map(b: &mut Bencher, n: usize) {
    let mut c = HashMap::new();
    for i in 0..n {
        c.insert(i, i);
    }
    let keys = make_random_keys(n);
    b.iter(|| {
        let mut sum = 0;
        for &i in &keys {
            sum += *c.get(&i).unwrap()
        }
        sum
    })
}
fn get_random_btree_map(b: &mut Bencher, n: usize) {
    let mut c = BTreeMap::new();
    for i in 0..n {
        c.insert(i, i);
    }
    let keys = make_random_keys(n);
    b.iter(|| {
        let mut sum = 0;
        for i in &keys {
            sum += *c.get(&i).unwrap()
        }
        sum
    })
}
fn get_random_this(b: &mut Bencher, n: usize) {
    let mut c = slab_iter::Slab::new();
    for i in 0..n {
        c.insert(i);
    }
    let keys = make_random_keys(n);
    b.iter(|| {
        let mut sum = 0;
        for i in &keys {
            sum += *c.get(*i).unwrap()
        }
        sum
    })
}
fn get_random_vec(b: &mut Bencher, n: usize) {
    let mut c = Vec::new();
    for i in 0..n {
        c.insert(i, i);
    }
    let keys = make_random_keys(n);
    b.iter(|| {
        let mut sum = 0;
        for i in &keys {
            sum += *c.get(*i).unwrap()
        }
        sum
    })
}

fn iter_hash_map(b: &mut Bencher, n: usize) {
    let mut c = HashMap::new();
    for i in 0..n {
        c.insert(i, i);
    }
    b.iter(|| {
        let sum: usize = c.values().sum();
        sum
    })
}
fn iter_btree_map(b: &mut Bencher, n: usize) {
    let mut c = BTreeMap::new();
    for i in 0..n {
        c.insert(i, i);
    }
    b.iter(|| {
        let sum: usize = c.values().sum();
        sum
    })
}
fn iter_this(b: &mut Bencher, n: usize) {
    let mut c = slab_iter::Slab::new();
    for i in 0..n {
        c.insert(i);
    }
    b.iter(|| {
        let sum: usize = c.values().sum();
        sum
    })
}
fn iter_vec(b: &mut Bencher, n: usize) {
    let mut c = Vec::new();
    for i in 0..n {
        c.insert(i, i);
    }
    b.iter(|| {
        let sum: usize = c.iter().sum();
        sum
    })
}

fn iter_insert_remove_hash_map(b: &mut Bencher, n: usize, m: usize) {
    let mut c = HashMap::new();
    for i in 0..n {
        c.insert(i, i);
    }
    let keys = make_random_keys(n);
    for i in m..n {
        c.remove(&keys[i]);
    }
    b.iter(|| {
        let sum: usize = c.values().sum();
        sum
    })
}
fn iter_insert_remove_btree_map(b: &mut Bencher, n: usize, m: usize) {
    let mut c = BTreeMap::new();
    for i in 0..n {
        c.insert(i, i);
    }
    let keys = make_random_keys(n);
    for i in m..n {
        c.remove(&keys[i]);
    }
    b.iter(|| {
        let sum: usize = c.values().sum();
        sum
    })
}
fn iter_insert_remove_this(b: &mut Bencher, n: usize, m: usize, optimized: bool) {
    let mut c = slab_iter::Slab::new();
    for i in 0..n {
        c.insert(i);
    }
    let keys = make_random_keys(n);
    for i in m..n {
        c.remove(keys[i]);
    }
    if optimized {
        c.optimize();
    }
    b.iter(|| {
        let sum: usize = c.values().sum();
        sum
    })
}
fn iter_insert_remove_slab(b: &mut Bencher, n: usize, m: usize) {
    let mut c = slab::Slab::new();
    for i in 0..n {
        c.insert(i);
    }
    let keys = make_random_keys(n);
    for i in m..n {
        c.remove(keys[i]);
    }
    b.iter(|| {
        let sum: usize = c.iter().map(|(_, v)| v).sum();
        sum
    })
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
