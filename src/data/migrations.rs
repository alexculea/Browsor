use rusqlite::OptionalExtension;

use crate::error::BSResult;

const MIGRATIONS_TABLE_SQL: &str = r#"
CREATE TABLE migrations (
  id INTEGER PRIMARY KEY,
  index INTEGER
);
"#;

const MIGRATIONS: &'static [&str] = &[
  r#"  
    CREATE TABLE selections (
      `id` INTEGER PRIMARY KEY,
      `browser_path_hash` INT8 NOT NULL UNIQUE,
      `browser_path` TEXT NOT NULL UNIQUE,
      `selection_count` INTEGER,
      UNIQUE KEY `hash_index` (`browser_path_hash`) USING BTREE,
    )
  "#,
  r#"
    CREATE TABLE history_selections (
      `id` INTEGER PRIMARY KEY,
      `base_domain_hash` INT8 NOT NULL,
      `fqdn` VARCHAR(255) NOT NULL,
      `uri` VARCHAR(255),
      `src` VARCHAR(255),
      `weekday` INTEGER NOT NULL,
      `hour` INTEGER NOT NULL,
      `selection_id` INTEGER NOT NULL,
      UNIQUE KEY `hash_index` (`base_domain_hash`) USING BTREE,
      FOREIGN KEY (selection_id) REFERENCES selections(id)
    );
  "#, // Convetion: New columns from here may not be not null to keep downgrades working with the migrated schema
];

pub fn migrate(conn: &mut rusqlite::Connection) -> crate::error::BSResult<()> {
  let migration_tbl_exists = conn
    .prepare("SELECT name FROM sqlite_schema WHERE type = 'table' AND name = 'migrations'")?
    .exists([])?;
  if !migration_tbl_exists {
    conn.execute(MIGRATIONS_TABLE_SQL, [])?;
  }

  let last_migration = conn
    .prepare("SELECT id FROM migrations ORDER BY id DESC LIMIT 1")?
    .query_row([], |row| row.get::<_, i64>(0))
    .optional()
    .unwrap_or_else(|_| Some(0))
    .unwrap();
  
  let mut migration_index = 0;
  MIGRATIONS
    .into_iter()
    .skip(last_migration.try_into().unwrap())
    .try_for_each(|migration| -> BSResult<()> {
      let tx = conn.transaction()?;
      tx.execute(migration, [])?;
      tx.execute("INSERT INTO migrations (index) VALUES (?)", [migration_index])?;
      tx.commit()?;
      
      migration_index += 1;
      Ok(())
    })?;

  Ok(())
}