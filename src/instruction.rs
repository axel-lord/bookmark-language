use crate::{value::Value, variable, Result};
use derive_more::IsVariant;
use serde::{Deserialize, Serialize};

pub mod meta;
pub mod mutating;
mod pure;

#[derive(Debug, Deserialize, Serialize, Clone, Default, IsVariant)]
pub enum Instruction {
    #[default]
    Noop,
    Pure(Pure),
    Mutating(Mutating),
    Meta(Meta),
}

#[derive(Debug, Default)]
pub struct Stack(Vec<Instruction>);

impl Stack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) -> &mut Self {
        self.0.clear();
        self
    }

    pub fn push(&mut self, instr: impl Into<Instruction>) -> &mut Self {
        self.0.push(instr.into());
        self
    }

    pub fn extend(&mut self, instrs: impl IntoIterator<Item = Instruction>) -> &mut Self {
        self.0.extend(instrs.into_iter());
        self
    }

    pub fn pop(&mut self) -> Option<Instruction> {
        self.0.pop()
    }
}

impl From<Vec<Instruction>> for Stack {
    fn from(value: Vec<Instruction>) -> Self {
        Self(value)
    }
}

pub mod traits {
    use crate::{value::Value, variable, Result};
    use serde::{Deserialize, Serialize};
    use std::fmt::Debug;

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

macro_rules! instruction_set {
    (
        Pure: {$($pu_name:ident($pu_ty:ty)),* $(,)?},
        Mutating: {$($mu_name:ident($mu_ty:ty)),* $(,)?},
        Meta: {$($me_name:ident($me_ty:ty)),* $(,)?}
        ) => {

        #[derive(Debug, Deserialize, Serialize, Clone)]
        pub enum Mutating {
            $(
            $mu_name($mu_ty),
            )*
        }

        impl  Mutating {
            pub fn perform(
                self,
                return_value: Value,
                variables: variable::Map
            ) -> Result<(Value, variable::Map)> {
                use traits::Mutating as _;
                match self {
                    $(
                    Self::$mu_name(instr) => instr.perform(return_value, variables),
                    )*
                }
            }
        }

        $(
        impl IntoInstruction for $mu_ty {
            fn into_instruction(self) -> Instruction {
                Instruction::Mutating(Mutating::$mu_name(self))
            }
        }
        )*

        #[derive(Debug, Deserialize, Serialize, Clone)]
        pub enum Meta {
            $(
            $me_name($me_ty),
            )*
        }

        impl Meta {
            pub fn perform(
                self,
                return_value: Value,
                variables: variable::Map,
                instruction_stack: Stack,
            ) -> Result<(Value, variable::Map, Stack)> {
                use traits::Meta as _;
                match self {
                    $(
                    Self::$me_name(instr) => instr.perform(return_value, variables, instruction_stack),
                    )*
                }
            }
        }

        $(
        impl IntoInstruction for $me_ty {
            fn into_instruction(self) -> Instruction {
                Instruction::Meta(Meta::$me_name(self))
            }
        }
        )*
    };
}

instruction_set! {
Pure: {},
Mutating: {
    Take(mutating::Take),
    Assign(mutating::Assign),
    Swap(mutating::Swap),
    GetTake(mutating::GetTake),
    MapAssign(mutating::MapAssign),
},
Meta: {
    List(meta::List),
    Return(meta::Return),
    Perform(meta::Perform),
    PerformClone(meta::PerformClone),
    PerformTake(meta::PerformTake),
}
}

pub trait IntoInstruction {
    fn into_instruction(self) -> Instruction;
}

pub use pure::Pure;

impl IntoInstruction for Pure {
    fn into_instruction(self) -> Instruction {
        Instruction::Pure(self)
    }
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
