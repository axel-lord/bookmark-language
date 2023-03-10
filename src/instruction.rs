use crate::{program::Program, value::Value, variable, Error, Result};
use serde::{Deserialize, Serialize};
use std::{mem, sync::Arc, thread, time::Duration};
use tap::Pipe;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub enum Instruction {
    #[default]
    Noop,
    Pure(Pure),
    Mutating(Mutating),
    Meta(Meta),
}

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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Mutating {
    Take(variable::Id),
    Assign(variable::Id),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Meta {
    Return,
    Perform(Value),
    PerformClone(variable::Id),
    PerformTake(variable::Id),
    List(Vec<Instruction>),
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

impl Mutating {
    pub fn perform(
        self,
        return_value: Value,
        mut variables: variable::Map,
    ) -> Result<(Value, variable::Map)> {
        match self {
            Mutating::Take(id) => Ok((mem::take(variables.read_mut(id)?), variables)),
            Mutating::Assign(id) => {
                *variables.read_mut(id)? = return_value;
                Ok((Value::None, variables))
            }
        }
    }
}

impl Meta {
    pub fn perform(
        self,
        return_value: Value,
        mut variables: variable::Map,
        mut instruction_stack: Vec<Instruction>,
    ) -> Result<(Value, variable::Map, Vec<Instruction>)> {
        match self {
            Meta::Return => {
                instruction_stack.clear();
                Ok((return_value, variables, instruction_stack))
            }
            Meta::Perform(value) => match return_value {
                Value::Instruction(mut instruction) => {
                    instruction_stack.push(Pure::Value(value).into());
                    instruction_stack.push(instruction.pipe_ref_mut(Arc::make_mut).pipe(mem::take));
                    Ok((Value::None, variables, instruction_stack))
                }
                value => Err(Error::PerformOnNonInstruction(value)),
            },
            Meta::PerformClone(value) => match return_value {
                Value::Instruction(mut instruction) => {
                    instruction_stack.push(variables.read(value)?.clone().pipe(Pure::Value).into());
                    instruction_stack.push(instruction.pipe_ref_mut(Arc::make_mut).pipe(mem::take));
                    Ok((Value::None, variables, instruction_stack))
                }
                value => Err(Error::PerformOnNonInstruction(value)),
            },
            Meta::PerformTake(value) => match return_value {
                Value::Instruction(mut instruction) => {
                    instruction_stack.push(
                        variables
                            .read_mut(value)?
                            .pipe(mem::take)
                            .pipe(Pure::Value)
                            .into(),
                    );
                    instruction_stack.push(instruction.pipe_ref_mut(Arc::make_mut).pipe(mem::take));
                    Ok((Value::None, variables, instruction_stack))
                }
                value => Err(Error::PerformOnNonInstruction(value)),
            },
            Meta::List(list) => {
                instruction_stack.extend(list.into_iter().rev());
                Ok((Value::None, variables, instruction_stack))
            }
        }
    }
}

impl From<Pure> for Instruction {
    fn from(value: Pure) -> Self {
        Instruction::Pure(value)
    }
}

impl From<Meta> for Instruction {
    fn from(value: Meta) -> Self {
        Instruction::Meta(value)
    }
}

impl From<Mutating> for Instruction {
    fn from(value: Mutating) -> Self {
        Instruction::Mutating(value)
    }
}

#[macro_export]
macro_rules! instruction_list {
    ($($inst:expr),* $(,)?) => {
        $crate::instruction::Instruction::Meta($crate::instruction::Meta::List(vec![$($inst.into()),*]))
    };
}
