CREATE TABLE tokens (
  name       TEXT NOT NULL,
  token      TEXT PRIMARY KEY,
  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW()
);
