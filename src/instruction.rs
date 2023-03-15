use std::{
    fmt::{self, Debug},
    sync::Arc,
};

use crate::{value::Value, variable, Error, Result};
use derive_more::IsVariant;
use serde::{Deserialize, Serialize};

pub mod loading;
pub mod meta;
pub mod mutating;
pub mod pure;
pub mod reading;
pub mod traits;

mod set_macro;
mod stack;

pub use stack::Stack;

use self::traits::Loader;

#[derive(Debug, Deserialize, Serialize, Clone, Default, IsVariant, PartialEq)]
pub enum Instruction {
    #[default]
    Noop,
    Pure(Pure),
    Reading(Reading),
    Mutating(Mutating),
    Meta(Meta),
    Loading(Loading),
    #[serde(skip)]
    External(External),
}

set_macro::instr! {
Pure(rval: Value) -> Value: [
    Sleep,
    Debug,
    Cond,
    Put,
    Coerce,
    Parse,
    Op,
    ToFallible,
    ToInfallible,
    Not,
],
Reading(rval: Value, map: &variable::Map) -> Value: [
    Clone,
    GetClone,
    OpClone,
],
Mutating(rval: Value, map: variable::Map) -> (Value, variable::Map): [
    Take,
    Assign,
    Swap,
    GetTake,
    MapAssign,
    OpTake,
],
Meta(rval: Value, map: variable::Map, stack: Stack) -> (Value, variable::Map, Stack): [
    List,
    Return,
    Perform,
    PerformClone,
    PerformTake,
],
Loading(rval: Value, loader: &dyn Loader) -> Value: [
    Program,
    Load,
]
}

impl Instruction {
    #[must_use]
    pub fn flatten(self) -> Self {
        let Instruction::Meta(Meta::List(meta::List(instr_vec))) = self else {
            return self;
        };

        let mut out_instrs = Vec::new();
        let mut instr_stack = instr_vec.into_iter().rev().collect::<Vec<_>>();

        while let Some(instr) = instr_stack.pop() {
            if let Instruction::Meta(Meta::List(meta::List(instr_vec))) = instr {
                instr_stack.extend(instr_vec.into_iter().rev());
            } else {
                out_instrs.push(instr);
            }
        }

        match out_instrs.len() {
            0 => Instruction::Noop,
            1 => out_instrs
                .into_iter()
                .next()
                .expect("should never fail since we know the size is 1"),
            _ => meta::List(out_instrs).into(),
        }
    }
}

#[derive(Clone)]
pub struct External(pub Arc<dyn traits::External>);

impl Debug for External {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "External")?;

        if let Some(extra_debug) = self.0.extra_debug() {
            write!(f, "(")?;
            extra_debug(f)?;
            write!(f, ")")?;
        }

        Ok(())
    }
}

impl PartialEq for External {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DefaultLoader;
impl traits::Loader for DefaultLoader {
    fn load(&self, value: Value) -> Result<Value> {
        Err(Error::UnloadableValue(value))
    }
}

// the name would make no sense otherwise and conflict with into
#[allow(clippy::module_name_repetitions)]
pub trait IntoInstruction {
    fn into_instruction(self) -> Instruction;
}

impl<T> From<T> for Instruction
where
    T: IntoInstruction,
{
    fn from(value: T) -> Self {
        value.into_instruction()
    }
}

impl<T> From<Option<T>> for Instruction
where
    T: IntoInstruction,
{
    fn from(value: Option<T>) -> Self {
        if let Some(value) = value {
            value.into_instruction()
        } else {
            Instruction::Noop
        }
    }
}

// the macro is exported in library root.
#[allow(clippy::module_name_repetitions)]
#[macro_export]
macro_rules! instruction_list {
    ($($inst:expr),* $(,)?) => {
        {
            use $crate::instruction::IntoInstruction as _;
            $crate::instruction::meta::List(vec![$($inst.into()),*]).into_instruction()
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use pure::put;

    #[test]
    pub fn instruction_flatten() {
        let instruction_expect = (1..=10)
            .map(|num| put(num).into_instruction())
            .collect::<meta::List>()
            .into_instruction();

        let instruction_input = instruction_list![
            put(1),
            put(2),
            instruction_list![put(3), put(4), instruction_list![put(5), put(6),], put(7),],
            instruction_list![put(8), put(9), put(10),],
        ];

        assert_eq!(instruction_expect, instruction_input.flatten());
    }
}
