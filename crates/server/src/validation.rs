/// Validate a Swedish organisationsnummer using the Luhn algorithm.
/// Accepts formats: "NNNNNNNNNN" or "NNNNNN-NNNN" (10 digits).
pub fn validate_organisationsnummer(org_nr: &str) -> bool {
    let digits: String = org_nr.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.len() != 10 {
        return false;
    }

    // The third digit must be >= 2 for organisationsnummer
    let third: u32 = digits.chars().nth(2).unwrap().to_digit(10).unwrap();
    if third < 2 {
        return false;
    }

    // Luhn check on all 10 digits
    let sum: u32 = digits
        .chars()
        .enumerate()
        .map(|(i, c)| {
            let mut d = c.to_digit(10).unwrap();
            if i % 2 == 0 {
                d *= 2;
                if d > 9 {
                    d -= 9;
                }
            }
            d
        })
        .sum();

    sum % 10 == 0
}

/// Validate that a BAS account number is in the valid range (1000-9999).
pub fn validate_bas_account_number(number: i32) -> bool {
    (1000..=9999).contains(&number)
}

/// Get the account class (1-8) from a BAS account number.
pub fn account_class(number: i32) -> Option<i32> {
    if validate_bas_account_number(number) {
        Some(number / 1000)
    } else {
        None
    }
}

/// Determine account type from BAS account number.
pub fn account_type_from_number(number: i32) -> &'static str {
    match number / 1000 {
        1 => "asset",
        2 => {
            // 20xx = equity, 21xx-22xx = untaxed reserves, 23xx-29xx = liability
            if number < 2100 {
                "equity"
            } else {
                "liability"
            }
        }
        3 => "revenue",
        4..=7 => "expense",
        8 => {
            // 8xxx: financial items, appropriations, tax
            // 83xx = financial income, 84xx = financial expense, etc.
            if (8300..8400).contains(&number) || (8900..9000).contains(&number) {
                "revenue"
            } else {
                "expense"
            }
        }
        _ => "expense",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_org_numbers() {
        // Known valid test org numbers
        assert!(validate_organisationsnummer("5561234567")); // fictional but valid Luhn
        assert!(validate_organisationsnummer("556123-4567"));
    }

    #[test]
    fn test_invalid_org_numbers() {
        assert!(!validate_organisationsnummer("5561234568")); // wrong check digit
        assert!(!validate_organisationsnummer("123")); // too short
        assert!(!validate_organisationsnummer("1234567890")); // third digit < 2
    }

    #[test]
    fn test_account_number_validation() {
        assert!(validate_bas_account_number(1910));
        assert!(validate_bas_account_number(3010));
        assert!(!validate_bas_account_number(999));
        assert!(!validate_bas_account_number(10000));
    }

    #[test]
    fn test_account_class() {
        assert_eq!(account_class(1910), Some(1));
        assert_eq!(account_class(2440), Some(2));
        assert_eq!(account_class(3010), Some(3));
        assert_eq!(account_class(999), None);
    }

    #[test]
    fn test_account_type() {
        assert_eq!(account_type_from_number(1910), "asset");
        assert_eq!(account_type_from_number(2010), "equity");
        assert_eq!(account_type_from_number(2440), "liability");
        assert_eq!(account_type_from_number(3010), "revenue");
        assert_eq!(account_type_from_number(5010), "expense");
        assert_eq!(account_type_from_number(7010), "expense");
    }
}
