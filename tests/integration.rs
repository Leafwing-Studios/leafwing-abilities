#![cfg(test)]

#[test]
fn always_true() {
    use leafwing_abilities::utils::returns_true;

    assert!(returns_true());
}
