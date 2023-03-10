use crate::{
    instruction::{self, Instruction},
    value::Value,
    variable, Result,
};
use serde::{Deserialize, Serialize};
use std::mem;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Program {
    variables: variable::Map,
    instruction: Instruction,
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

#[derive(Debug, Default, Clone)]
pub struct ProgramBuilder {
    instruction_vec: Vec<Instruction>,
}

impl ProgramBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_instruction(&mut self, instruction: Instruction) -> &mut Self {
        self.instruction_vec.push(instruction);
        self
    }

    pub fn build(self, variable_map: variable::Map) -> Program {
        let ProgramBuilder {
            mut instruction_vec,
        } = self;

        Program {
            variables: variable_map,
            instruction: match instruction_vec.len() {
                0 => Instruction::Noop,
                1 => instruction_vec
                    .pop()
                    .expect("since we now the length is 1 pop should always succeed"),
                _ => Instruction::Meta(instruction::Meta::List(instruction_vec)),
            },
        }
    }
}
