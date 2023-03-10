use serde::{Deserialize, Serialize};

mod meta;
mod mutating;
mod pure;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub enum Instruction {
    #[default]
    Noop,
    Pure(Pure),
    Mutating(Mutating),
    Meta(Meta),
}

pub trait IntoInstruction {
    fn into_instruction(self) -> Instruction;
}

pub use meta::Meta;
pub use mutating::Mutating;
pub use pure::Pure;

impl IntoInstruction for Pure {
    fn into_instruction(self) -> Instruction {
        Instruction::Pure(self)
    }
}

impl IntoInstruction for Meta {
    fn into_instruction(self) -> Instruction {
        Instruction::Meta(self)
    }
}

impl IntoInstruction for Mutating {
    fn into_instruction(self) -> Instruction {
        Instruction::Mutating(self)
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
        $crate::instruction::Instruction::Meta($crate::instruction::Meta::List(vec![$($inst.into()),*]))
    };
}
