-- Initial schema. See docs/start.md for the concepts (Prompt, Test Case, Run, Experiment) and
-- STATE.md for why the shape below was chosen up front.
--
-- Only `runs` has repository code as of this migration (see storage/runs_repo.rs) — the other
-- tables exist now so the schema doesn't need a disruptive later migration, but get their own
-- Rust domain types and repositories when the features that use them are built (prompt saving,
-- side-by-side comparison, model list caching).

CREATE TABLE prompts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE test_cases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE experiments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL
);

-- A Run is Provider + Model + System Prompt + User Prompt + params, plus whatever the call
-- produced. `experiment_id` is nullable: a solo Playground run is a Run with no experiment, a
-- side-by-side comparison is several Runs sharing one experiment_id.
CREATE TABLE runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    experiment_id INTEGER REFERENCES experiments (id) ON DELETE SET NULL,
    provider TEXT NOT NULL,
    model_id TEXT NOT NULL,
    system_prompt TEXT NOT NULL,
    user_prompt TEXT NOT NULL,
    params_json TEXT NOT NULL,
    -- result_text/usage_json/duration_ms/ttft_ms are all NULL together when `error` is set, and
    -- all present together when the call succeeded (see domain::run::Run).
    result_text TEXT,
    usage_json TEXT,
    duration_ms INTEGER,
    ttft_ms INTEGER,
    cost_estimate_usd REAL,
    error TEXT,
    started_at TEXT NOT NULL
);

CREATE INDEX idx_runs_experiment_id ON runs (experiment_id);

-- Cache of models fetched from a provider's API (see the Model Registry section of
-- docs/start.md). `capabilities_json` mirrors domain::model::ModelCapabilities.
CREATE TABLE model_cache (
    provider TEXT NOT NULL,
    model_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    capabilities_json TEXT NOT NULL,
    context_window INTEGER,
    fetched_at TEXT NOT NULL,
    PRIMARY KEY (provider, model_id)
);
