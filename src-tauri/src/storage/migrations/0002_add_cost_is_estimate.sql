-- Distinguishes an exact cost (from our own hand-curated pricing.json, or OpenRouter's own real
-- per-request price when a Run itself used the OpenRouter provider) from an approximate one (a
-- different provider's cost estimated via OpenRouter's published price for what looks like the
-- equivalent model — see pricing::openrouter_fallback). The UI only shows a "≈" + disclosure
-- tooltip when this is set.
ALTER TABLE runs ADD COLUMN cost_is_estimate INTEGER NOT NULL DEFAULT 0;
