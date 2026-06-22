-- The users table for the sqlx-axum starter.
-- This is the schema your CRUD handlers will operate on. It is identical to
-- the one in ../sqlx-lab so you can compare your implementation against the
-- reference solution once you're done.
CREATE TABLE IF NOT EXISTS users (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name       VARCHAR(255) NOT NULL,
    email      VARCHAR(255) NOT NULL UNIQUE,
    bio        TEXT,
    is_active  BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at DESC);

-- Keep updated_at in sync on row changes (used by an UPDATE handler).

-- PGSQL SUPPORTS FUNCTION AS WELL, here updated_at is being set at the db level instead of backend. 
-- We sometimes miss updating updated_at field in columns. Doing it here makes the updates clean.
CREATE OR REPLACE FUNCTION touch_updated_at() RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS users_touch_updated_at ON users;
CREATE TRIGGER users_touch_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION touch_updated_at();