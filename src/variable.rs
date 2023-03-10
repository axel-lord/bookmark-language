use std::sync::Arc;

use crate::{value::Value, Error, Result};
use serde::{Deserialize, Serialize};
use tap::Pipe;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Id(IdInternal);

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
enum IdInternal {
    Rw(usize),
    Ro(usize),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Map(Box<[Value]>, Arc<[Value]>);

impl Default for Map {
    fn default() -> Self {
        Self(Box::default(), Arc::new([]))
    }
}

impl Map {
    pub fn read(&self, id: Id) -> Result<&Value> {
        match id.0 {
            IdInternal::Rw(id_index) => self.0.get(id_index).ok_or(Error::UnknownVariable(id)),
            IdInternal::Ro(id_index) => self.1.get(id_index).ok_or(Error::UnknownVariable(id)),
        }
    }

    pub fn read_mut(&mut self, id: Id) -> Result<&mut Value> {
        match id.0 {
            IdInternal::Rw(id_index) => self.0.get_mut(id_index).ok_or(Error::UnknownVariable(id)),
            IdInternal::Ro(_) => Err(Error::WriteToReadOnly(id)),
        }
    }

    pub fn maybe_read(&self, value: Value) -> Result<Value> {
        if let Value::Id(id) = value {
            self.read(id).cloned()
        } else {
            Ok(value)
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MapBuilder(Vec<Value>, Vec<Value>);

impl MapBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn insert_rw(&mut self, init: Value) -> Id {
        let id = self.0.len().pipe(IdInternal::Rw).pipe(Id);
        self.0.push(init);
        id
    }

    #[must_use]
    pub fn reserve_rw(&mut self) -> Id {
        self.insert_rw(Value::None)
    }

    #[must_use]
    pub fn insert_ro(&mut self, init: Value) -> Id {
        let id = self.1.len().pipe(IdInternal::Ro).pipe(Id);
        self.1.push(init);
        id
    }

    #[must_use]
    pub fn reserve_ro(&mut self) -> Id {
        self.insert_ro(Value::None)
    }

    pub fn set(&mut self, id: Id, value: Value) -> Result<()> {
        *match id.0 {
            IdInternal::Rw(i) => self.0.get_mut(i),
            IdInternal::Ro(i) => self.1.get_mut(i),
        }
        .ok_or(Error::UnknownVariable(id))? = value;
        Ok(())
    }

    #[must_use]
    pub fn build(self) -> Map {
        Map(
            self.0.into_boxed_slice(),
            self.1.into_boxed_slice().pipe(Arc::from),
        )
    }
}
