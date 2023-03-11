use super::{traits::Meta, Instruction, Pure};
use crate::{value::Value, variable, Error, Result};
use serde::{Deserialize, Serialize};
use std::mem;
use tap::Pipe;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct List(pub Vec<Instruction>);
impl Meta for List {
    fn perform(
        self,
        _return_value: Value,
        variables: variable::Map,
        mut instruction_stack: super::Stack,
    ) -> Result<(Value, variable::Map, super::Stack)> {
        let Self(list) = self;

        instruction_stack.extend(list.into_iter().rev());
        Ok((Value::None, variables, instruction_stack))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Return;
impl Meta for Return {
    fn perform(
        self,
        return_value: Value,
        variables: variable::Map,
        mut instruction_stack: super::Stack,
    ) -> Result<(Value, variable::Map, super::Stack)> {
        instruction_stack.clear();
        Ok((return_value, variables, instruction_stack))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Perform(pub Value);
impl Meta for Perform {
    fn perform(
        self,
        return_value: Value,
        variables: variable::Map,
        mut instruction_stack: super::Stack,
    ) -> Result<(Value, variable::Map, super::Stack)> {
        let Self(value) = self;

        match return_value {
            Value::Instruction(instruction) => {
                instruction_stack.push(Pure::Value(value));
                instruction_stack.push(*instruction);
                Ok((Value::None, variables, instruction_stack))
            }
            value => Err(Error::PerformOnNonInstruction(value)),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct PerformClone(pub variable::Id);
impl Meta for PerformClone {
    fn perform(
        self,
        return_value: Value,
        variables: variable::Map,
        mut instruction_stack: super::Stack,
    ) -> Result<(Value, variable::Map, super::Stack)> {
        let Self(value) = self;

        match return_value {
            Value::Instruction(instruction) => {
                instruction_stack.push(variables.read(value)?.clone().pipe(Pure::Value));
                instruction_stack.push(*instruction);
                Ok((Value::None, variables, instruction_stack))
            }
            value => Err(Error::PerformOnNonInstruction(value)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct PerformTake(pub variable::Id);
impl Meta for PerformTake {
    fn perform(
        self,
        return_value: Value,
        mut variables: variable::Map,
        mut instruction_stack: super::Stack,
    ) -> Result<(Value, variable::Map, super::Stack)> {
        let Self(value) = self;

        match return_value {
            Value::Instruction(instruction) => {
                instruction_stack
                    .push(variables.read_mut(value)?.pipe(mem::take).pipe(Pure::Value));
                instruction_stack.push(*instruction);
                Ok((Value::None, variables, instruction_stack))
            }
            value => Err(Error::PerformOnNonInstruction(value)),
        }
    }
}
