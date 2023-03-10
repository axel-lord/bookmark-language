use crate::{value::Value, variable, Result};
use serde::{Deserialize, Serialize};
use std::mem;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Mutating {
    Take(variable::Id),
    Assign(variable::Id),
    Swap(variable::Id),
}

impl Mutating {
    pub fn perform(
        self,
        return_value: Value,
        mut variables: variable::Map,
    ) -> Result<(Value, variable::Map)> {
        match self {
            Mutating::Take(id) => Ok((mem::take(variables.read_mut(id)?), variables)),
            Mutating::Assign(id) => {
                *variables.read_mut(id)? = return_value;
                Ok((Value::None, variables))
            }
            Mutating::Swap(id) => Ok((
                mem::replace(variables.read_mut(id)?, return_value),
                variables,
            )),
        }
    }
}
