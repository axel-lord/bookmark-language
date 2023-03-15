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

set_macro::instr! {
Pure: [
    Sleep(pure::Sleep),
    Debug(pure::Debug),
    Cond(pure::Cond),
    Put(pure::Put),
    Coerce(pure::Coerce),
    Parse(pure::Parse),
    Op(pure::Op),
    ToFallible(pure::ToFallible),
    ToInfallible(pure::ToInfallible),
    Not(pure::Not),
],
Reading: [
    Clone(reading::Clone),
    GetClone(reading::GetClone),
    OpClone(reading::OpClone),
],
Mutating: [
    Take(mutating::Take),
    Assign(mutating::Assign),
    Swap(mutating::Swap),
    GetTake(mutating::GetTake),
    MapAssign(mutating::MapAssign),
    OpTake(mutating::OpTake),
],
Meta: [
    List(meta::List),
    Return(meta::Return),
    Perform(meta::Perform),
    PerformClone(meta::PerformClone),
    PerformTake(meta::PerformTake),
],
Loading: [
    Program(loading::Program),
    Load(loading::Load),
]
}

impl Instruction {
    pub fn flatten(self) -> Self {
        let Instruction::Meta(Meta::List(meta::List(instr_vec))) = self else {
            return self;
        };

        let mut out_instrs = Vec::new();
        let mut instr_stack = Vec::from_iter(instr_vec.into_iter().rev());

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

#[derive(Clone, Copy, Debug)]
pub struct DefaultLoader;
impl traits::Loader for DefaultLoader {
    fn load(&self, value: Value) -> Result<Value> {
        Err(Error::UnloadableValue(value))
    }
}

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
