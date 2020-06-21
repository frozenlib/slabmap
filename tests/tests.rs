use proptest::prelude::*;
use slab_iter::*;
use std::collections::HashMap;
use std::fmt::Debug;

struct Tester<T> {
    slab: Slab<T>,
    m: HashMap<usize, T>,
}

impl<T: Clone + Eq + PartialEq + Debug + PartialOrd + Ord> Tester<T> {
    pub fn new() -> Self {
        Self {
            slab: Slab::new(),
            m: HashMap::new(),
        }
    }
    pub fn insert(&mut self, value: T, log: bool) {
        let key = self.slab.insert(value.clone());
        self.m.insert(key, value.clone());
        if log {
            eprintln!("insert({:?}) -> {}", value, key);
        }
    }
    pub fn remove(&mut self, key: usize, log: bool) {
        let l = self.slab.remove(key);
        let r = self.m.remove(&key);
        assert_eq!(l, r, "remove {}", key);
        if log {
            eprintln!("remove({}) -> {:?}", key, l);
        }
    }
    pub fn clear(&mut self, log: bool) {
        self.slab.clear();
        self.m.clear();
        if log {
            eprintln!("clear");
        }
    }
    pub fn optimize(&mut self, log: bool) {
        self.slab.optimize();
        if log {
            eprintln!("optimize()");
        }
    }
    pub fn check(&mut self, log: bool) {
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

        if log {
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
}
fn do_actions(actions: &[Action], log: bool) {
    let mut t = Tester::new();
    let mut c = 0;
    for a in actions {
        match a {
            Action::Insert => t.insert(c, log),
            Action::Remove(key) => t.remove(*key % (c + 2), log),
            Action::Clear => t.clear(log),
            Action::Optimize => t.optimize(log),
        }
        t.check(log);
        c += 1;
    }
}

fn make_action(max_key: usize) -> impl Strategy<Value = Action> {
    prop_oneof![
        Just(Action::Insert),
        (0..max_key).prop_map(Action::Remove),
        Just(Action::Clear),
        Just(Action::Optimize)
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
