//! Moving an item up or down a list ("↑" / "↓" in the admin).

/// Where `id` is in `ids` and where it moves to (`direction` -1 is up, +1
/// down); None at either end of the list or when it isn't in it.
pub fn neighbour(ids: &[i64], id: i64, direction: i64) -> Option<(usize, usize)> {
    let index = ids.iter().position(|other| *other == id)?;
    let target = index.checked_add_signed(direction as isize)?;
    (target < ids.len()).then_some((index, target))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moves_stop_at_the_ends() {
        let ids = [7, 3, 9];
        assert_eq!(neighbour(&ids, 3, -1), Some((1, 0)));
        assert_eq!(neighbour(&ids, 3, 1), Some((1, 2)));
        assert_eq!(neighbour(&ids, 7, -1), None);
        assert_eq!(neighbour(&ids, 9, 1), None);
        assert_eq!(neighbour(&ids, 5, 1), None);
    }
}
