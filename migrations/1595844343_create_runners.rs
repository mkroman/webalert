use crate::{down, up};

up!(r#"
    CREATE TABLE runners (
      id       SERIAL PRIMARY KEY,
      name     TEXT NOT NULL,
      hostname TEXT NOT NULL,
      arch     TEXT NOT NULL,
      created_at timestamptz DEFAULT NOW(),
      updated_at timestamptz DEFAULT NOW()
    );
"#);

down!(
    r#"
    DROP TABLE runners;
    "#
);
