use crate::error::BSResult;

const MIGRATIONS_TABLE_SQL: &str = r#"
CREATE TABLE migrations (
  id INTEGER PRIMARY KEY,
  idx INTEGER,
  date TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
"#;

const MIGRATIONS: &'static [&str] = &[
  r#"  
    CREATE TABLE selections (
      `id` INTEGER PRIMARY KEY,
      `path_hash` VARCHAR(128),
      `path` TEXT NOT NULL UNIQUE
    );
  "#,
  r#"
    CREATE TABLE selections_history (
      `id` INTEGER PRIMARY KEY,
      `url` TEXT NOT NULL,
      `src` VARCHAR(255),
      `tld` VARCHAR(255) NOT NULL,
      `weekday` INTEGER NOT NULL,
      `hour` INTEGER NOT NULL,
      `selection_id` INTEGER NOT NULL,
      `date` TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
      `count` INTEGER DEFAULT 1,
      FOREIGN KEY (selection_id) REFERENCES selections(id)
    );
  "#,
  r#"
    CREATE INDEX sel_browser_hash ON selections (path);
    CREATE INDEX sel_history_tld ON selections_history (tld);
    CREATE INDEX sel_date ON selections_history (date);
  "#, // Think about downgrade paths when adding migrations
];

pub fn migrate(conn: &mut rusqlite::Connection) -> BSResult<()> {
  let migration_tbl_exists = conn
    .prepare("SELECT name FROM sqlite_schema WHERE type = 'table' AND name = 'migrations'")?
    .exists([])?;
  if !migration_tbl_exists {
    conn.execute(MIGRATIONS_TABLE_SQL, [])?;
  }

  let last_migration_index: isize = conn
    .prepare("SELECT idx FROM migrations ORDER BY idx DESC LIMIT 1")?
    .query_row([], |row| row.get::<_, i64>(0))
    .unwrap_or_else(|_| -1)
    .try_into()
    .expect("Failed reading last migration from DB");
  
  let mut migration_index = if last_migration_index < 0 { 0 } else { last_migration_index };
  let already_migrated = last_migration_index + 1;
  MIGRATIONS
    .into_iter()
    .skip(already_migrated as usize)
    .try_for_each(|migration| -> BSResult<()> {
      let tx = conn.transaction()?;
      tx.execute(migration, [])?;
      tx.execute("INSERT INTO migrations (idx) VALUES (?)", [migration_index])?;
      tx.commit()?;
      
      migration_index += 1;
      Ok(())
    })?;

  Ok(())
}