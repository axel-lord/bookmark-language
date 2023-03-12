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

#[derive(Debug, Deserialize, Serialize, Clone, Default, IsVariant)]
pub enum Instruction {
    #[default]
    Noop,
    Pure(Pure),
    Reading(Reading),
    Mutating(Mutating),
    Meta(Meta),
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

pub mod traits {
    use crate::{value::Value, variable, Result};
    use serde::{Deserialize, Serialize};
    use std::fmt::Debug;

    pub trait Pure: Debug + Serialize + Deserialize<'static> + Clone {
        fn perform(self, return_value: Value) -> Result<Value>;
    }

    pub trait Reading: Debug + Serialize + Deserialize<'static> + Clone {
        fn perform(self, return_value: Value, variables: &variable::Map) -> Result<Value>;
    }

    pub trait Mutating: Debug + Serialize + Deserialize<'static> + Clone {
        fn perform(
            self,
            return_value: Value,
            variables: variable::Map,
        ) -> Result<(Value, variable::Map)>;
    }

    pub trait Meta: Debug + Serialize + Deserialize<'static> + Clone {
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
