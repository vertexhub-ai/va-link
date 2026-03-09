CREATE TABLE IF NOT EXISTS links (
    id          BIGSERIAL PRIMARY KEY,
    slug        TEXT        NOT NULL UNIQUE,
    target_url  TEXT        NOT NULL,
    title       TEXT,
    click_count BIGINT      NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_links_slug ON links (slug);