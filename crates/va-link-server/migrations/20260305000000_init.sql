-- Add migration script here
CREATE TABLE IF NOT EXISTS short_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    original_url TEXT NOT NULL,
    short_code VARCHAR(20) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    user_id UUID, -- Optional: Link to a user if authentication is implemented
    clicks BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_short_code ON short_links (short_code);
CREATE INDEX IF NOT EXISTS idx_original_url ON short_links (original_url);

CREATE TABLE IF NOT EXISTS clicks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    short_link_id UUID NOT NULL,
    clicked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address VARCHAR(45), -- IPv4 or IPv6
    user_agent TEXT,
    FOREIGN KEY (short_link_id) REFERENCES short_links(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_clicks_short_link_id ON clicks (short_link_id);