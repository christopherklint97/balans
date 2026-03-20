use rust_decimal::Decimal;
use std::str::FromStr;

use super::encoding::decode_sie_bytes;
use super::types::*;

#[derive(Debug, thiserror::Error)]
pub enum SieParseError {
    #[error("Invalid SIE type: {0}")]
    InvalidType(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Parse error on line {line}: {message}")]
    ParseError { line: usize, message: String },
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
}

/// Parse SIE file from raw bytes (handles encoding automatically).
pub fn parse_sie(bytes: &[u8]) -> Result<SieFile, SieParseError> {
    let content = decode_sie_bytes(bytes);
    parse_sie_str(&content)
}

/// Parse SIE file from a UTF-8 string.
pub fn parse_sie_str(content: &str) -> Result<SieFile, SieParseError> {
    let mut file = SieFile::new(SieType::Type4);
    let mut current_voucher: Option<SieVoucher> = None;
    let mut in_voucher_block = false;

    for (line_num, raw_line) in content.lines().enumerate() {
        let line = raw_line.trim();

        if line.is_empty() {
            continue;
        }

        // Handle voucher block boundaries
        if line == "{" {
            in_voucher_block = true;
            continue;
        }
        if line == "}" {
            in_voucher_block = false;
            if let Some(voucher) = current_voucher.take() {
                file.vouchers.push(voucher);
            }
            continue;
        }

        // Lines inside a voucher block
        if in_voucher_block {
            if line.starts_with("#TRANS") {
                let trans = parse_trans_line(line, line_num)?;
                if let Some(ref mut v) = current_voucher {
                    v.lines.push(trans);
                }
            }
            // #RTRANS and #BTRANS are ignored (removed/added transactions)
            continue;
        }

        // Top-level tags
        if !line.starts_with('#') {
            continue;
        }

        let (tag, rest) = split_tag(line);

        match tag {
            "#FLAGGA" => {} // Ignored
            "#FORMAT" => {} // Encoding hint — we handle this in decode
            "#SIETYP" => {
                file.sie_type = SieType::from_str(rest)
                    .ok_or_else(|| SieParseError::InvalidType(rest.to_string()))?;
            }
            "#PROGRAM" => {
                let parts = parse_quoted_fields(rest);
                file.program = parts.first().cloned();
                file.program_version = parts.get(1).cloned();
            }
            "#GEN" => {
                let parts: Vec<&str> = rest.split_whitespace().collect();
                file.gen_date = parts.first().map(|s| format_sie_date(s));
            }
            "#FNAMN" => {
                file.company_name = Some(unquote(rest));
            }
            "#ORGNR" => {
                file.org_number = Some(rest.split_whitespace().next().unwrap_or("").to_string());
            }
            "#RAR" => {
                if let Some(fy) = parse_rar_line(rest, line_num)? {
                    file.fiscal_years.push(fy);
                }
            }
            "#KONTO" => {
                if let Some(account) = parse_konto_line(rest, line_num)? {
                    file.accounts.push(account);
                }
            }
            "#KTYP" => {
                // Update account type for existing account
                let parts: Vec<&str> = rest.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(num) = parts[0].parse::<i32>() {
                        if let Some(acc) = file.accounts.iter_mut().find(|a| a.number == num) {
                            acc.account_type = Some(parts[1].to_string());
                        }
                    }
                }
            }
            "#IB" => {
                if let Some(balance) = parse_balance_line(rest, BalanceType::Opening, line_num)? {
                    file.balances.push(balance);
                }
            }
            "#UB" => {
                if let Some(balance) = parse_balance_line(rest, BalanceType::Closing, line_num)? {
                    file.balances.push(balance);
                }
            }
            "#VER" => {
                current_voucher = Some(parse_ver_line(rest, line_num)?);
            }
            _ => {
                // Unknown tags are silently ignored per SIE spec
            }
        }
    }

    Ok(file)
}

/// Split a line into tag and rest. E.g. "#KONTO 1910 \"Kassa\"" -> ("#KONTO", "1910 \"Kassa\"")
fn split_tag(line: &str) -> (&str, &str) {
    if let Some(pos) = line.find(|c: char| c.is_whitespace()) {
        (&line[..pos], line[pos..].trim())
    } else {
        (line, "")
    }
}

/// Remove surrounding quotes from a string.
fn unquote(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// Parse quoted fields: "field1" "field2" -> vec!["field1", "field2"]
fn parse_quoted_fields(s: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut chars = s.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c == '"' {
            chars.next(); // consume opening quote
            let mut field = String::new();
            for c in chars.by_ref() {
                if c == '"' {
                    break;
                }
                field.push(c);
            }
            fields.push(field);
        } else if c.is_whitespace() {
            chars.next();
        } else {
            // Unquoted field — read until whitespace
            let mut field = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() {
                    break;
                }
                field.push(c);
                chars.next();
            }
            fields.push(field);
        }
    }

    fields
}

