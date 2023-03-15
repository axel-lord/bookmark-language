use crate::{value::Value, variable, Result};
use std::fmt;

type ExtraDebugFn = Box<dyn Fn(&mut fmt::Formatter<'_>) -> fmt::Result>;
pub trait External {
    fn perform(
        &self,
        return_value: Value,
        variables: variable::Map,
        instruction_stack: super::Stack,
    ) -> Result<(Value, variable::Map, super::Stack)>;

    fn perform_tup(
        &self,
        tup: (Value, variable::Map, super::Stack),
    ) -> Result<(Value, variable::Map, super::Stack)> {
        self.perform(tup.0, tup.1, tup.2)
    }

    fn extra_debug(&self) -> Option<ExtraDebugFn> {
        None
    }
}

pub trait Loader {
    fn load(&self, value: Value) -> Result<Value>;
}
