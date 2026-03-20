-- Audit trail for all changes
CREATE TABLE IF NOT EXISTS audit_log (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,         -- 'company', 'voucher', 'account', 'fiscal_year'
    entity_id TEXT NOT NULL,
    action TEXT NOT NULL,              -- 'create', 'update', 'delete', 'close'
    details TEXT,                      -- JSON with change details
    created_at TEXT NOT NULL,
    created_by TEXT                    -- User identifier (future use)
);

CREATE INDEX IF NOT EXISTS idx_audit_entity ON audit_log(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_time ON audit_log(created_at);

-- Add SRU code column to accounts
ALTER TABLE accounts ADD COLUMN sru_code TEXT;

-- K2 eligibility data per fiscal year
CREATE TABLE IF NOT EXISTS k2_eligibility (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id),
    fiscal_year_id TEXT NOT NULL REFERENCES fiscal_years(id),
    average_employees INTEGER NOT NULL DEFAULT 0,
    balance_sheet_total INTEGER NOT NULL DEFAULT 0,   -- in ören
    net_revenue INTEGER NOT NULL DEFAULT 0,           -- in ören
    is_eligible INTEGER NOT NULL DEFAULT 1,
    reason TEXT,
    created_at TEXT NOT NULL,
    UNIQUE(company_id, fiscal_year_id)
);
