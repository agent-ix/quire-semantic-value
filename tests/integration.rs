use quire_semantic_value::hello;

#[test]
fn hello_is_non_empty() {
    assert!(!hello().is_empty());
}
