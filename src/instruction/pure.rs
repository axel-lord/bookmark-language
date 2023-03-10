use super::Instruction;
use crate::{
    program::Program,
    value::{self, Value},
    variable, Error, Result,
};
use serde::{Deserialize, Serialize};
use std::{mem, sync::Arc, thread, time::Duration};
use tap::Pipe;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Pure {
    Debug,
    Sleep,
    Program(Arc<Program>),
    ToFallible,
    ToInfallible,
    Clone(variable::Id),
    Add(Value),
    Sub(Value),
    Mul(Value),
    Div(Value),
    Value(Value),
    Cond { if_true: Value, if_false: Value },
    Coerce(value::Type),
    Parse(value::Type),
    GetClone(variable::Id),
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
            Pure::ToFallible | Pure::ToInfallible => {
                // ensure instruction
                let Value::Instruction(box_instr) = return_value else {
                    return Err(Error::WrongInstructionInput(return_value, self.into()))
                };

                let Instruction::Pure(Pure::Program(mut arc_prgr)) = *box_instr else {
                    return Err(Error::WrongInstructionInput(Value::Instruction(box_instr), self.into()))
                };

                Arc::make_mut(&mut arc_prgr)
                    .pipe(mem::take)
                    .pipe(|prgr| {
                        if matches!(self, Pure::ToFallible) {
                            prgr.into_fallible()
                        } else {
                            prgr.into_infallible()
                        }
                    })
                    .pipe(Arc::new)
                    .pipe(Pure::Program)
                    .pipe(Instruction::Pure)
                    .pipe(Box::new)
                    .pipe(Value::Instruction)
                    .pipe(Ok)
            }
            Pure::GetClone(id) => variables
                .read(id)?
                .get(variables.maybe_read(return_value)?)
                .cloned(),
            Pure::Cond { if_true, if_false } => {
                if let Value::Bool(value) = return_value {
                    Ok(if value { if_true } else { if_false })
                } else {
                    Err(Error::WrongInstructionInput(
                        return_value,
                        Pure::Cond { if_true, if_false }.into(),
                    ))
                }
            }
            Pure::Coerce(ty) => return_value.cast(ty),
            Pure::Parse(ty) => return_value.parse(ty),
        }
    }

    pub fn value(value: impl Into<Value>) -> Self {
        Self::Value(value.into())
    }
}
