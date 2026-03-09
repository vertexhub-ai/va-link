-- Migration: 001_create_links
-- Creates the links table for va-link short URL service

CREATE TABLE IF NOT EXISTS links (
    id          BIGSERIAL PRIMARY KEY,
    slug        TEXT        NOT NULL,
    target_url  TEXT        NOT NULL,
    title       TEXT,
    click_count BIGINT      NOT NULL DEFAULT 0,
    active      BOOLEAN     NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS links_slug_idx ON links (slug);
CREATE INDEX IF NOT EXISTS links_active_idx ON links (active);
CREATE INDEX IF NOT EXISTS links_created_at_idx ON links (created_at DESC);