use criterion::{
    black_box, criterion_group, criterion_main, measurement::WallTime, Bencher, BenchmarkGroup,
    BenchmarkId, Criterion,
};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use slab::Slab;
use slabmap::SlabMap;
use std::collections::{BTreeMap, HashMap};

criterion_main!(benches);
criterion_group!(benches, criterion_benchmark);

fn inputs_small() -> Vec<usize> {
    vec![1, 2, 3, 5, 10, 20, 30]
}

fn inputs_large() -> Vec<usize> {
    let mut inputs = vec![1];
    inputs.extend((1..=20usize).map(|x| x * 500));
    inputs
}

pub fn criterion_benchmark(c: &mut Criterion) {
    struct Insert;
    impl BenchFunc for Insert {
        fn bench<T: BenchTarget>(b: &mut Bencher, n: usize) {
            b.iter(|| T::new_n(n))
        }
    }
    Insert::register(c, "insert_small", &inputs_small());
    Insert::register(c, "insert_large", &inputs_large());
    Insert::register_with(
        c,
        "insert_large_fast_only",
        &inputs_large(),
        BenchTargets::DEFAULT.no_map(),
    );

    struct InsertNoAlloc;
    impl BenchFunc for InsertNoAlloc {
        fn bench<T: BenchTarget>(b: &mut Bencher, n: usize) {
            let mut c = T::new_n(n);
            b.iter(|| {
                c.clear();
                for i in 0..n {
                    c.insert(i);
                }
                black_box(&c);
            })
        }
    }
    InsertNoAlloc::register(c, "insert_small_with_capacity", &inputs_small());
    InsertNoAlloc::register_with(
        c,
        "insert_small_with_capacity_fast_only",
        &inputs_small(),
        BenchTargets::DEFAULT.no_map(),
    );

    struct RemoveFront;
    impl BenchFunc for RemoveFront {
        fn bench<T: BenchTarget>(b: &mut Bencher, n: usize) {
            let c = T::new_n(n);
            b.iter(|| {
                let mut c = c.clone();
                for i in 0..n {
                    c.remove_front(i);
                }
                c
            })
        }
    }
    RemoveFront::register(c, "remove_front", &inputs_large());
    RemoveFront::register_with(
        c,
        "remove_front_fast_only",
        &inputs_large(),
        BenchTargets::DEFAULT.no_vec(),
    );

    struct RemoveBack;
    impl BenchFunc for RemoveBack {
        fn bench<T: BenchTarget>(b: &mut Bencher, n: usize) {
            let c = T::new_n(n);
            b.iter(|| {
                let mut c = c.clone();
                for i in (0..n).rev() {
                    c.remove_back(i);
                }
                c
            })
        }
    }
    RemoveBack::register(c, "remove_back", &inputs_large());
    RemoveBack::register_with(
        c,
        "remove_back",
        &inputs_large(),
        BenchTargets::DEFAULT.no_map(),
    );

    struct RemoveRandom;
    impl BenchFunc for RemoveRandom {
        fn bench<T: BenchTarget>(b: &mut Bencher, n: usize) {
            let c = T::new_n(n);
            let keys = make_random_keys(n);
            b.iter(|| {
                let mut c = c.clone();
                for &i in &keys {
                    c.remove(i);
                }
                c
            })
        }
    }
    RemoveRandom::register(c, "remove_random_small", &inputs_small());
    RemoveRandom::register(c, "remove_random_large", &inputs_large());
    RemoveRandom::register_with(
        c,
        "remove_random_large_fast_only",
        &inputs_large(),
        BenchTargets::DEFAULT.no_vec(),
    );

    struct GetSeq;
    impl BenchFunc for GetSeq {
        fn bench<T: BenchTarget>(b: &mut Bencher, n: usize) {
            let c = T::new_n(n);
            b.iter(|| {
                let mut sum = 0;
                for i in 0..n {
                    sum += c.get(i);
                }
                sum
            })
        }
    }
    GetSeq::register(c, "get_sequential", &inputs_large());
    GetSeq::register_with(
        c,
        "get_sequential_fast_only",
        &inputs_large(),
        BenchTargets::DEFAULT.no_map(),
    );

    struct GetRandom;
    impl BenchFunc for GetRandom {
        fn bench<T: BenchTarget>(b: &mut Bencher, n: usize) {
            let c = T::new_n(n);
            let keys = make_random_keys(n);
            b.iter(|| {
                let mut sum = 0;
                for &i in &keys {
                    sum += c.get(i);
                }
                sum
            })
        }
    }
    GetRandom::register(c, "get_random", &inputs_large());
    GetRandom::register_with(
        c,
        "get_random_fast_only",
        &inputs_large(),
        BenchTargets::DEFAULT.no_map(),
    );

    struct IterValues;
    impl BenchFunc for IterValues {
        fn bench<T: BenchTarget>(b: &mut Bencher, n: usize) {
            let c = T::new_n(n);
            b.iter(|| c.values())
        }
    }
    IterValues::register(c, "iter_values", &inputs_large());

    struct IterValuesSparse;
    impl BenchFunc for IterValuesSparse {
        fn bench<T: BenchTarget>(b: &mut Bencher, n: usize) {
            let mut c = T::new_n_retain(10000, n);
            c.optimize();
            b.iter(|| c.values())
        }
    }
    IterValuesSparse::register_with(c, "iter_values_removed", &inputs_large(), BenchTargets::ALL);
    IterValuesSparse::register_with(c, "iter_values_sparse", &inputs_small(), BenchTargets::ALL);
    IterValuesSparse::register_with(
        c,
        "iter_values_sparse_fast_only",
        &inputs_small(),
        BenchTargets::ALL.no_slab(),
    );

    struct IterKeyValues;
    impl BenchFunc for IterKeyValues {
        fn bench<T: BenchTarget>(b: &mut Bencher, n: usize) {
            let c = T::new_n(n);
            b.iter(|| c.key_values())
        }
    }
    IterKeyValues::register(c, "iter_key_values", &inputs_large());

    struct IterKeyValuesSparse;
    impl BenchFunc for IterKeyValuesSparse {
        fn bench<T: BenchTarget>(b: &mut Bencher, n: usize) {
            let mut c = T::new_n_retain(10000, n);
            c.optimize();
            b.iter(|| c.key_values())
        }
    }
    IterKeyValuesSparse::register_with(
        c,
        "iter_key_values_removed",
        &inputs_large(),
        BenchTargets::ALL,
    );
    IterKeyValuesSparse::register_with(
        c,
        "iter_key_values_sparse",
        &inputs_small(),
        BenchTargets::ALL,
    );
    IterKeyValuesSparse::register_with(
        c,
        "iter_key_values_sparse_fast_only",
        &inputs_small(),
        BenchTargets::ALL.no_slab(),
    );
}

