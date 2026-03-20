use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::money::Money;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Voucher {
    pub id: String,
    pub company_id: String,
    pub fiscal_year_id: String,
    pub voucher_number: i32,
    pub date: String,
    pub description: String,
    pub is_closing_entry: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct VoucherLine {
    pub id: String,
    pub voucher_id: String,
    pub account_number: i32,
    pub debit: Money,
    pub credit: Money,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VoucherWithLines {
    #[serde(flatten)]
    pub voucher: Voucher,
    pub lines: Vec<VoucherLine>,
}

#[derive(Debug, Deserialize)]
pub struct CreateVoucher {
    pub date: String,
    pub description: String,
    pub lines: Vec<CreateVoucherLine>,
}

#[derive(Debug, Deserialize)]
pub struct CreateVoucherLine {
    pub account_number: i32,
    pub debit: Money,
    pub credit: Money,
    pub description: Option<String>,
}
