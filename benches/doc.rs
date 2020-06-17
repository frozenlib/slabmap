use criterion::{
    black_box, criterion_group, criterion_main, measurement::WallTime, Bencher, BenchmarkGroup,
    BenchmarkId, Criterion,
};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use std::collections::{BTreeMap, HashMap};

criterion_main!(benches);
criterion_group!(benches, criterion_benchmark);

fn inputs_small() -> Vec<usize> {
    vec![1, 2, 3, 5, 10, 20, 30]
}

fn inputs_large() -> Vec<usize> {
    (1..20usize).map(|x| x * 500).collect()
}

pub fn criterion_benchmark(c: &mut Criterion) {
    struct Insert;
    impl BenchFunc for Insert {
        fn bench<T: BenchTarget>(b: &mut Bencher, n: usize) {
            b.iter(|| T::new_n(n))
        }
    }
    Insert::register(c, "insert_large", &inputs_large());
    Insert::register(c, "insert_small", &inputs_small());

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
    InsertNoAlloc::register(c, "insert_small_no_alloc", &inputs_small());

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
    RemoveRandom::register_with(c, "remove_random", &inputs_large(), false, false);

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

    struct Iter;
    impl BenchFunc for Iter {
        fn bench<T: BenchTarget>(b: &mut Bencher, n: usize) {
            let c = T::new_n(n);
            b.iter(|| c.values())
        }
    }
    Iter::register(c, "iter", &inputs_large());

    struct IterSparse;
    impl BenchFunc for IterSparse {
        fn bench<T: BenchTarget>(b: &mut Bencher, n: usize) {
            let m = 10000;
            let mut c = T::new_n(m);
            let keys = make_random_keys(m);
            for i in n..m {
                c.remove(keys[i]);
            }
            c.optimize();
            b.iter(|| c.values())
        }
    }
    IterSparse::register_with(c, "iter-sparse", &inputs_large(), false, true);
}

trait BenchFunc {
    fn bench<T: BenchTarget>(b: &mut Bencher, n: usize);

    fn bench_as<T: BenchTarget>(g: &mut BenchmarkGroup<WallTime>, input: usize) {
        g.bench_with_input(T::id(input), &input, |b, n| Self::bench::<T>(b, *n));
    }

    fn register(c: &mut Criterion, name: &str, inputs: &[usize]) {
        Self::register_with(c, name, inputs, true, false)
    }

    fn register_with(
        c: &mut Criterion,
        name: &str,
        inputs: &[usize],
        include_vec: bool,
        include_optimized: bool,
    ) {
        let mut g = c.benchmark_group(name);
        for &input in inputs {
            if include_vec {
                Self::bench_as::<Vec<usize>>(&mut g, input);
            }
            Self::bench_as::<HashMap<usize, usize>>(&mut g, input);
            Self::bench_as::<BTreeMap<usize, usize>>(&mut g, input);
            Self::bench_as::<slab_iter::Slab<usize>>(&mut g, input);
            if include_optimized {
                Self::bench_as::<OptimizedSlab>(&mut g, input);
            }
            Self::bench_as::<slab::Slab<usize>>(&mut g, input);
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
    fn get(&self, i: usize) -> usize;

    #[inline]
    fn new_n(n: usize) -> Self {
        let mut s = Self::new();
        for i in 0..n {
            s.insert(i);
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
    fn remove(&mut self, _n: usize) {
        panic!("not supported");
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
    fn get(&self, i: usize) -> usize {
        self[i]
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
    fn get(&self, i: usize) -> usize {
        self[&i]
    }
}
impl BenchTarget for slab_iter::Slab<usize> {
    const NAME: &'static str = "slab_iter::Slab";

    #[inline]
    fn new() -> Self {
        slab_iter::Slab::new()
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
    fn get(&self, i: usize) -> usize {
        self[i]
    }
}

#[derive(Clone)]
struct OptimizedSlab(slab_iter::Slab<usize>);

impl BenchTarget for OptimizedSlab {
    const NAME: &'static str = "slab_iter::Slab(optimized)";

    #[inline]
    fn new() -> Self {
        OptimizedSlab(slab_iter::Slab::new())
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
    fn get(&self, i: usize) -> usize {
        self.0[i]
    }
    #[inline]
    fn optimize(&mut self) {
        self.0.optimize()
    }
}

impl BenchTarget for slab::Slab<usize> {
    const NAME: &'static str = "slab::Slab";

    #[inline]
    fn new() -> Self {
        slab::Slab::new()
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
