use super::{Instruction, Pure};
use crate::{value::Value, variable, Error, Result};
use serde::{Deserialize, Serialize};
use std::{mem, sync::Arc};
use tap::Pipe;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Meta {
    Return,
    Perform(Value),
    PerformClone(variable::Id),
    PerformTake(variable::Id),
    List(Vec<Instruction>),
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
