use crate::{value::Value, variable, Result};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug};

pub trait Pure: Debug + Serialize + Deserialize<'static> + Clone + PartialEq {
    fn perform(self, return_value: Value) -> Result<Value>;
}

pub trait Loading: Debug + Serialize + Deserialize<'static> + Clone + PartialEq {
    fn perform(self, return_value: Value, loader: &dyn Loader) -> Result<Value>;
}

pub trait Reading: Debug + Serialize + Deserialize<'static> + Clone + PartialEq {
    fn perform(self, return_value: Value, variables: &variable::Map) -> Result<Value>;
}

pub trait Mutating: Debug + Serialize + Deserialize<'static> + Clone + PartialEq {
    fn perform(
        self,
        return_value: Value,
        variables: variable::Map,
    ) -> Result<(Value, variable::Map)>;
}

pub trait Meta: Debug + Serialize + Deserialize<'static> + Clone + PartialEq {
    fn perform(
        self,
        return_value: Value,
        variables: variable::Map,
        instruction_stack: super::Stack,
    ) -> Result<(Value, variable::Map, super::Stack)>;
}

type ExtraDebugFn = Box<dyn Fn(&mut fmt::Formatter<'_>) -> fmt::Result>;
pub trait External {
    fn perform(
        &self,
        return_value: Value,
        variables: variable::Map,
        instruction_stack: super::Stack,
    ) -> Result<(Value, variable::Map, super::Stack)>;

    fn extra_debug(&self) -> Option<ExtraDebugFn> {
        None
    }
}

pub trait Loader {
    fn load(&self, value: Value) -> Result<Value>;
}
