fn main() {
    use slab_iter::Slab;

    let mut s = Slab::new();
    let key_a = s.insert("aaa");
    let key_b = s.insert("bbb");

    assert_eq!(s[key_a], "aaa");
    assert_eq!(s[key_b], "bbb");

    s.optimize();
    for (key, value) in &s {
        println!("{} -> {}", key, value);
    }

    let value = s.remove(key_a);
    assert_eq!(value, Some("aaa"));
}
