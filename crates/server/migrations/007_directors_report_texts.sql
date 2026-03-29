-- Store editable directors report (förvaltningsberättelse) texts per fiscal year
CREATE TABLE IF NOT EXISTS directors_report_texts (
    id TEXT PRIMARY KEY NOT NULL,
    fiscal_year_id TEXT NOT NULL UNIQUE REFERENCES fiscal_years(id),
    business_description TEXT,
    important_events TEXT,
    future_outlook TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
