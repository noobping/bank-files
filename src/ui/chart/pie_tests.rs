use super::*;

#[test]
fn pie_denominator_uses_total_without_capacity() {
    assert_eq!(
        pie_denominator(Decimal::new(75, 0), None),
        Decimal::new(75, 0)
    );
}

#[test]
fn pie_denominator_uses_capacity_when_larger() {
    assert_eq!(
        pie_denominator(Decimal::new(75, 0), Some(Decimal::new(100, 0))),
        Decimal::new(100, 0)
    );
}

#[test]
fn pie_denominator_uses_total_when_capacity_is_smaller() {
    assert_eq!(
        pie_denominator(Decimal::new(125, 0), Some(Decimal::new(100, 0))),
        Decimal::new(125, 0)
    );
}

#[test]
fn pie_slices_sort_largest_first_with_stable_tiebreakers() {
    let mut slices = vec![
        (
            "b".to_string(),
            PieSlice::new("Beta".to_string(), Decimal::new(10, 0), String::new()),
        ),
        (
            "a".to_string(),
            PieSlice::new("Alpha".to_string(), Decimal::new(10, 0), String::new()),
        ),
        (
            "c".to_string(),
            PieSlice::new("Gamma".to_string(), Decimal::new(25, 0), String::new()),
        ),
    ];

    sort_pie_slices_largest_first(&mut slices);

    let keys = slices.into_iter().map(|(key, _)| key).collect::<Vec<_>>();
    assert_eq!(keys, ["c", "a", "b"]);
}
