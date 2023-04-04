use rusqlite::*;
// use serde::{Deserialize, Serialize};
// use serde_json::Value::Object;

fn main() {
    let db = Connection::open_in_memory().unwrap();
    db.execute(
        "CREATE TABLE IF NOT EXISTS 'users' (`name` VARCHAR(100));",
        (),
    )
    .unwrap();
    db.execute("INSERT INTO 'users' (name) VALUES ('Alice');", ())
        .unwrap();
    db.execute("INSERT INTO 'users' (name) VALUES ('\"Bob\"');", ())
        .unwrap(); // notice the extra quotes

    let alice = serde_json::to_value::<String>("Alice".into()).unwrap();
    let bob = serde_json::to_value::<String>("Bob".into()).unwrap();

    let mut stmt = db.prepare("SELECT * FROM users WHERE name=?").unwrap();
    let alice_exists = stmt.exists(rusqlite::params_from_iter([alice])).unwrap();
    let bob_exists = stmt.exists(rusqlite::params_from_iter([bob])).unwrap();

    assert_eq!(alice_exists, true); // panics, but shouldn't
    assert_eq!(bob_exists, false); // same, should be false but it yields true
}
