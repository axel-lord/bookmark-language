use crate::{
    instruction::{self, traits::Loader, External, Instruction, IntoInstruction},
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

#[derive(Debug, Clone, PartialEq)]
pub enum Running {
    Active(variable::Map, instruction::Stack, Value),
    Finished(Result<Value>),
}

impl Default for Running {
    fn default() -> Self {
        Self::Finished(Ok(Value::None))
    }
}

impl Running {
    fn handle_instruction(
        instr: Instruction,
        value: Value,
        variables: &mut variable::Map,
        stack: &mut instruction::Stack,
        loader: &dyn Loader,
    ) -> Result<Value> {
        let mut return_value = Value::None;
        match instr {
            Instruction::Noop => (),
            Instruction::Pure(instr) => return_value = instr.perform(value)?,
            Instruction::Reading(instr) => return_value = instr.perform(value, variables)?,
            Instruction::Mutating(instr) => {
                (return_value, *variables) = instr.perform(value, mem::take(variables))?;
            }
            Instruction::Meta(instr) => {
                (return_value, *variables, *stack) =
                    instr.perform(value, mem::take(variables), mem::take(stack))?;
            }
            Instruction::Loading(instr) => return_value = instr.perform(value, loader)?,
            Instruction::External(External(instr)) => {
                (return_value, *variables, *stack) =
                    instr.perform(value, mem::take(variables), mem::take(stack))?;
            }
        }
        Ok(return_value)
    }

    #[must_use]
    pub fn progress(self, loader: &dyn Loader) -> Self {
        if let Self::Active(mut variables, mut stack, value) = self {
            let Some(instr) = stack.pop() else {
                return Self::Finished(Ok(value));
            };

            match Self::handle_instruction(instr, value, &mut variables, &mut stack, loader) {
                Ok(value) => Self::Active(variables, stack, value),
                Err(err) => Self::Finished(Err(err)),
            }
        } else {
            self
        }
    }

    pub fn progress_in_place(&mut self, loader: &dyn Loader) {
        *self = mem::take(self).progress(loader);
    }
}

impl Program {
    fn try_run_to_completion(self, input: Value, loader: &dyn Loader) -> Result<Value> {
        let mut running = self.run(input);

        loop {
            match running.progress(loader) {
                Running::Finished(value) => break value,
                active @ Running::Active(..) => running = active,
            }
        }
    }

    #[must_use]
    pub fn run(self, input: Value) -> Running {
        Running::Active(self.variables, vec![self.instruction].into(), input)
    }

    pub fn run_to_completion(self, input: Value, loader: &dyn Loader) -> Result<Value> {
        if self.is_fallible {
            self.try_run_to_completion(input, loader)
                .or(Ok(Value::None))
        } else {
            self.try_run_to_completion(input, loader)
        }
    }

    #[must_use]
    pub fn into_fallible(self) -> Self {
        Self {
            is_fallible: true,
            ..self
        }
    }

    #[must_use]
    pub fn into_infallible(self) -> Self {
        Self {
            is_fallible: false,
            ..self
        }
    }

    #[must_use]
    pub fn is_fallible(&self) -> bool {
        self.is_fallible
    }
}

#[derive(Debug, Default, Clone)]
pub struct Builder {
    instruction_vec: Vec<Instruction>,
    is_fallible: bool,
}

impl Builder {
    #[must_use]
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

    #[must_use]
    pub fn build(self, variable_map: variable::Map) -> Program {
        let Builder {
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
