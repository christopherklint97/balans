-- Add status column to users for approval workflow
-- Values: 'approved', 'pending', 'rejected'
-- Default 'approved' so existing users are grandfathered in
ALTER TABLE users ADD COLUMN status TEXT NOT NULL DEFAULT 'approved'
    CHECK (status IN ('approved', 'pending', 'rejected'));

CREATE INDEX idx_users_status ON users(status);

-- Backfill user_companies for existing users/companies
-- First user (admin) gets 'owner' role, others get 'member'
INSERT OR IGNORE INTO user_companies (user_id, company_id, role, created_at)
SELECT u.id, c.id,
    CASE WHEN u.role = 'admin' THEN 'owner' ELSE 'member' END,
    datetime('now')
FROM users u
CROSS JOIN companies c;
