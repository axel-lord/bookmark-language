use crate::{
    instruction::{self, External, Instruction, IntoInstruction},
    value::Value,
    variable, Result,
};
use serde::{Deserialize, Serialize};
use std::mem;

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub struct Program {
    variables: variable::Map,
    instruction: Instruction,
    is_fallible: bool,
}

impl Program {
    fn try_run(input: Value, variables: variable::Map, instruction: Instruction) -> Result<Value> {
        // Program state
        let mut return_value = input; // Input is stored as first return value
        let mut instruction_stack = instruction::Stack::from(vec![instruction]);
        let mut variable_map = variables;

        // Pop and execute instructions
        while let Some(instruction) = instruction_stack.pop() {
            match instruction {
                Instruction::Reading(instruction) => {
                    return_value =
                        instruction.perform(mem::take(&mut return_value), &variable_map)?
                }
                Instruction::Pure(instruction) => {
                    return_value = instruction.perform(mem::take(&mut return_value))?
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
                Instruction::External(External(instr)) => {
                    (return_value, variable_map, instruction_stack) = instr(
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

    pub fn run(self, input: Value) -> Result<Value> {
        let Self {
            variables,
            instruction,
            is_fallible: fallible,
        } = self;

        if fallible {
            Self::try_run(input, variables, instruction).or(Ok(Value::None))
        } else {
            Self::try_run(input, variables, instruction)
        }
    }

    pub fn into_fallible(self) -> Self {
        Self {
            is_fallible: true,
            ..self
        }
    }

    pub fn into_infallible(self) -> Self {
        Self {
            is_fallible: false,
            ..self
        }
    }

    pub fn is_fallible(&self) -> bool {
        self.is_fallible
    }
}

#[derive(Debug, Default, Clone)]
pub struct ProgramBuilder {
    instruction_vec: Vec<Instruction>,
    is_fallible: bool,
}

impl ProgramBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_instruction(&mut self, instruction: Instruction) -> &mut Self {
        self.instruction_vec.push(instruction);
        self
    }

    pub fn is_fallible(&mut self, is_fallible: bool) -> &mut Self {
        self.is_fallible = is_fallible;
        self
    }

    pub fn build(self, variable_map: variable::Map) -> Program {
        let ProgramBuilder {
            mut instruction_vec,
            is_fallible,
        } = self;

        Program {
            is_fallible,
            variables: variable_map,
            instruction: match instruction_vec.len() {
                0 => Instruction::Noop,
                1 => instruction_vec
                    .pop()
                    .expect("since we now the length is 1 pop should always succeed"),
                _ => instruction::meta::List(instruction_vec).into_instruction(),
            },
        }
    }
}
