use derive_more::IsVariant;
use instruction::Instruction;
use std::result;
use thiserror::Error;
use value::{Operation, Value};

pub mod instruction;
pub mod program;
pub mod variable;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Error, IsVariant)]
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

    #[error("over/underflow occured on integer operation {op:?}, lhs = {lhs:?}, rhs = {rhs:?}")]
    IntegerOverOrUnderFlow {
        op: Operation,
        lhs: Value,
        rhs: Value,
    },

    #[error("failed to cast {0:?} to {1:?}")]
    FailedCast(Value, value::Type),

    #[error("failed to parse {1:?} into {0:?}")]
    FailedParse(value::Type, Value),

    #[error("cast {0:?} to {0:?} is invalid, value tried {2:?}")]
    InvalidCast(value::Type, value::Type, Value),

    #[error("parse to {0:?} is invalid")]
    InvalidParse(value::Type),

    #[error("can only parse strings, value tried")]
    NonStringParse(Value),

    #[error("{key:?} is not a key of {map:?}")]
    InvalidAcces { key: Value, map: Value },

    #[error("{0:?} is the wrong type of key used for access to type {1:?}")]
    WrongKeyType(Value, value::Type),

    #[error("{0:?} cannot be used for {1:?}")]
    WrongInstructionInput(Value, Instruction),

    #[error("{0:?} cannot be loaded using current loader")]
    UnloadableValue(Value),
}

pub mod value;
