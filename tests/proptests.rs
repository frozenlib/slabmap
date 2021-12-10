use proptest::prelude::*;
use slabmap::*;
use std::collections::HashMap;
use std::fmt::Debug;
use test_strategy::{proptest, Arbitrary};

struct Tester<T> {
    slab: SlabMap<T>,
    m: HashMap<usize, T>,
    log: bool,
}

impl<T: Clone + Eq + PartialEq + Debug + PartialOrd + Ord> Tester<T> {
    pub fn new(log: bool) -> Self {
        Self {
            slab: SlabMap::new(),
            m: HashMap::new(),
            log,
        }
    }
    pub fn insert(&mut self, value: T) {
        let key = self.slab.insert(value.clone());
        self.m.insert(key, value.clone());
        if self.log {
            eprintln!("insert({:?}) -> {}", value, key);
        }
    }
    pub fn remove(&mut self, key: usize) {
        let l = self.slab.remove(key);
        let r = self.m.remove(&key);
        assert_eq!(l, r, "remove {}", key);
        if self.log {
            eprintln!("remove({}) -> {:?}", key, l);
        }
    }
    pub fn clear(&mut self) {
        self.slab.clear();
        self.m.clear();
        if self.log {
            eprintln!("clear");
        }
    }
    pub fn optimize(&mut self) {
        self.slab.optimize();
        if self.log {
            eprintln!("optimize()");
        }
    }
    pub fn reserve(&mut self, additional: usize) {
        self.slab.reserve(additional);
        if self.log {
            eprintln!("reserve({})", additional);
        }
        assert!(self.slab.capacity() >= self.slab.len() + additional);
    }
    pub fn check(&mut self) {
        assert_eq!(self.slab.len(), self.m.len(), "len");
        let mut l: Vec<_> = self
            .slab
            .iter()
            .map(|(key, value)| (key, value.clone()))
            .collect();
        let mut l_mut: Vec<_> = self
            .slab
            .iter_mut()
            .map(|(key, value)| (key, value.clone()))
            .collect();
        let mut r: Vec<_> = self
            .m
            .iter()
            .map(|(key, value)| (*key, value.clone()))
            .collect();
        l.sort();
        l_mut.sort();
        r.sort();
        assert_eq!(l, r, "items");
        assert_eq!(l_mut, r, "items mut");

        if self.log {
            eprintln!("{:?}", l);
        }
    }
}

#[derive(Default)]
struct Args {
    max_key: usize,
}
#[derive(Debug, Clone, Arbitrary)]
#[arbitrary(args = Args)]
enum Action {
    #[weight(5)]
    Insert,
    #[weight(5)]
    Remove(#[strategy(0..args.max_key)] usize),
    Clear,
    Optimize,
    Reserve(#[strategy(0..100usize)] usize),
}

#[derive(Debug, Clone, Arbitrary)]
struct Actions {
    #[strategy(0..100usize)]
    len: usize,
    #[strategy(prop::collection::vec(any_with::<Action>(Args { max_key: #len } ), #len))]
    actions: Vec<Action>,
}

fn do_actions(actions: &[Action], log: bool) {
    let mut t = Tester::new(log);
    for (c, a) in actions.iter().enumerate() {
        match a {
            Action::Insert => t.insert(c),
            Action::Remove(key) => t.remove(*key % (c + 2)),
            Action::Clear => t.clear(),
            Action::Optimize => t.optimize(),
            Action::Reserve(additional) => t.reserve(*additional),
        }
        t.check();
    }
}

#[proptest]
fn test_random(actions: Actions) {
    do_actions(&actions.actions, false);
}

#[test]
fn test_x1() {
    use Action::*;
    let actions = vec![
        Insert,
        Insert,
        Insert,
        Insert,
        Insert,
        Remove(3),
        Remove(1),
        Remove(2),
        Remove(0),
        Insert,
        Insert,
        Insert,
        Insert,
        Insert,
    ];
    do_actions(&actions, false);
}
#[test]
fn test_x2() {
    use Action::*;
    let actions = vec![Insert, Insert, Insert, Remove(0), Remove(1)];
    do_actions(&actions, false);
}

#[test]
fn test_xx() {
    // use Action::*;
    // let actions = vec![Insert];
    // do_actions(&actions, true);
}

#[test]
fn debug() {
    let mut s = SlabMap::new();
    s.insert(5);
    s.insert(10);

    assert_eq!(format!("{:?}", s), "{0: 5, 1: 10}");
}
