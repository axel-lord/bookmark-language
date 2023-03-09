use crate::{value::Value, Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Id(usize);

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Map(Box<[Value]>);

impl Map {
    pub fn read(&self, id: Id) -> Result<&Value> {
        self.0.get(id.0).ok_or(Error::UnknownVariable(id))
    }

    pub fn read_mut(&mut self, id: Id) -> Result<&mut Value> {
        self.0.get_mut(id.0).ok_or(Error::UnknownVariable(id))
    }

    pub fn maybe_read(&self, value: Value) -> Result<Value> {
        if let Value::Id(id) = value {
            self.read(id).cloned()
        } else {
            Ok(value)
        }
    }
}

#[derive(Debug, Clone)]
pub struct MapBuilder(Vec<Value>);

impl Default for MapBuilder {
    fn default() -> Self {
        Self(vec![Value::None])
    }
}

impl MapBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn insert(&mut self, init: Value) -> Id {
        let id = Id(self.0.len());
        self.0.push(init);
        id
    }

    #[must_use]
    pub fn build(self) -> Map {
        Map(self.0.into_boxed_slice())
    }
}