trait BenchFunc {
    fn bench<T: BenchTarget>(b: &mut Bencher, n: usize);

    fn bench_as<T: BenchTarget>(g: &mut BenchmarkGroup<WallTime>, input: usize) {
        g.bench_with_input(T::id(input), &input, |b, n| Self::bench::<T>(b, *n));
    }
    fn not_available<T: BenchTarget>(g: &mut BenchmarkGroup<WallTime>) {
        let input = 0usize;
        g.bench_with_input(
            BenchmarkId::new(&format!("{} - n/a", T::NAME), input),
            &input,
            |b, i| b.iter(|| i + 10),
        );
    }

    fn register(c: &mut Criterion, name: &str, inputs: &[usize]) {
        Self::register_with(c, name, inputs, BenchTargets::DEFAULT)
    }

    fn register_with(c: &mut Criterion, name: &str, inputs: &[usize], targets: BenchTargets) {
        let mut g = c.benchmark_group(name);
        if !targets.vec {
            Self::not_available::<Vec<usize>>(&mut g);
        }
        if !targets.hash_map {
            Self::not_available::<HashMap<usize, usize>>(&mut g);
        }
        if !targets.btree_map {
            Self::not_available::<BTreeMap<usize, usize>>(&mut g);
        }
        if !targets.slab {
            Self::not_available::<Slab<usize>>(&mut g);
        }
        if !targets.slabmap {
            Self::not_available::<SlabMap<usize>>(&mut g);
        }
        if !targets.slabmap_optimized {
            Self::not_available::<SlabMapOptimized>(&mut g);
        }
        for &input in inputs {
            if targets.vec {
                Self::bench_as::<Vec<usize>>(&mut g, input);
            }
            if targets.hash_map {
                Self::bench_as::<HashMap<usize, usize>>(&mut g, input);
            }
            if targets.btree_map {
                Self::bench_as::<BTreeMap<usize, usize>>(&mut g, input);
            }
            if targets.slab {
                Self::bench_as::<Slab<usize>>(&mut g, input);
            }
            if targets.slabmap {
                Self::bench_as::<SlabMap<usize>>(&mut g, input);
            }
            if targets.slabmap_optimized {
                Self::bench_as::<SlabMapOptimized>(&mut g, input);
            }
        }
    }
}

