use criterion::{criterion_group, criterion_main, Criterion};

criterion_main!(benches);
criterion_group!(benches, criterion_benchmark);

pub fn criterion_benchmark(c: &mut Criterion) {
    {
        let mut g = c.benchmark_group("insert 10000");
        g.bench_function("this", |b| b.iter(|| insert_this(10000)));
        g.bench_function("slab", |b| b.iter(|| insert_slab(10000)));
    }
    {
        let mut g = c.benchmark_group("insert remove 10000");
        g.bench_function("this", |b| b.iter(|| insert_remove_this(10000)));
        g.bench_function("slab", |b| b.iter(|| insert_remove_slab(10000)));
    }
    {
        let mut g = c.benchmark_group("iter 1000 1000");
        g.bench_function("this", |b| b.iter(|| iter_this(1000, 1000)));
        g.bench_function("slab", |b| b.iter(|| iter_slab(1000, 1000)));
    }
    {
        let mut g = c.benchmark_group("iter_head 1000 1000");
        g.bench_function("this_optimize", |b| {
            b.iter(|| iter_head_this(1000, 1000, true))
        });
        g.bench_function("this", |b| b.iter(|| iter_head_this(1000, 1000, false)));
        g.bench_function("slab", |b| b.iter(|| iter_head_slab(1000, 1000)));
    }
    {
        let mut g = c.benchmark_group("iter_tail 1000 1000");
        g.bench_function("this_optimize", |b| {
            b.iter(|| iter_tail_this(1000, 1000, true))
        });
        g.bench_function("this", |b| b.iter(|| iter_tail_this(1000, 1000, false)));
        g.bench_function("slab", |b| b.iter(|| iter_tail_slab(1000, 1000)));
    }
    {
        let mut g = c.benchmark_group("iter_sparse 1000 1000");
        g.bench_function("this_optimize", |b| {
            b.iter(|| iter_sparse_this(1000, 1000, true))
        });
        g.bench_function("this", |b| b.iter(|| iter_sparse_this(1000, 1000, false)));
        g.bench_function("slab", |b| b.iter(|| iter_sparse_slab(1000, 1000)));
    }
}

fn insert_this(n: usize) -> slab_iter::Slab<usize> {
    let mut s = slab_iter::Slab::new();
    for i in 0..n {
        s.insert(i);
    }
    s
}
fn insert_slab(n: usize) -> slab::Slab<usize> {
    let mut s = slab::Slab::new();
    for i in 0..n {
        s.insert(i);
    }
    s
}

fn insert_remove_this(n: usize) -> slab_iter::Slab<usize> {
    let mut s = slab_iter::Slab::new();
    for i in 0..n {
        s.insert(i);
    }
    for i in 0..n {
        s.remove(i);
    }
    s
}
fn insert_remove_slab(n: usize) -> slab::Slab<usize> {
    let mut s = slab::Slab::new();
    for i in 0..n {
        s.insert(i);
    }
    for i in 0..n {
        s.remove(i);
    }
    s
}

fn iter_this(len: usize, n: usize) -> usize {
    let mut s = slab_iter::Slab::new();
    for i in 0..len {
        s.insert(i);
    }
    let mut sum = 0;
    for _ in 0..n {
        for x in s.iter() {
            sum += x.1;
        }
    }
    sum
}
fn iter_slab(len: usize, n: usize) -> usize {
    let mut s = slab::Slab::new();
    for i in 0..len {
        s.insert(i);
    }
    let mut sum = 0;
    for _ in 0..n {
        for x in s.iter() {
            sum += x.1;
        }
    }
    sum
}

fn iter_head_this(len: usize, n: usize, optimize: bool) -> usize {
    let mut s = slab_iter::Slab::new();
    for i in 0..len {
        s.insert(i);
    }
    for i in (0..len).take(n * 3 / 4) {
        s.remove(i);
    }
    if optimize {
        s.optimize();
    }

    let mut sum = 0;
    for _ in 0..n {
        for x in s.iter() {
            sum += x.1;
        }
    }
    sum
}

fn iter_head_slab(len: usize, n: usize) -> usize {
    let mut s = slab::Slab::new();
    for i in 0..len {
        s.insert(i);
    }
    for i in (0..len).take(n * 3 / 4) {
        s.remove(i);
    }
    let mut sum = 0;
    for _ in 0..n {
        for x in s.iter() {
            sum += x.1;
        }
    }
    sum
}

fn iter_tail_this(len: usize, n: usize, optimize: bool) -> usize {
    let mut s = slab_iter::Slab::new();
    for i in 0..len {
        s.insert(i);
    }
    for i in (0..len).rev().take(n * 3 / 4) {
        s.remove(i);
    }
    if optimize {
        s.optimize();
    }

    let mut sum = 0;
    for _ in 0..n {
        for x in s.iter() {
            sum += x.1;
        }
    }
    sum
}

fn iter_tail_slab(len: usize, n: usize) -> usize {
    let mut s = slab::Slab::new();
    for i in 0..len {
        s.insert(i);
    }
    for i in (0..len).rev().take(n * 3 / 4) {
        s.remove(i);
    }
    let mut sum = 0;
    for _ in 0..n {
        for x in s.iter() {
            sum += x.1;
        }
    }
    sum
}

fn iter_sparse_this(len: usize, n: usize, optimize: bool) -> usize {
    let mut s = slab_iter::Slab::new();
    for i in 0..len {
        s.insert(i);
    }
    for i in 0..len {
        if i % (len / 10) != 0 {
            s.remove(i);
        }
    }
    if optimize {
        s.optimize();
    }

    let mut sum = 0;
    for _ in 0..n {
        for x in s.iter() {
            sum += x.1;
        }
    }
    sum
}

fn iter_sparse_slab(len: usize, n: usize) -> usize {
    let mut s = slab::Slab::new();
    for i in 0..len {
        s.insert(i);
    }
    for i in 0..len {
        if i % (len / 10) != 0 {
            s.remove(i);
        }
    }
    let mut sum = 0;
    for _ in 0..n {
        for x in s.iter() {
            sum += x.1;
        }
    }
    sum
}
