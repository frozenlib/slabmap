fn main() {
    use slabmap::SlabMap;

    let mut s = SlabMap::new();
    let key_a = s.insert("aaa");
    let key_b = s.insert("bbb");

    assert_eq!(s[key_a], "aaa");
    assert_eq!(s[key_b], "bbb");

    for (key, value) in &s {
        println!("{} -> {}", key, value);
    }

    assert_eq!(s.remove(key_a), Some("aaa"));
    assert_eq!(s.remove(key_a), None);
}