#[derive(Copy, Clone)]
struct BenchTargets {
    vec: bool,
    hash_map: bool,
    btree_map: bool,
    slab: bool,
    slabmap: bool,
    slabmap_optimized: bool,
}
impl BenchTargets {
    const DEFAULT: Self = Self {
        slabmap_optimized: false,
        ..Self::ALL
    };
    const ALL: Self = Self {
        vec: true,
        hash_map: true,
        btree_map: true,
        slab: true,
        slabmap: true,
        slabmap_optimized: true,
    };
    fn no_vec(self) -> Self {
        Self { vec: false, ..self }
    }
    fn no_map(self) -> Self {
        Self {
            hash_map: false,
            btree_map: false,
            ..self
        }
    }
    fn no_slab(self) -> Self {
        Self {
            slab: false,
            slabmap: false,
            ..self
        }
    }
}

trait BenchTarget: Clone {
    const NAME: &'static str;
    fn id(p: usize) -> BenchmarkId {
        BenchmarkId::new(Self::NAME, p)
    }

    fn new() -> Self;
    fn insert(&mut self, n: usize);
    fn remove(&mut self, n: usize);
    fn clear(&mut self);
    fn values(&self) -> usize;
    fn key_values(&self) -> usize;
    fn get(&self, i: usize) -> usize;

    #[inline]
    fn new_n(n: usize) -> Self {
        let mut s = Self::new();
        for i in 0..n {
            s.insert(i);
        }
        s
    }
    fn new_n_retain(n: usize, m: usize) -> Self {
        let mut s = Self::new_n(n);
        let keys = make_random_keys(n);
        for i in m..n {
            s.remove(keys[i]);
        }
        s
    }

    #[inline]
    fn remove_front(&mut self, n: usize) {
        self.remove(n);
    }
    #[inline]
    fn remove_back(&mut self, n: usize) {
        self.remove(n);
    }

    #[inline]
    fn optimize(&mut self) {}
}

impl BenchTarget for Vec<usize> {
    const NAME: &'static str = "Vec";
    #[inline]
    fn new() -> Self {
        Vec::new()
    }
    #[inline]
    fn insert(&mut self, n: usize) {
        self.push(n)
    }
    #[inline]
    fn remove(&mut self, n: usize) {
        let idx = self.iter().position(|&x| x == n).unwrap();
        self.remove(idx);
    }
    #[inline]
    fn clear(&mut self) {
        self.clear();
    }
    #[inline]
    fn values(&self) -> usize {
        self.iter().sum()
    }
    #[inline]
    fn key_values(&self) -> usize {
        self.iter().enumerate().fold(0, |s, (k, v)| s + k + v)
    }
    #[inline]
    fn get(&self, i: usize) -> usize {
        self[i]
    }

    fn new_n_retain(n: usize, m: usize) -> Self {
        let mut s = Self::new_n(n);
        s.truncate(m);
        s
    }

    #[inline]
    fn remove_front(&mut self, _n: usize) {
        self.remove(0);
    }
    #[inline]
    fn remove_back(&mut self, _n: usize) {
        self.pop();
    }
}
impl BenchTarget for HashMap<usize, usize> {
    const NAME: &'static str = "HashMap";

