use rust_decimal::Decimal;
/// Represents a complete SIE file.
#[derive(Debug, Clone)]
pub struct SieFile {
    pub sie_type: SieType,
    pub program: Option<String>,
    pub program_version: Option<String>,
    pub gen_date: Option<String>,
    pub company_name: Option<String>,
    pub org_number: Option<String>,
    pub fiscal_years: Vec<SieFiscalYear>,
    pub accounts: Vec<SieAccount>,
    pub balances: Vec<SieBalance>,
    pub vouchers: Vec<SieVoucher>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SieType {
    Type1, // Year-end balances (Årssaldon)
    Type2, // Period balances (Periodsaldon)
    Type4, // Transactions (Transaktioner)
}

impl SieType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim() {
            "1" => Some(Self::Type1),
            "2" => Some(Self::Type2),
            "4" => Some(Self::Type4),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Type1 => "1",
            Self::Type2 => "2",
            Self::Type4 => "4",
        }
    }
}

/// Fiscal year definition from #RAR
#[derive(Debug, Clone)]
pub struct SieFiscalYear {
    /// 0 = current year, -1 = previous year, etc.
    pub index: i32,
    pub start_date: String,
    pub end_date: String,
}

/// Account definition from #KONTO / #KTYP
#[derive(Debug, Clone)]
pub struct SieAccount {
    pub number: i32,
    pub name: String,
    /// T=Tillgång, S=Skuld, K=Kostnad, I=Intäkt
    pub account_type: Option<String>,
}

/// Balance from #IB (opening) or #UB (closing)
#[derive(Debug, Clone)]
pub struct SieBalance {
    pub balance_type: BalanceType,
    /// 0 = current year, -1 = previous, etc.
    pub year_index: i32,
    pub account_number: i32,
    pub amount: Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BalanceType {
    /// Ingående balans (opening balance)
    Opening,
    /// Utgående balans (closing balance)
    Closing,
}

/// Voucher from #VER / #TRANS
#[derive(Debug, Clone)]
pub struct SieVoucher {
    pub series: String,
    pub number: Option<i32>,
    pub date: String,
    pub description: String,
    pub lines: Vec<SieTransaction>,
}

/// Transaction line from #TRANS
#[derive(Debug, Clone)]
pub struct SieTransaction {
    pub account_number: i32,
    pub amount: Decimal,
    pub date: Option<String>,
    pub description: Option<String>,
}

impl SieFile {
    pub fn new(sie_type: SieType) -> Self {
        Self {
            sie_type,
            program: None,
            program_version: None,
            gen_date: None,
            company_name: None,
            org_number: None,
            fiscal_years: Vec::new(),
            accounts: Vec::new(),
            balances: Vec::new(),
            vouchers: Vec::new(),
        }
    }
}

/// Summary of a parsed SIE file for preview before import.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SieImportPreview {
    pub sie_type: String,
    pub company_name: Option<String>,
    pub org_number: Option<String>,
    pub fiscal_years: Vec<SieFiscalYearPreview>,
    pub account_count: usize,
    pub voucher_count: usize,
    pub transaction_count: usize,
    pub opening_balances: usize,
    pub closing_balances: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SieFiscalYearPreview {
    pub index: i32,
    pub start_date: String,
    pub end_date: String,
}

impl From<&SieFile> for SieImportPreview {
    fn from(file: &SieFile) -> Self {
        Self {
            sie_type: file.sie_type.as_str().to_string(),
            company_name: file.company_name.clone(),
            org_number: file.org_number.clone(),
            fiscal_years: file
                .fiscal_years
                .iter()
                .map(|fy| SieFiscalYearPreview {
                    index: fy.index,
                    start_date: fy.start_date.clone(),
                    end_date: fy.end_date.clone(),
                })
                .collect(),
            account_count: file.accounts.len(),
            voucher_count: file.vouchers.len(),
            transaction_count: file.vouchers.iter().map(|v| v.lines.len()).sum(),
            opening_balances: file
                .balances
                .iter()
                .filter(|b| b.balance_type == BalanceType::Opening)
                .count(),
            closing_balances: file
                .balances
                .iter()
                .filter(|b| b.balance_type == BalanceType::Closing)
                .count(),
        }
    }
}