/// Format a SIE date (YYYYMMDD) to ISO (YYYY-MM-DD).
fn format_sie_date(date: &str) -> String {
    let d = date.trim();
    if d.len() == 8 {
        format!("{}-{}-{}", &d[0..4], &d[4..6], &d[6..8])
    } else {
        d.to_string()
    }
}

/// Parse SIE date back from ISO or YYYYMMDD format, returning YYYYMMDD.
pub fn to_sie_date(date: &str) -> String {
    date.replace('-', "")
}

fn parse_decimal(s: &str) -> Result<Decimal, SieParseError> {
    Decimal::from_str(s).map_err(|_| SieParseError::InvalidAmount(s.to_string()))
}

/// Parse #RAR line: index start_date end_date
fn parse_rar_line(rest: &str, line: usize) -> Result<Option<SieFiscalYear>, SieParseError> {
    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.len() < 3 {
        return Err(SieParseError::ParseError {
            line,
            message: "RAR requires index, start_date, end_date".into(),
        });
    }
    let index = parts[0]
        .parse::<i32>()
        .map_err(|_| SieParseError::ParseError {
            line,
            message: format!("Invalid RAR index: {}", parts[0]),
        })?;

    Ok(Some(SieFiscalYear {
        index,
        start_date: format_sie_date(parts[1]),
        end_date: format_sie_date(parts[2]),
    }))
}

/// Parse #KONTO line: number "name"
fn parse_konto_line(rest: &str, line: usize) -> Result<Option<SieAccount>, SieParseError> {
    let fields = parse_quoted_fields(rest);
    if fields.len() < 2 {
        return Err(SieParseError::ParseError {
            line,
            message: "KONTO requires number and name".into(),
        });
    }
    let number = fields[0]
        .parse::<i32>()
        .map_err(|_| SieParseError::ParseError {
            line,
            message: format!("Invalid account number: {}", fields[0]),
        })?;

    Ok(Some(SieAccount {
        number,
        name: fields[1].clone(),
        account_type: None,
    }))
}

/// Parse #IB/#UB line: year_index account_number amount
fn parse_balance_line(
    rest: &str,
    balance_type: BalanceType,
    line: usize,
) -> Result<Option<SieBalance>, SieParseError> {
    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.len() < 3 {
        return Err(SieParseError::ParseError {
            line,
            message: "Balance line requires year_index, account, amount".into(),
        });
    }

    let year_index = parts[0]
        .parse::<i32>()
        .map_err(|_| SieParseError::ParseError {
            line,
            message: format!("Invalid year index: {}", parts[0]),
        })?;
    let account_number = parts[1]
        .parse::<i32>()
        .map_err(|_| SieParseError::ParseError {
            line,
            message: format!("Invalid account number: {}", parts[1]),
        })?;
    let amount = parse_decimal(parts[2])?;

    Ok(Some(SieBalance {
        balance_type,
        year_index,
        account_number,
        amount,
    }))
}

/// Parse #VER line: series number date "description"
fn parse_ver_line(rest: &str, line: usize) -> Result<SieVoucher, SieParseError> {
    let fields = parse_quoted_fields(rest);
    if fields.len() < 3 {
        return Err(SieParseError::ParseError {
            line,
            message: "VER requires series, number, date".into(),
        });
    }

    let series = fields[0].clone();
    let number = fields[1].parse::<i32>().ok();
    let date = format_sie_date(&fields[2]);
    let description = fields.get(3).cloned().unwrap_or_default();

    Ok(SieVoucher {
        series,
        number,
        date,
        description,
        lines: Vec::new(),
    })
}

