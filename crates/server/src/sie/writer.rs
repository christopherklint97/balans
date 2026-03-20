use rust_decimal::Decimal;
use std::fmt::Write;

use super::encoding::encode_to_latin1;
use super::parser::to_sie_date;
use super::types::*;

/// Write a SIE file to bytes (Latin-1 encoded).
pub fn write_sie(file: &SieFile) -> Vec<u8> {
    let content = write_sie_string(file);
    encode_to_latin1(&content)
}

/// Write a SIE file to a UTF-8 string (for testing/debugging).
pub fn write_sie_string(file: &SieFile) -> String {
    let mut out = String::new();

    // Header
    writeln!(out, "#FLAGGA 0").unwrap();
    writeln!(out, "#FORMAT PC8").unwrap();
    writeln!(out, "#SIETYP {}", file.sie_type.as_str()).unwrap();

    if let Some(ref program) = file.program {
        let version = file.program_version.as_deref().unwrap_or("1.0");
        writeln!(out, "#PROGRAM \"{program}\" \"{version}\"").unwrap();
    }

    if let Some(ref date) = file.gen_date {
        writeln!(out, "#GEN {}", to_sie_date(date)).unwrap();
    }

    if let Some(ref name) = file.company_name {
        writeln!(out, "#FNAMN \"{name}\"").unwrap();
    }

    if let Some(ref org) = file.org_number {
        writeln!(out, "#ORGNR {org}").unwrap();
    }

    // Fiscal years
    for fy in &file.fiscal_years {
        writeln!(
            out,
            "#RAR {} {} {}",
            fy.index,
            to_sie_date(&fy.start_date),
            to_sie_date(&fy.end_date)
        )
        .unwrap();
    }

    // Accounts
    for acc in &file.accounts {
        writeln!(out, "#KONTO {} \"{}\"", acc.number, acc.name).unwrap();
        if let Some(ref atype) = acc.account_type {
            writeln!(out, "#KTYP {} {atype}", acc.number).unwrap();
        }
    }

    // Balances
    for bal in &file.balances {
        let tag = match bal.balance_type {
            BalanceType::Opening => "#IB",
            BalanceType::Closing => "#UB",
        };
        writeln!(
            out,
            "{tag} {} {} {}",
            bal.year_index, bal.account_number, bal.amount
        )
        .unwrap();
    }

    // Vouchers (only for Type 4)
    if file.sie_type == SieType::Type4 {
        for voucher in &file.vouchers {
            let num_str = voucher
                .number
                .map(|n| n.to_string())
                .unwrap_or_default();
            writeln!(
                out,
                "#VER {} {} {} \"{}\"",
                voucher.series,
                num_str,
                to_sie_date(&voucher.date),
                voucher.description
            )
            .unwrap();
            writeln!(out, "{{").unwrap();
            for trans in &voucher.lines {
                write!(out, "\t#TRANS {} {{}} {}", trans.account_number, trans.amount).unwrap();
                if let Some(ref date) = trans.date {
                    write!(out, " {}", to_sie_date(date)).unwrap();
                }
                if let Some(ref desc) = trans.description {
                    write!(out, " \"{desc}\"").unwrap();
                }
                writeln!(out).unwrap();
            }
            writeln!(out, "}}").unwrap();
        }
    }

    out
}

/// Build a SIE Type 1 file (year-end balances only) from database data.
pub struct SieType1Builder {
    pub company_name: String,
    pub org_number: String,
    pub fiscal_year_start: String,
    pub fiscal_year_end: String,
    pub accounts: Vec<(i32, String, Option<String>)>,
    pub closing_balances: Vec<(i32, Decimal)>,
}

impl SieType1Builder {
    pub fn build(self) -> SieFile {
        let mut file = SieFile::new(SieType::Type1);
        file.program = Some("Balans".to_string());
        file.program_version = Some(env!("CARGO_PKG_VERSION").to_string());
        file.gen_date = Some(chrono::Utc::now().format("%Y-%m-%d").to_string());
        file.company_name = Some(self.company_name);
        file.org_number = Some(self.org_number);

        file.fiscal_years.push(SieFiscalYear {
            index: 0,
            start_date: self.fiscal_year_start,
            end_date: self.fiscal_year_end,
        });

        for (number, name, atype) in self.accounts {
            file.accounts.push(SieAccount {
                number,
                name,
                account_type: atype,
            });
        }

        for (account_number, amount) in self.closing_balances {
            if !amount.is_zero() {
                file.balances.push(SieBalance {
                    balance_type: BalanceType::Closing,
                    year_index: 0,
                    account_number,
                    amount,
                });
            }
        }

        file
    }
}

