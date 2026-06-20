-- accounts: used by 05_transactions to demo ACID transfers.
-- The CHECK constraint (credits >= 0) is what makes a bad transfer *fail*
-- instead of silently going negative.

CREATE TABLE IF NOT EXISTS accounts (
    user_id  UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    credits  BIGINT NOT NULL DEFAULT 0 CHECK (credits >= 0)
);