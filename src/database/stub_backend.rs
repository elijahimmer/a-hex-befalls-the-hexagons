use bevy::prelude::*;
use thiserror::Error;

use serde::{Serialize, de::DeserializeOwned};

#[derive(Error, Debug)]
pub enum Error {}
pub type SetKvError = Error;

#[derive(Resource)]
pub struct Database;

impl Database {
    pub fn open() -> Result<Self, Error> {
        Ok(Self)
    }

    pub fn get_kv<T>(&self, _: &str, _: &str, default: T) -> T
    where
        T: Serialize + DeserializeOwned + Clone,
    {
        default
    }

    pub fn set_kv<T: Serialize>(&self, _: &str, _: &str, _: T) -> Result<(), SetKvError> {
        Ok(())
    }
}
