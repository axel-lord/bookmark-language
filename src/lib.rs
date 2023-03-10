use instruction::Instruction;
use std::result;
use thiserror::Error;
use value::{Operation, Value};

pub mod instruction;
pub mod program;
pub mod variable;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0:?} is not the id of a variable in use")]
    UnknownVariable(variable::Id),
    #[error("attempt to get a mutable reference to read only variable {0:?}")]
    WriteToReadOnly(variable::Id),
    #[error("the Perform instruction was used when last return value was not an instruction")]
    PerformOnNonInstruction(Value),
    #[error("operation {0:?} is not supported for {1:?} and {2:?}")]
    UnsuppurtedOperation(Operation, Value, Value),
    #[error("tried to divide {0:?} by {1:?} (Zero)")]
    ZeroDiv(Value, Value),
    #[error("{0:?} cannot be used for {1:?}")]
    WrongInstructionInput(Value, Instruction),
}

pub mod value;
