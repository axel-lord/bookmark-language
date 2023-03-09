use crate::{program::Program, variable, Result, Value};
use serde::{Deserialize, Serialize};
use std::mem;
use tap::Pipe;

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
    Program(Box<modname::Program>, variable::Id),
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
