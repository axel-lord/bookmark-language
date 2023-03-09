use crate::{program::Program, value::Value, variable, Error, Result};
use serde::{Deserialize, Serialize};
use std::{mem, sync::Arc};
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
    Program(Box<Program>, variable::Id),
    Clone(variable::Id),
    Add(Value),
    Sub(Value),
    Mul(Value),
    Div(Value),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Mutating {
    Take(variable::Id),
    Assign(variable::Id),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Meta {
    Return,
    Perform,
    List(Box<[Instruction]>),
}

impl Pure {
    pub fn perform(self, return_value: Value, variables: &variable::Map) -> Result<Value> {
        match self {
            Pure::Program(program, id) => program.run(variables.read(id)?.clone()),
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
        }
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
        variables: variable::Map,
        mut instruction_stack: Vec<Instruction>,
    ) -> Result<(Value, variable::Map, Vec<Instruction>)> {
        match self {
            Meta::Return => {
                instruction_stack.clear();
                Ok((return_value, variables, instruction_stack))
            }
            Meta::Perform => match return_value {
                Value::Instruction(mut instruction) => {
                    instruction_stack.push(instruction.pipe_ref_mut(Arc::make_mut).pipe(mem::take));
                    Ok((Value::None, variables, instruction_stack))
                }
                value => Err(Error::PerformOnNonInstruction(value)),
            },
            Meta::List(list) => {
                instruction_stack.extend(list.into_vec().into_iter().rev());
                Ok((Value::None, variables, instruction_stack))
            }
        }
    }
}
