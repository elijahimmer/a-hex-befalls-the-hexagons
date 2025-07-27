//! The SQLite Database backend!
//!
//! TODO: Alert the user in the game when there is a database issue.
//!       Be it at startup or at runtime.
use super::*;

use bevy::prelude::*;
use const_format::formatcp;
use rusqlite::Connection;
use rusqlite::params;
use serde::{Serialize, de::DeserializeOwned};
use std::cmp::Ordering;
use thiserror::Error;

pub type DatabaseError = rusqlite::Error;

type Version = i64;

const DB_VERSION: Version = 7;

const ADD_SCHEMA: &str = formatcp!(
    r#"
    BEGIN TRANSACTION;

    CREATE TABLE Version(
      version INTEGER PRIMARY KEY
    ) STRICT;

    INSERT INTO Version VALUES({DB_VERSION});

    CREATE TABLE Keybinds(
        key   TEXT PRIMARY KEY,
        value TEXT
    ) STRICT;

    CREATE TABLE Style(
        key   TEXT PRIMARY KEY,
        value ANY
    ) STRICT;

    CREATE TABLE SaveGame(
        game_id     INTEGER PRIMARY KEY AUTOINCREMENT,
        created     TEXT DEFAULT CURRENT_TIMESTAMP,
        last_saved  TEXT,
        world_seed  INTEGER
    ) STRICT;

    CREATE TABLE Actor(
      game_id           INTEGER REFERENCES SaveGame(game_id),
      name              TEXT,
      party             TEXT,
      health_max        INTEGER,
      health_curr       INTEGER,
      attack_damage_min INTEGER,
      attack_damage_max INTEGER,
      hit_chance        INTEGER
    ) STRICT;

    COMMIT;
"#
);

pub struct Database {
    pub connection: Connection,
}

impl Database {
    pub fn open() -> Result<Self, OpenError> {
        let mut path = get_default_db_directory();
        path.push("database.sqlite");

        let exists = path.exists();
        let db = {
            let connection = match rusqlite::Connection::open(&path) {
                Ok(conn) => conn,
                Err(err) => {
                    warn!(
                        "Failed to open database at '{}' with error: {err}",
                        path.display()
                    );
                    rusqlite::Connection::open_in_memory()?
                }
            };
            Self {
                connection: connection,
            }
        };

        if exists {
            info!("Using existing database at '{}'!", path.display());
            match check_version(&db)? {
                VersionCompatability::Future(v) => {
                    error!(
                        "Database is from a future version {v} compared to current version {DB_VERSION}! You may be running an outdated version of the game"
                    );
                    return Err(OpenError::IncompatableVersion(v));
                }
                VersionCompatability::Same => {
                    info!("Database version is up to date!");
                }
                VersionCompatability::Migratable(v) => {
                    warn!(
                        "Database version is out dated, but migrateable. Backing up database then attempting migration..."
                    );

                    if let Err(err) = backup_database() {
                        error!("Failed to back up database before migration! {err}");
                        return Err(err.into());
                    }

                    info!("Backup successful! Migrating from database version {v} to {DB_VERSION}");

                    if let Err(err) = migrate_database(&db, v) {
                        error!("Failed to migrate database with error {err}");
                        return Err(err.into());
                    }

                    info!("Database migration successful!");
                }
                VersionCompatability::Incompatable(v) => {
                    error!(
                        "Database version is out dated, and not migrateable. Version is {v} when expected in the range of versions {MIN_VERSION_MIGRATEABLE} to {DB_VERSION}"
                    );
                    error!(
                        "Ask the developers to help get your data back, or on how to delete it to proceed!"
                    );
                    return Err(OpenError::IncompatableVersion(v));
                }
            }
        } else {
            info!("Database not found! Creating it at '{}'!", path.display());
            db.connection.execute_batch(ADD_SCHEMA)?;
        }

        info!("Running database validation checks.");
        match validate_schema(&db) {
            Ok(()) => {}
            Err(err) => {
                error!("Failed to validate SQLite Table with error {err}.");
                error!(
                    "Ask the developers to help get your data back, or on how to delete it to proceed!"
                );
                return Err(OpenError::ValidationFailed(err));
            }
        };
        info!("Passed database validation checks.");

        Ok(db)
    }

