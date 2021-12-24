/// Returns true!
///
/// A useless function used for testing that CI works.
///
/// # Examples
/// ```
/// # use leafwing_abilities::utils::returns_true;
/// assert!(returns_true());
/// ```
pub fn returns_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trivial_unit_test() {
        assert!(returns_true());
    }
}
