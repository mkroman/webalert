use webalert::*;

#[test]
fn has_migrations() {
    let (up, down) = migrations::init();
    assert!(!up.is_empty());
    assert!(!down.is_empty());
}
