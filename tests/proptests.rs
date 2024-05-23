use proptest::collection::vec;
use proptest::prelude::*;
use slabmap::*;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use test_strategy::{proptest, Arbitrary};

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

impl Action {
    fn apply_slab_map(
        &self,
        s: &mut SlabMap<usize>,
        m: &mut HashMap<usize, usize>,
        index: usize,
        log: bool,
    ) {
        match self {
            Action::Insert => {
                let key = s.insert(0);
                m.insert(key, 0);
                if log {
                    eprintln!("insert({}) -> {}", 0, key);
                }
            }
            Action::Remove(key) => {
                let key = *key % (index + 2);
                let l = s.remove(key);
                let r = m.remove(&key);
                assert_eq!(l, r, "remove {}", key);
                if log {
                    eprintln!("remove({}) -> {:?}", key, l);
                }
            }
            Action::Clear => {
                s.clear();
                m.clear();
                if log {
                    eprintln!("clear");
                }
            }
            Action::Optimize => {
                s.optimize();
                if log {
                    eprintln!("optimize()");
                }
            }
            Action::Reserve(additional) => {
                s.reserve(*additional);
                assert!(s.capacity() >= s.len() + *additional);
                if log {
                    eprintln!("reserve({})", additional);
                }
            }
        }
        check(s.iter(), m);
    }
    fn apply_small_slab_map<const N: usize>(
        &self,
        s: &mut SmallSlabMap<usize, N>,
        m: &mut HashMap<usize, usize>,
        index: usize,
        log: bool,
    ) {
        match self {
            Action::Insert => {
                let key = s.insert(0);
                m.insert(key, 0);
                if log {
                    eprintln!("insert({}) -> {}", 0, key);
                }
            }
            Action::Remove(key) => {
                let key = *key % (index + 2);
                let l = s.remove(key);
                let r = m.remove(&key);
                assert_eq!(l, r, "remove {}", key);
                if log {
                    eprintln!("remove({}) -> {:?}", key, l);
                }
            }
            Action::Clear => {
                s.clear();
                m.clear();
                if log {
                    eprintln!("clear");
                }
            }
            Action::Optimize => {
                s.optimize();
                if log {
                    eprintln!("optimize()");
                }
            }
            Action::Reserve(additional) => {
                s.reserve(*additional);
                assert!(s.capacity() >= s.len() + *additional);
                if log {
                    eprintln!("reserve({})", additional);
                }
            }
        }
        check(s.iter(), m);
    }
}
fn check<'a>(s: impl Iterator<Item = (usize, &'a usize)>, m: &HashMap<usize, usize>) {
    let mut l: Vec<_> = s.map(|(key, value)| (key, *value)).collect();
    let mut r: Vec<_> = m.iter().map(|(key, value)| (*key, *value)).collect();
    l.sort();
    r.sort();
    assert_eq!(l, r, "items");
}

#[derive(Debug, Clone, Arbitrary)]
struct Actions {
    #[strategy(0..100usize)]
    _len: usize,
    #[strategy(prop::collection::vec(any_with::<Action>(Args { max_key: #_len } ), #_len))]
    actions: Vec<Action>,
}

fn test_slab_map(actions: &[Action], log: bool) {
    let mut s = SlabMap::new();
    let mut m = HashMap::new();
    for (c, a) in actions.iter().enumerate() {
        a.apply_slab_map(&mut s, &mut m, c, log);
    }
}

fn test_small_slab_map<const N: usize>(actions: &[Action], log: bool) {
    let mut s = SmallSlabMap::<_, N>::new();
    let mut m = HashMap::new();
    for (c, a) in actions.iter().enumerate() {
        a.apply_small_slab_map(&mut s, &mut m, c, log);
    }
}

#[proptest]
fn test_random_slab_map(actions: Actions) {
    test_slab_map(&actions.actions, false);
}

#[proptest]
fn test_random_small_slab_map_0(actions: Actions) {
    test_small_slab_map::<0>(&actions.actions, false);
}

#[proptest]
fn test_random_small_slab_map_1(actions: Actions) {
    test_small_slab_map::<1>(&actions.actions, false);
}

// #[proptest]
// fn test_random_small_slab_map_2(actions: Actions) {
//     test_small_slab_map::<2>(&actions.actions, false);
// }

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
    test_slab_map(&actions, false);
}
#[test]
fn test_x2() {
    use Action::*;
    let actions = vec![Insert, Insert, Insert, Remove(0), Remove(1)];
    test_slab_map(&actions, false);
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

#[proptest]
fn from_iter(#[strategy(vec((0..16usize, 0..10016usize), 0..16))] key_values: Vec<(usize, usize)>) {
    let a = SlabMap::from_iter(key_values.clone());
    let e: BTreeMap<usize, usize> = BTreeMap::from_iter(key_values);
    let a: Vec<_> = a.into_iter().collect();
    let e: Vec<_> = e.into_iter().collect();
    assert_eq!(a, e);
}