    #[inline]
    fn new() -> Self {
        HashMap::new()
    }
    #[inline]
    fn insert(&mut self, n: usize) {
        self.insert(n, n);
    }
    #[inline]
    fn remove(&mut self, n: usize) {
        self.remove(&n);
    }
    #[inline]
    fn clear(&mut self) {
        self.clear();
    }
    #[inline]
    fn values(&self) -> usize {
        self.values().sum()
    }
    #[inline]
    fn key_values(&self) -> usize {
        self.iter().fold(0, |s, (k, v)| s + k + v)
    }
    #[inline]
    fn get(&self, i: usize) -> usize {
        self[&i]
    }
}
impl BenchTarget for BTreeMap<usize, usize> {
    const NAME: &'static str = "BTreeMap";

    #[inline]
    fn new() -> Self {
        BTreeMap::new()
    }
    #[inline]
    fn insert(&mut self, n: usize) {
        self.insert(n, n);
    }
    #[inline]
    fn remove(&mut self, n: usize) {
        self.remove(&n);
    }
    #[inline]
    fn clear(&mut self) {
        self.clear();
    }
    #[inline]
    fn values(&self) -> usize {
        self.values().sum()
    }
    #[inline]
    fn key_values(&self) -> usize {
        self.iter().fold(0, |s, (k, v)| s + k + v)
    }
    #[inline]
    fn get(&self, i: usize) -> usize {
        self[&i]
    }
}
impl BenchTarget for SlabMap<usize> {
    const NAME: &'static str = "SlabMap";

    #[inline]
    fn new() -> Self {
        SlabMap::new()
    }
    #[inline]
    fn insert(&mut self, n: usize) {
        self.insert(n);
    }
    #[inline]
    fn remove(&mut self, n: usize) {
        self.remove(n);
    }
    #[inline]
    fn clear(&mut self) {
        self.clear();
    }
    #[inline]
    fn values(&self) -> usize {
        self.values().sum()
    }
    #[inline]
    fn key_values(&self) -> usize {
        self.iter().fold(0, |s, (k, v)| s + k + v)
    }
    #[inline]
    fn get(&self, i: usize) -> usize {
        self[i]
    }
}

#[derive(Clone)]
struct SlabMapOptimized(SlabMap<usize>);

impl BenchTarget for SlabMapOptimized {
    const NAME: &'static str = "SlabMap(optimized)";

    #[inline]
    fn new() -> Self {
        SlabMapOptimized(SlabMap::new())
    }
    #[inline]
    fn insert(&mut self, n: usize) {
        self.0.insert(n);
    }
    #[inline]
    fn remove(&mut self, n: usize) {
        self.0.remove(n);
    }
    #[inline]
    fn clear(&mut self) {
        self.0.clear();
    }
    #[inline]
    fn values(&self) -> usize {
        self.0.values().sum()
    }
    #[inline]
    fn key_values(&self) -> usize {
        self.0.iter().fold(0, |s, (k, v)| s + k + v)
    }
    #[inline]
    fn get(&self, i: usize) -> usize {
        self.0[i]
    }
    #[inline]
    fn optimize(&mut self) {
        self.0.optimize()
    }
}

impl BenchTarget for Slab<usize> {
    const NAME: &'static str = "Slab";

    #[inline]
    fn new() -> Self {
        Slab::new()
    }
    #[inline]
    fn insert(&mut self, n: usize) {
        self.insert(n);
    }
    #[inline]
    fn remove(&mut self, n: usize) {
        self.remove(n);
    }
    #[inline]
    fn clear(&mut self) {
        self.clear();
    }
    #[inline]
    fn values(&self) -> usize {
        self.iter().map(|x| x.1).sum()
    }
    #[inline]
    fn key_values(&self) -> usize {
        self.iter().fold(0, |s, (k, v)| s + k + v)
    }
    #[inline]
    fn get(&self, i: usize) -> usize {
        self[i]
    }
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
