use std::{collections::BTreeMap, result};

use crate::instruction::Instruction;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod instruction;
pub mod program;
pub mod variable;

type Result<T> = result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0:?} is not the id of a variable in use")]
    UnknownVariable(variable::Id),
    #[error("the Perform instruction was used when last return value was not an instruction")]
    PerformOnNonInstruction(Value),
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Id(variable::Id),
    Instruction(Instruction),
    List(Vec<Value>),
    Map(BTreeMap<String, Value>),
    #[default]
    None,
}
