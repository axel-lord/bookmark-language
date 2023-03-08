use std::{collections::BTreeMap, mem, result};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::instruction::Instruction;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Program {
    variables: variable::Map,
    instruction: Instruction,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0:?} is not the id of a variable in use")]
    UnknownVariable(variable::Id),
    #[error("the Perform instruction was used when last return value was not an instruction")]
    PerformOnNonInstruction(Value),
}

type Result<T> = result::Result<T, Error>;

pub mod instruction {
    use tap::Pipe;

    use super::*;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub enum Instruction {
        Perform,
        Pure(Pure),
        Mutating(Mutating),
        Assign(variable::Id),
        List(Box<[Instruction]>),
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub enum Pure {
        Program(Box<Program>, variable::Id),
        Clone(variable::Id),
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub enum Mutating {
        Take(variable::Id),
    }

    impl Pure {
        pub fn perform(self, variables: &variable::Map) -> Result<Value> {
            match self {
                Pure::Program(program, id) => program.run(variables.read(id)?.clone()),
                Pure::Clone(id) => variables.read(id).cloned(),
            }
        }
    }

    impl Mutating {
        pub fn perform(self, variables: &mut variable::Map) -> Result<Value> {
            match self {
                Mutating::Take(id) => mem::take(variables.read_mut(id)?).pipe(Ok),
            }
        }
    }
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

pub mod variable {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, Clone, Copy)]
    pub struct Id(usize);

    impl Id {
        pub fn input() -> Self {
            Self(0)
        }
    }

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Map(Box<[Value]>);

    impl Map {
        pub fn read(&self, id: Id) -> Result<&Value> {
            self.0.get(id.0).ok_or(Error::UnknownVariable(id))
        }

        pub fn read_mut(&mut self, id: Id) -> Result<&mut Value> {
            self.0.get_mut(id.0).ok_or(Error::UnknownVariable(id))
        }
    }

    #[derive(Debug, Clone)]
    pub struct MapBuilder(Vec<Value>);

    impl Default for MapBuilder {
        fn default() -> Self {
            Self(vec![Value::None])
        }
    }

    impl MapBuilder {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn push(&mut self, init: Value) -> Id {
            let id = Id(self.0.len());
            self.0.push(init);
            id
        }

        pub fn build(self) -> Map {
            Map(self.0.into_boxed_slice())
        }
    }
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
