ALTER TABLE vouchers ADD COLUMN is_voided INTEGER NOT NULL DEFAULT 0;
ALTER TABLE vouchers ADD COLUMN voided_at TEXT;
ALTER TABLE vouchers ADD COLUMN corrected_by_voucher_id TEXT REFERENCES vouchers(id);
ALTER TABLE vouchers ADD COLUMN corrects_voucher_id TEXT REFERENCES vouchers(id);
