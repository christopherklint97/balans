-- Fixed asset register (Anläggningstillgångar)
CREATE TABLE IF NOT EXISTS fixed_assets (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id),
    name TEXT NOT NULL,
    description TEXT,
    -- Asset classification
    asset_type TEXT NOT NULL CHECK (asset_type IN (
        'intangible',   -- Immateriella (10xx)
        'building',     -- Byggnader (11xx)
        'machinery',    -- Maskiner (12xx)
        'equipment',    -- Inventarier (12xx)
        'vehicle',      -- Bilar (12xx)
        'computer',     -- Datorer (12xx)
        'financial'     -- Finansiella (13xx)
    )),
    -- Acquisition
    acquisition_date TEXT NOT NULL,
    acquisition_cost INTEGER NOT NULL,      -- in ören
    -- Depreciation
    useful_life_months INTEGER NOT NULL,    -- K2: linear depreciation
    residual_value INTEGER NOT NULL DEFAULT 0, -- in ören
    depreciation_start_date TEXT NOT NULL,  -- When depreciation begins
    -- Account mapping
    asset_account INTEGER NOT NULL,         -- e.g., 1210 Maskiner
    depreciation_account INTEGER NOT NULL,  -- e.g., 1219 Ack avskr maskiner
    expense_account INTEGER NOT NULL,       -- e.g., 7832 Avskr maskiner
    -- Status
    is_disposed INTEGER NOT NULL DEFAULT 0,
    disposal_date TEXT,
    disposal_amount INTEGER DEFAULT 0,      -- in ören
    -- Metadata
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_fixed_assets_company ON fixed_assets(company_id);
