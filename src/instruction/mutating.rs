use super::traits::Mutating;
use crate::{value::Value, variable, Result};
use serde::{Deserialize, Serialize};
use std::mem;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Take(pub variable::Id);
impl Mutating for Take {
    fn perform(
        self,
        _return_value: Value,
        mut variables: variable::Map,
    ) -> Result<(Value, variable::Map)> {
        Ok((mem::take(variables.read_mut(self.0)?), variables))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Assign(pub variable::Id);
impl Mutating for Assign {
    fn perform(
        self,
        return_value: Value,
        mut variables: variable::Map,
    ) -> Result<(Value, variable::Map)> {
        *variables.read_mut(self.0)? = return_value;
        Ok((Value::None, variables))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Swap(pub variable::Id);
impl Mutating for Swap {
    fn perform(
        self,
        return_value: Value,
        mut variables: variable::Map,
    ) -> Result<(Value, variable::Map)> {
        Ok((
            mem::replace(variables.read_mut(self.0)?, return_value),
            variables,
        ))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GetTake(pub variable::Id);
impl Mutating for GetTake {
    fn perform(
        self,
        return_value: Value,
        mut variables: variable::Map,
    ) -> Result<(Value, variable::Map)> {
        let key = variables.maybe_read(return_value)?;
        let value = variables.read_mut(self.0)?.get_take(key)?;
        Ok((value, variables))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MapAssign {
    map: variable::Id,
    key: Value,
}
impl Mutating for MapAssign {
    fn perform(
        self,
        return_value: Value,
        mut variables: variable::Map,
    ) -> Result<(Value, variable::Map)> {
        let Self { map, key } = self;
        let key = variables.maybe_read(key)?;
        *variables.read_mut(map)?.get_mut(key)? = return_value;
        Ok((Value::None, variables))
    }
}
