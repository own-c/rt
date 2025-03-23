use tauri_plugin_sql::{Migration, MigrationKind};

pub fn users_migrations() -> Vec<Migration> {
    vec![Migration {
        version: 1,
        description: "create_users_table",
        sql: r"
                CREATE TABLE IF NOT EXISTS twitch (
                    id TEXT,
                    username TEXT NOT NULL PRIMARY KEY,
                    avatar BLOB
                );
            ",
        kind: MigrationKind::Up,
    }]
}

pub fn feeds_migrations() -> Vec<Migration> {
    vec![Migration {
        version: 1,
        description: "create_feeds_table",
        sql: r"
                CREATE TABLE IF NOT EXISTS twitch (
                    username TEXT NOT NULL PRIMARY KEY,
                    started_at TEXT
                );
            ",
        kind: MigrationKind::Up,
    }]
}

pub fn emotes_migrations() -> Vec<Migration> {
    vec![Migration {
        version: 1,
        description: "create_emotes_table",
        sql: r"
                CREATE TABLE IF NOT EXISTS twitch (
                    username TEXT NOT NULL,
                    name TEXT NOT NULL,
                    url TEXT,
                    width INTEGER,
                    height INTEGER,
                    PRIMARY KEY (username, name)
                );
            ",
        kind: MigrationKind::Up,
    }]
}