    pub fn get_kv<T>(&self, table: &str, key: &str, default: T) -> T
    where
        T: Serialize + DeserializeOwned + Clone,
    {
        let query = format!("SELECT value FROM {table} WHERE key = ?1");
        let ret = self
            .connection
            .prepare_cached(&query)
            .map(|mut q| q.query_row((key,), |row| row.get::<_, String>(0)));

        match ret {
            Err(err) => {
                warn!("Failed to read key '{key}' from table '{table}' with error: {err}");
                default
            }
            Ok(Err(err)) => {
                warn!(
                    "Error {err} when settings '{key}' in table '{table}' (this is expected first launch or after an update)."
                );
                if let Err(err) = self.set_kv(table, key, default.clone()) {
                    warn!(
                        "Failed to set key '{key}' in table '{table}' in database with error: {err}"
                    )
                }
                default
            }
            Ok(Ok(t)) => ron::from_str(&t).unwrap_or(default),
        }
    }

    pub fn set_kv<T: Serialize>(&self, table: &str, key: &str, value: T) -> Result<(), SetKvError> {
        let value = ron::to_string(&value)?;

        let query = format!("INSERT OR REPLACE INTO {table} VALUES (?1, ?2)");
        self.connection.execute(&query, params![key, value])?;

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum OpenError {
    #[error("Failed to backup database with {0}!")]
    BackupFailed(#[from] BackupError),
    #[error("Migration failed with {0}!")]
    MigrationFailed(#[from] MigrationError),
    #[error("Version Incompatable found version `{0}`!")]
    IncompatableVersion(Version),
    #[error("Version check failed with `{0}`")]
    CheckVersionError(#[from] CheckVersionError),
    #[error("Schema valdation failed with `{0}`")]
    ValidationFailed(#[from] ValidateSchemaError),
    #[error("SQLite error occured: `{0}`")]
    DatabaseError(#[from] DatabaseError),
}

#[derive(Error, Debug)]
pub enum CheckVersionError {
    #[error("No version found in database!")]
    VersionNotFound,
    #[error("Version table incompatable! Assuming data is invalid.")]
    IncompatableVersionTable,
    #[error("SQLite error occured: `{0}`")]
    DatabaseError(#[from] DatabaseError),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum VersionCompatability {
    Same,
    Future(Version),
    Migratable(Version),
    Incompatable(Version),
}

#[derive(Error, Debug)]
pub enum SetKvError {
    #[error("Failed to serialize value with error `{0}`")]
    SerializeError(#[from] ron::Error),
    #[error("SQLite error occured: `{0}`")]
    DatabaseError(#[from] DatabaseError),
}

fn check_version(db: &Database) -> Result<VersionCompatability, CheckVersionError> {
    let mut statement = db.connection.prepare("SELECT version FROM Version")?;

    let version = match statement.query_one([], |row| row.get::<_, Version>(0)) {
        Ok(v) => v,
        Err(err) => {
            warn!("Version entry not found in table with error: {err}");
            return Err(CheckVersionError::VersionNotFound);
        }
    };

    Ok(match version.cmp(&DB_VERSION) {
        Ordering::Equal => VersionCompatability::Same,
        Ordering::Less if version >= MIN_VERSION_MIGRATEABLE => {
            VersionCompatability::Migratable(version)
        }
        Ordering::Less => VersionCompatability::Incompatable(version),
        Ordering::Greater => VersionCompatability::Future(version),
    })
}

#[derive(Error, Debug)]
pub enum ValidateSchemaError {
    #[error("SQLite table '{0}' failed validation!")]
    Invalid(Box<str>),
    #[error("SQLite error occured: `{0}`")]
    DatabaseError(#[from] DatabaseError),
}

const _: () = assert!(DB_VERSION == 7, "UPDATE VALIDATE SCRIPT");
fn validate_schema(db: &Database) -> Result<(), ValidateSchemaError> {
    db.connection
        .execute_batch("PRAGMA integrity_check; PRAGMA optimize;")?;

    validate_table(db, "Version", &[("version", "INTEGER")])?;
    validate_table(db, "Keybinds", &[("key", "TEXT"), ("value", "TEXT")])?;
    validate_table(db, "Style", &[("key", "TEXT"), ("value", "ANY")])?;
    validate_table(
        db,
        "SaveGame",
        &[
            ("game_id", "INTEGER"),
            ("created", "TEXT"),
            ("last_saved", "TEXT"),
            ("world_seed", "INTEGER"),
        ],
    )?;
    validate_table(
        db,
        "Actor",
        &[
            ("game_id", "INTEGER"),
            ("name", "TEXT"),
            ("party", "TEXT"),
            ("health_max", "INTEGER"),
            ("health_curr", "INTEGER"),
            ("attack_damage_min", "INTEGER"),
            ("attack_damage_max", "INTEGER"),
            ("hit_chance", "INTEGER"),
            ("sprite_id", "INTEGER"),
        ],
    )?;
    validate_table(
        db,
        "Sprite",
        &[
            ("sprite_id", "INTEGER"),
            ("sprite_path", "TEXT"),
            ("normal_animation", "TEXT"),
            ("damaged_animation", "TEXT"),
            ("dead_animation", "TEXT"),
        ],
    )?;

    Ok(())
}

fn validate_table(
    db: &Database,
    table_name: &str,
    contents: &[(&str, &str)],
) -> Result<(), ValidateSchemaError> {
    // SAFETY: Use `format` here as it has to be the exact table name with no quotes.
    //         This name should also not be user input in any way.
    let query = format!("PRAGMA table_info({table_name});");

    let mut statement = db.connection.prepare(&query)?;
    let mut rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(1)?, row.get::<_, String>(2)?))
        })?
        .filter_map(|row| row.ok());

    let mut contents = contents.into_iter();

    while let (Some((expected_name, expected_ctype)), Some((name, ctype))) =
        (rows.next(), contents.next())
    {
        if *name != expected_name {
            error!(
                "SQLite table `{table_name}` found column `{name}` yet expected column `{expected_name}`"
            );
            return Err(ValidateSchemaError::Invalid(table_name.into()));
        }
        if *ctype != expected_ctype {
            error!(
                "SQLite table `{table_name}` found column `{name}` of type `{ctype}` yet expected the type `{expected_ctype}`"
            );
            return Err(ValidateSchemaError::Invalid(table_name.into()));
        }
    }

    if let Some((expected_name, expected_ctype)) = contents.next() {
        error!(
            "SQLite table `{table_name}` is missing column `{expected_name}` of type `{expected_ctype}`"
        );
        return Err(ValidateSchemaError::Invalid(table_name.into()));
    };

    if let Some((name, ctype)) = rows.next() {
        error!("SQLite table `{table_name}` has unexpected column `{name}` of type `{ctype}`");
        return Err(ValidateSchemaError::Invalid(table_name.into()));
    };

    Ok(())
}

#[derive(Error, Debug)]
pub enum BackupError {
    #[error("Failed to find migration script!")]
    NoMigrationScript,
    #[error("Failed to save backup with error: {0}")]
    FileError(#[from] std::io::Error),
}

/// Backs up the database to another file in the same directory with a timestamp in the name.
fn backup_database() -> Result<(), BackupError> {
    let mut db_path = get_default_db_directory();
    db_path.push("database.sqlite");

    let mut backup_path = get_default_db_directory();
    backup_path.push(format!(
        "{}_database.sqlite.backup",
        chrono::offset::Utc::now().format("%+")
    ));

    // While theoretically now bounded, this should be bounded in practice.
    while backup_path.exists() {
        backup_path.set_file_name(format!(
            "{}-database.sqlite.backup",
            chrono::offset::Utc::now().format("%+")
        ));
    }

    std::fs::copy(db_path, backup_path)?;

    Ok(())
}

#[derive(Error, Debug)]
pub enum MigrationError {
    #[error("Failed to find migration script!")]
    NoMigrationScript,
    #[error("SQLite error occured: `{0}`")]
    DatabaseError(#[from] DatabaseError),
    #[error("Migration script failed version update: `{0}`")]
    CheckVersionError(#[from] CheckVersionError),
}

const MIN_VERSION_MIGRATEABLE: Version = 3;
/// Make sure the migrations are set up properly
const _: () = assert!(DB_VERSION == 7, "UPDATE THE MIGRATION SCRIPT");

/// MAINTENANCE: UPDATE EVERY DATABASE UPDGRADE
fn migrate_database(db: &Database, from: Version) -> Result<(), MigrationError> {
    assert!((MIN_VERSION_MIGRATEABLE..DB_VERSION).contains(&from));

    let mut from = from;

    if from == 3 {
        db.connection.execute_batch(MIGRATE_FROM_3_TO_4)?;
        from = 4;
    }

    if from == 4 {
        db.connection.execute_batch(MIGRATE_FROM_4_TO_5)?;
        from = 5;
    }

    if from == 5 {
        db.connection.execute_batch(MIGRATE_FROM_5_TO_6)?;
        from = 6;
    }

    if from == 6 {
        db.connection.execute_batch(MIGRATE_FROM_6_TO_7)?;
        from = 7;
    }

    assert_eq!(
        from, DB_VERSION,
        "Failed to find migration script to migrate fully."
    );

    assert_eq!(
        check_version(db)?,
        VersionCompatability::Same,
        "Migration script failed to update version"
    );

    Ok(())
}

const MIGRATE_FROM_3_TO_4: &str = r#"
    BEGIN TRANSACTION;

    UPDATE Version SET version = 4;

    DROP TABLE Colors;

    CREATE TABLE Style(
        key   TEXT PRIMARY KEY,
        value ANY
    ) STRICT;

    COMMIT;
"#;

const MIGRATE_FROM_4_TO_5: &str = r#"
    BEGIN TRANSACTION;

    UPDATE Version SET version = 5;

    UPDATE Keybinds Set key1 = CONCAT('Some(', key1, ')') WHERE key1 IS NOT NULL;
    UPDATE Keybinds Set key2 = CONCAT('Some(', key2, ')') WHERE key2 IS NOT NULL;
    UPDATE Keybinds Set key1 = 'None' WHERE key1 IS NULL;
    UPDATE Keybinds Set key2 = 'None' WHERE key2 IS NULL;

    UPDATE Keybinds SET key1 = CONCAT('(', key1, ',', key2, ')');

    ALTER TABLE Keybinds DROP COLUMN key2;
    ALTER TABLE Keybinds RENAME COLUMN key1 TO value;
    ALTER TABLE Keybinds RENAME COLUMN keybind TO key;

    COMMIT;
"#;

const MIGRATE_FROM_5_TO_6: &str = r#"
    BEGIN TRANSACTION;

    UPDATE Version SET version = 6;

    CREATE TABLE SaveGame (
        game_id     INTEGER PRIMARY KEY AUTOINCREMENT,
        created     DATETIME DEFAULT CURRENT_TIMESTAMP,
        last_saved  DATETIME
    );

    CREATE TABLE Actor (
      game_id           INTEGER,
      name              TEXT,
      party             TEXT,
      health_max        INTEGER,
      health_curr       INTEGER,
      attack_damage_min INTEGER,
      attack_damage_max INTEGER,
      hit_chance        INTEGER,
      FOREIGN KEY(game_id) REFERENCES SaveGame(game_id)
    ) STRICT;

    COMMIT;
"#;

const MIGRATE_FROM_6_TO_7: &str = r#"
    BEGIN TRANSACTION;

    UPDATE Version SET version = 7;

    DROP TABLE KeyValue;

    ALTER TABLE SaveGame ADD COLUMN world_seed INTEGER;

    COMMIT;
"#;
