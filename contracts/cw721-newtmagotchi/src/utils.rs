pub fn calculate_total_cost(days_unfed: u64, feeding_cost_multiplier: u64) -> u64 {
    let mut total = 0;
    for day in 0..days_unfed {
        let cost_for_day = 1000 + (1000 * feeding_cost_multiplier * day / 1000);
        total += cost_for_day;
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_total_cost_0_days() {
        let result = calculate_total_cost(0, 100);
        assert_eq!(result, 0, "Total cost for 0 days should be 0");
    }

    #[test]
    fn test_calculate_total_cost_1_day() {
        let result = calculate_total_cost(1, 100);
        assert_eq!(result, 1000, "Total cost for 1 day should be 1000");
    }

    #[test]
    fn test_calculate_total_cost_5_days() {
        let result = calculate_total_cost(5, 100);
        assert_eq!(
            result,
            1000 + 1100 + 1200 + 1300 + 1400,
            "Total cost for 5 days should be 6000"
        );
    }

    #[test]
    fn test_calculate_total_cost_10_days() {
        let result = calculate_total_cost(10, 100);
        let expected: u64 = (0..10).map(|day| 1000 + (1000 * 100 * day / 1000)).sum();
        assert_eq!(result, expected, "Total cost for 10 days should be correct");
    }

    #[test]
    fn test_calculate_total_cost_with_different_multiplier() {
        let result = calculate_total_cost(5, 200);
        assert_eq!(
            result,
            1000 + 1200 + 1400 + 1600 + 1800,
            "Total cost for 5 days with multiplier 200 should be 7000"
        );
    }
}
