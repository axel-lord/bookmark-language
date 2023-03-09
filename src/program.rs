use crate::{instruction::Instruction, value::Value, variable, Result};
use serde::{Deserialize, Serialize};
use std::mem;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
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
        let mut return_value = input; // Input is stored as first return value
        let mut instruction_stack = vec![instruction];
        let mut variable_map = variables;

        // Pop and execute instructions
        while let Some(instruction) = instruction_stack.pop() {
            match instruction {
                Instruction::Pure(instruction) => {
                    return_value =
                        instruction.perform(mem::take(&mut return_value), &variable_map)?
                }
                Instruction::Mutating(instruction) => {
                    (return_value, variable_map) = instruction
                        .perform(mem::take(&mut return_value), mem::take(&mut variable_map))?
                }
                Instruction::Meta(instruction) => {
                    (return_value, variable_map, instruction_stack) = instruction.perform(
                        mem::take(&mut return_value),
                        mem::take(&mut variable_map),
                        mem::take(&mut instruction_stack),
                    )?;
                }
                Instruction::Noop => (),
            }
        }

        Ok(return_value)
    }
}
