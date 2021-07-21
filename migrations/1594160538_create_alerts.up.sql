CREATE TABLE alerts (
  id         SERIAL PRIMARY KEY,
  url        TEXT NOT NULL,
  selector   TEXT NOT NULL,
  created_at timestamptz DEFAULT NOW(),
  updated_at timestamptz DEFAULT NOW(),
  creator_token TEXT REFERENCES tokens(token) NOT NULL
);
