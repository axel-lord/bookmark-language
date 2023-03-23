//! Library for bookmark-lang vm.

#![warn(
    missing_copy_implementations,
    //missing_docs,
    clippy::unwrap_used,
    clippy::pedantic,
    //rustdoc::all
)]
#![allow(clippy::missing_errors_doc)]

use derive_more::IsVariant;
use instruction::Instruction;
use std::result;
use thiserror::Error;
use value::{Operation, Value};

pub mod instruction;
pub mod program;
pub mod variable;

/// Result alias used in library.
pub type Result<T> = result::Result<T, Error>;

/// Error type in use by library.
#[derive(Debug, Error, IsVariant, PartialEq, Clone)]
pub enum Error {
    /// Used when an attempt is made to get access to a variable using an invalid id.
    #[error("{0:?} is not the id of a variable in use")]
    UnknownVariable(variable::Id),

    /// Used when an an attempt is made to get read-write access to a read-only variable.
    #[error("attempt to get a mutable reference to read only variable {0:?}")]
    WriteToReadOnly(variable::Id),

    /// Used when the [Perform][instruction::meta::Perform] instruction was used with an input that
    /// is not performable.
    #[error("the Perform instruction was used when last return value was not an instruction")]
    PerformOnNonInstruction(Value),

    /// Used when an [operation][value::Operation] is applied to two incompatible values.
    #[error("operation {0:?} is not supported for {1:?} and {2:?}")]
    UnsuppurtedOperation(Operation, Value, Value),

    /// Used when zero division is tried.
    #[error("tried to divide {0:?} by {1:?} (Zero)")]
    ZeroDiv(Value, Value),

    /// Used when an arithmetic operation under or overflow.
    #[error("over/underflow occured on integer operation {op:?}, lhs = {lhs:?}, rhs = {rhs:?}")]
    IntegerOverOrUnderFlow {
        /// The operation that was tried.
        op: Operation,
        /// The left hand side value used in the operation.
        lhs: Value,
        /// The right hand side value used in the operation.
        rhs: Value,
    },

    /// Used when the [cast][instruction::pure::cast] instruction fails.
    #[error("failed to cast {0:?} to {1:?}")]
    FailedCast(Value, value::Type),

    /// Used when the [parse][instruction::pure::Parse] instruction fails.
    #[error("failed to parse {1:?} into {0:?}")]
    FailedParse(value::Type, Value),

    /// Used when the [cast][instruction::pure::cast] intructon is used to perform a cast that is
    /// not allowed.
    #[error("cast {0:?} to {0:?} is invalid, value tried {2:?}")]
    InvalidCast(value::Type, value::Type, Value),

    /// Used when the [parse][instruction::pure::Parse] instrution is used to try an parse to an
    /// uparseable type.
    #[error("parse to {0:?} is invalid")]
    InvalidParse(value::Type),

    /// Used when the [parse][instruction::pure::Parse] instruction is given an input that is not a
    /// string.
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
