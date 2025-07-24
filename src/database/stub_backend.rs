use bevy::prelude::*;
use thiserror::Error;

use serde::{Serialize, de::DeserializeOwned};

#[derive(Error, Debug)]
pub enum DatabaseError {}

#[derive(Resource)]
pub struct Database;

impl Database {
    pub fn open() -> Result<Self, DatabaseError> {
        Ok(Self)
    }

    pub fn get_kv<T>(&self, table: &str, key: &str, default: T) -> T
    where
        T: Serialize + DeserializeOwned + Clone,
    {
        default
    }

    pub fn set_kv<T: Serialize>(&self, table: &str, key: &str, value: T) -> Result<(), SetKvError> {
        Ok(())
    }
}

pub type SetKvError = DatabaseError;
