//! TODO: Add wasm local storage backend

#[cfg(not(feature = "sqlite"))]
mod stub_backend;
#[cfg(not(feature = "sqlite"))]
pub use stub_backend::*;

#[cfg(feature = "sqlite")]
mod sqlite_backend;
#[cfg(feature = "sqlite")]
pub use sqlite_backend::*;

use bevy::prelude::*;
#[cfg(feature = "sqlite")]
use std::path::PathBuf;

pub struct DatabasePlugin;

impl Plugin for DatabasePlugin {
    fn build(&self, app: &mut App) {
        app.insert_non_send_resource(
            Database::open()
                .inspect_err(|e| error!("Failed to open database with: {e}"))
                .unwrap(),
        );
    }
}

pub trait FromDatabase {
    /// Cannot fail, must resort to defaults.
    fn from_database(database: &Database) -> Self;
}

pub trait ToDatabase {
    fn to_database(&self, database: &Database) -> Result<(), Error>;
}

/// Gets the default database path in the user's home directory
/// This isn't only for sqlite, but for anything that needs it.
#[cfg(feature = "sqlite")]
fn get_default_db_directory() -> PathBuf {
    let project_dir =
        directories::ProjectDirs::from("com", "TeamCounterSpell", "A-Hex-Befalls-The-Hexagons");
    match project_dir.as_ref().map(|d| d.config_dir()) {
        Some(config_dir) if config_dir.is_dir() => config_dir.into(),
        Some(config_dir) => {
            info!("Config directory not found! creating directory!");
            std::fs::DirBuilder::new()
                .recursive(true)
                .create(config_dir)
                .and(Ok(config_dir.into()))
                .inspect_err(|e| warn!("Failed to create config directory with: {e}. Resorting to using local directory!"))
                .unwrap_or("".into())
        }
        Option::None => "".into(),
    }
}
