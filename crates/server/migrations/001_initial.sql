-- Companies
CREATE TABLE IF NOT EXISTS companies (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    org_number TEXT NOT NULL UNIQUE,
    company_form TEXT NOT NULL CHECK (company_form IN ('AB', 'HB', 'KB', 'EF', 'EK')),
    fiscal_year_start_month INTEGER NOT NULL DEFAULT 1 CHECK (fiscal_year_start_month BETWEEN 1 AND 12),
    address TEXT,
    postal_code TEXT,
    city TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Fiscal Years
CREATE TABLE IF NOT EXISTS fiscal_years (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id),
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    is_closed INTEGER NOT NULL DEFAULT 0,
    closed_at TEXT,
    created_at TEXT NOT NULL,
    UNIQUE(company_id, start_date)
);

-- Chart of Accounts (BAS Kontoplan)
CREATE TABLE IF NOT EXISTS accounts (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id),
    number INTEGER NOT NULL CHECK (number BETWEEN 1000 AND 9999),
    name TEXT NOT NULL,
    account_type TEXT NOT NULL CHECK (account_type IN ('asset', 'equity', 'liability', 'revenue', 'expense')),
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    UNIQUE(company_id, number)
);

-- Vouchers (Verifikationer)
CREATE TABLE IF NOT EXISTS vouchers (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id),
    fiscal_year_id TEXT NOT NULL REFERENCES fiscal_years(id),
    voucher_number INTEGER NOT NULL,
    date TEXT NOT NULL,
    description TEXT NOT NULL,
    is_closing_entry INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    UNIQUE(fiscal_year_id, voucher_number)
);

-- Voucher Lines (Verifikationsrader)
CREATE TABLE IF NOT EXISTS voucher_lines (
    id TEXT PRIMARY KEY,
    voucher_id TEXT NOT NULL REFERENCES vouchers(id) ON DELETE CASCADE,
    account_number INTEGER NOT NULL,
    debit INTEGER NOT NULL DEFAULT 0,
    credit INTEGER NOT NULL DEFAULT 0,
    description TEXT,
    CHECK (debit >= 0 AND credit >= 0)
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_fiscal_years_company ON fiscal_years(company_id);
CREATE INDEX IF NOT EXISTS idx_accounts_company ON accounts(company_id);
CREATE INDEX IF NOT EXISTS idx_vouchers_fiscal_year ON vouchers(fiscal_year_id);
CREATE INDEX IF NOT EXISTS idx_voucher_lines_voucher ON voucher_lines(voucher_id);
CREATE INDEX IF NOT EXISTS idx_voucher_lines_account ON voucher_lines(account_number);