/// Build a SIE Type 4 file (full transactions) from database data.
pub struct SieType4Builder {
    pub company_name: String,
    pub org_number: String,
    pub fiscal_year_start: String,
    pub fiscal_year_end: String,
    pub accounts: Vec<(i32, String, Option<String>)>,
    pub opening_balances: Vec<(i32, Decimal)>,
    pub closing_balances: Vec<(i32, Decimal)>,
    pub vouchers: Vec<SieVoucher>,
}

impl SieType4Builder {
    pub fn build(self) -> SieFile {
        let mut file = SieFile::new(SieType::Type4);
        file.program = Some("Balans".to_string());
        file.program_version = Some(env!("CARGO_PKG_VERSION").to_string());
        file.gen_date = Some(chrono::Utc::now().format("%Y-%m-%d").to_string());
        file.company_name = Some(self.company_name);
        file.org_number = Some(self.org_number);

        file.fiscal_years.push(SieFiscalYear {
            index: 0,
            start_date: self.fiscal_year_start,
            end_date: self.fiscal_year_end,
        });

        for (number, name, atype) in self.accounts {
            file.accounts.push(SieAccount {
                number,
                name,
                account_type: atype,
            });
        }

        for (account_number, amount) in self.opening_balances {
            if !amount.is_zero() {
                file.balances.push(SieBalance {
                    balance_type: BalanceType::Opening,
                    year_index: 0,
                    account_number,
                    amount,
                });
            }
        }

        for (account_number, amount) in self.closing_balances {
            if !amount.is_zero() {
                file.balances.push(SieBalance {
                    balance_type: BalanceType::Closing,
                    year_index: 0,
                    account_number,
                    amount,
                });
            }
        }

        file.vouchers = self.vouchers;

        file
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sie::parser::parse_sie_str;
    use std::str::FromStr;

    #[test]
    fn test_write_type1() {
        let builder = SieType1Builder {
            company_name: "Test AB".to_string(),
            org_number: "5561234567".to_string(),
            fiscal_year_start: "2025-01-01".to_string(),
            fiscal_year_end: "2025-12-31".to_string(),
            accounts: vec![
                (1910, "Kassa".to_string(), Some("T".to_string())),
                (3010, "Försäljning".to_string(), Some("I".to_string())),
            ],
            closing_balances: vec![
                (1910, Decimal::from_str("75000.00").unwrap()),
                (3010, Decimal::from_str("-50000.00").unwrap()),
            ],
        };

        let file = builder.build();
        let output = write_sie_string(&file);

        assert!(output.contains("#SIETYP 1"));
        assert!(output.contains("#FNAMN \"Test AB\""));
        assert!(output.contains("#KONTO 1910 \"Kassa\""));
        assert!(output.contains("#UB 0 1910 75000.00"));
        // Type 1 should not contain vouchers
        assert!(!output.contains("#VER"));
    }

    #[test]
    fn test_roundtrip_type4() {
        let voucher = SieVoucher {
            series: "A".to_string(),
            number: Some(1),
            date: "2025-03-15".to_string(),
            description: "Testverifikation".to_string(),
            lines: vec![
                SieTransaction {
                    account_number: 1910,
                    amount: Decimal::from_str("5000.00").unwrap(),
                    date: None,
                    description: None,
                },
                SieTransaction {
                    account_number: 3010,
                    amount: Decimal::from_str("-5000.00").unwrap(),
                    date: None,
                    description: None,
                },
            ],
        };

        let builder = SieType4Builder {
            company_name: "Roundtrip AB".to_string(),
            org_number: "5561234567".to_string(),
            fiscal_year_start: "2025-01-01".to_string(),
            fiscal_year_end: "2025-12-31".to_string(),
            accounts: vec![
                (1910, "Kassa".to_string(), Some("T".to_string())),
                (3010, "Försäljning".to_string(), Some("I".to_string())),
            ],
            opening_balances: vec![(1910, Decimal::from_str("10000.00").unwrap())],
            closing_balances: vec![(1910, Decimal::from_str("15000.00").unwrap())],
            vouchers: vec![voucher],
        };

        let original = builder.build();
        let written = write_sie_string(&original);

        // Parse it back
        let parsed = parse_sie_str(&written).unwrap();

        assert_eq!(parsed.sie_type, SieType::Type4);
        assert_eq!(parsed.company_name.as_deref(), Some("Roundtrip AB"));
        assert_eq!(parsed.accounts.len(), 2);
        assert_eq!(parsed.vouchers.len(), 1);
        assert_eq!(parsed.vouchers[0].lines.len(), 2);
        assert_eq!(
            parsed.vouchers[0].lines[0].amount,
            Decimal::from_str("5000.00").unwrap()
        );

        // Balances
        let opening: Vec<_> = parsed
            .balances
            .iter()
            .filter(|b| b.balance_type == BalanceType::Opening)
            .collect();
        assert_eq!(opening.len(), 1);
        assert_eq!(opening[0].amount, Decimal::from_str("10000.00").unwrap());
    }
}
