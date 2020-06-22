use proptest::prelude::*;
use slab_iter::*;
use std::collections::HashMap;
use std::fmt::Debug;

struct Tester<T> {
    slab: Slab<T>,
    m: HashMap<usize, T>,
    log: bool,
}

impl<T: Clone + Eq + PartialEq + Debug + PartialOrd + Ord> Tester<T> {
    pub fn new(log: bool) -> Self {
        Self {
            slab: Slab::new(),
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

#[derive(Debug, Clone)]
enum Action {
    Insert,
    Remove(usize),
    Clear,
    Optimize,
    Reserve(usize),
}
fn do_actions(actions: &[Action], log: bool) {
    let mut t = Tester::new(log);
    let mut c = 0;
    for a in actions {
        match a {
            Action::Insert => t.insert(c),
            Action::Remove(key) => t.remove(*key % (c + 2)),
            Action::Clear => t.clear(),
            Action::Optimize => t.optimize(),
            Action::Reserve(additional) => t.reserve(*additional),
        }
        t.check();
        c += 1;
    }
}

fn make_action(max_key: usize) -> impl Strategy<Value = Action> {
    prop_oneof![
        Just(Action::Insert),
        (0..max_key).prop_map(Action::Remove),
        Just(Action::Clear),
        Just(Action::Optimize),
        (0..100usize).prop_map(Action::Reserve)
    ]
}

fn make_actions() -> impl Strategy<Value = (usize, Vec<Action>)> {
    (1..100usize).prop_flat_map(|n| (Just(n), prop::collection::vec(make_action(n), n)))
}

proptest! {
    #[test]
    fn test_random(ref actions in make_actions()) {
        do_actions(&actions.1, false);
    }
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
    let mut s = Slab::new();
    s.insert(5);
    s.insert(10);

    assert_eq!(format!("{:?}", s), "{0: 5, 1: 10}");
}
