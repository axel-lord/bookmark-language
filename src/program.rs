use crate::{instruction::Instruction, variable, Error, Result, Value};
use serde::{Deserialize, Serialize};
use std::mem;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Program {
    pub variables: variable::Map,
    pub instruction: Instruction,
}

impl Program {
    pub fn run(self, input: Value) -> Result<Value> {
        // Destructure self
        let Self {
            variables,
            instruction,
        } = self;

        // Program state
        let mut return_value = Value::None;
        let mut instruction_stack = vec![instruction];
        let mut variable_map = variables;

        // Map input to input variable
        *variable_map.read_mut(variable::Id::input())? = input;

        // Pop and execute instructions
        while let Some(instruction) = instruction_stack.pop() {
            match instruction {
                Instruction::Pure(instruction) => {
                    return_value = instruction.perform(&variable_map)?
                }
                Instruction::Mutating(instruction) => {
                    return_value = instruction.perform(&mut variable_map)?
                }
                Instruction::List(instruction_list) => {
                    instruction_stack.extend(instruction_list.into_vec().into_iter().rev());
                    return_value = Value::None;
                }
                Instruction::Assign(id) => {
                    *variable_map.read_mut(id)? = mem::take(&mut return_value)
                }
                Instruction::Perform => match mem::take(&mut return_value) {
                    Value::Instruction(instruction) => instruction_stack.push(instruction),
                    value => return Err(Error::PerformOnNonInstruction(value)),
                },
            }
        }

        Ok(return_value)
    }
}