/// Parse #TRANS line: account {} amount [date] ["description"]
fn parse_trans_line(line: &str, line_num: usize) -> Result<SieTransaction, SieParseError> {
    // #TRANS account_number {dimension_list} amount [date] ["description"]
    let after_tag = line.strip_prefix("#TRANS").unwrap_or(line).trim();

    // Extract account number (first token)
    let mut parts_iter = after_tag.splitn(2, |c: char| c.is_whitespace() || c == '{');
    let account_str = parts_iter.next().unwrap_or("").trim();
    let account_number =
        account_str
            .parse::<i32>()
            .map_err(|_| SieParseError::ParseError {
                line: line_num,
                message: format!("Invalid account in TRANS: {account_str}"),
            })?;

    let remaining = parts_iter.next().unwrap_or("");

    // Skip the dimension list (everything between { and })
    let after_dims = if let Some(pos) = remaining.find('}') {
        remaining[pos + 1..].trim()
    } else {
        remaining.trim()
    };

    // Parse remaining fields
    let fields = parse_quoted_fields(after_dims);
    let amount = if let Some(amt_str) = fields.first() {
        parse_decimal(amt_str)?
    } else {
        return Err(SieParseError::ParseError {
            line: line_num,
            message: "TRANS requires amount".into(),
        });
    };

    let date = fields.get(1).map(|d| format_sie_date(d));
    let description = fields.get(2).cloned();

    Ok(SieTransaction {
        account_number,
        amount,
        date,
        description,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_SIE4: &str = r#"#FLAGGA 0
#FORMAT PC8
#SIETYP 4
#PROGRAM "Balans" "1.0"
#GEN 20260101
#FNAMN "Testföretaget AB"
#ORGNR 5561234567
#RAR 0 20250101 20251231
#RAR -1 20240101 20241231
#KONTO 1910 "Kassa"
#KONTO 3010 "Försäljning tjänster"
#KTYP 1910 T
#KTYP 3010 I
#IB 0 1910 50000.00
#UB 0 1910 75000.00
#VER A 1 20250115 "Kundbetalning"
{
  #TRANS 1910 {} 10000.00
  #TRANS 3010 {} -10000.00
}
#VER A 2 20250220 "Kontantförsäljning"
{
  #TRANS 1910 {} 5000.00
  #TRANS 3010 {} -5000.00
}
"#;

    #[test]
    fn test_parse_full_sie4() {
        let file = parse_sie_str(SAMPLE_SIE4).unwrap();

        assert_eq!(file.sie_type, SieType::Type4);
        assert_eq!(file.company_name.as_deref(), Some("Testföretaget AB"));
        assert_eq!(file.org_number.as_deref(), Some("5561234567"));
        assert_eq!(file.program.as_deref(), Some("Balans"));

        // Fiscal years
        assert_eq!(file.fiscal_years.len(), 2);
        assert_eq!(file.fiscal_years[0].index, 0);
        assert_eq!(file.fiscal_years[0].start_date, "2025-01-01");

        // Accounts
        assert_eq!(file.accounts.len(), 2);
        assert_eq!(file.accounts[0].number, 1910);
        assert_eq!(file.accounts[0].name, "Kassa");
        assert_eq!(file.accounts[0].account_type.as_deref(), Some("T"));

        // Balances
        assert_eq!(file.balances.len(), 2);
        let ib = &file.balances[0];
        assert_eq!(ib.balance_type, BalanceType::Opening);
        assert_eq!(ib.account_number, 1910);
        assert_eq!(ib.amount, Decimal::from_str("50000.00").unwrap());

        // Vouchers
        assert_eq!(file.vouchers.len(), 2);
        assert_eq!(file.vouchers[0].series, "A");
        assert_eq!(file.vouchers[0].number, Some(1));
        assert_eq!(file.vouchers[0].date, "2025-01-15");
        assert_eq!(file.vouchers[0].description, "Kundbetalning");
        assert_eq!(file.vouchers[0].lines.len(), 2);
        assert_eq!(
            file.vouchers[0].lines[0].amount,
            Decimal::from_str("10000.00").unwrap()
        );
        assert_eq!(
            file.vouchers[0].lines[1].amount,
            Decimal::from_str("-10000.00").unwrap()
        );
    }

    #[test]
    fn test_parse_preview() {
        let file = parse_sie_str(SAMPLE_SIE4).unwrap();
        let preview = SieImportPreview::from(&file);

        assert_eq!(preview.sie_type, "4");
        assert_eq!(preview.account_count, 2);
        assert_eq!(preview.voucher_count, 2);
        assert_eq!(preview.transaction_count, 4);
        assert_eq!(preview.opening_balances, 1);
        assert_eq!(preview.closing_balances, 1);
    }

    #[test]
    fn test_unquote() {
        assert_eq!(unquote("\"hello\""), "hello");
        assert_eq!(unquote("hello"), "hello");
        assert_eq!(unquote("\"\""), "");
    }

    #[test]
    fn test_format_sie_date() {
        assert_eq!(format_sie_date("20250115"), "2025-01-15");
        assert_eq!(format_sie_date("2025-01-15"), "2025-01-15");
    }

    #[test]
    fn test_parse_quoted_fields() {
        let fields = parse_quoted_fields("\"Balans\" \"1.0\"");
        assert_eq!(fields, vec!["Balans", "1.0"]);

        let fields = parse_quoted_fields("A 1 20250115 \"Kundbetalning\"");
        assert_eq!(fields, vec!["A", "1", "20250115", "Kundbetalning"]);
    }
}
