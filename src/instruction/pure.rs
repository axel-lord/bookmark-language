use super::Instruction;
use crate::{program::Program, value::Value, variable, Error, Result};
use serde::{Deserialize, Serialize};
use std::{mem, sync::Arc, thread, time::Duration};
use tap::Pipe;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Pure {
    Debug,
    Sleep,
    Program(Arc<Program>),
    Clone(variable::Id),
    Add(Value),
    Sub(Value),
    Mul(Value),
    Div(Value),
    Value(Value),
}

impl Pure {
    pub fn perform(self, return_value: Value, variables: &variable::Map) -> Result<Value> {
        match self {
            Pure::Program(mut program) => program
                .pipe_ref_mut(Arc::make_mut)
                .pipe(mem::take)
                .run(return_value),
            Pure::Clone(id) => variables.read(id).cloned(),
            Pure::Add(value) => {
                variables.maybe_read(return_value)? + variables.maybe_read(value)?
            }
            Pure::Sub(value) => {
                variables.maybe_read(return_value)? - variables.maybe_read(value)?
            }
            Pure::Mul(value) => {
                variables.maybe_read(return_value)? * variables.maybe_read(value)?
            }
            Pure::Div(value) => {
                variables.maybe_read(return_value)? / variables.maybe_read(value)?
            }
            Pure::Debug => println!("{return_value:#?}").pipe(|_| Ok(return_value)),
            Pure::Value(value) => Ok(value),
            Pure::Sleep => {
                if let Value::Float(duration) = return_value {
                    thread::sleep(Duration::from_secs_f64(duration));
                    Ok(Value::None)
                } else {
                    Err(Error::WrongInstructionInput(
                        return_value,
                        Instruction::Pure(Pure::Sleep),
                    ))
                }
            }
        }
    }

    pub fn value(value: impl Into<Value>) -> Self {
        Self::Value(value.into())
    }
}
