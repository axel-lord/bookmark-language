use std::{borrow::Cow, fmt::Debug, sync::Arc};

use crate::{value::Value, variable, Result};
use derive_more::IsVariant;
use serde::{Deserialize, Serialize};

pub mod meta;
pub mod mutating;
pub mod pure;
pub mod reading;

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
    #[serde(skip)]
    External(External),
}

type ExternalReturn = Result<(Value, variable::Map, Stack)>;
type ExternalInner = Arc<dyn Fn(Value, variable::Map, Stack) -> ExternalReturn>;

#[derive(Clone)]
pub struct External(pub ExternalInner);

impl Debug for External {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "External")
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
    Program(pure::Program),
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

pub mod traits {
    use crate::{value::Value, variable, Result};
    use serde::{Deserialize, Serialize};
    use std::fmt::Debug;

    pub trait Pure: Debug + Serialize + Deserialize<'static> + Clone + PartialEq {
        fn perform(self, return_value: Value) -> Result<Value>;
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
