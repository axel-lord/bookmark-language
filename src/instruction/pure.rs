use super::{traits::Pure, IntoInstruction};
use crate::{
    program,
    value::{self, def_op_fn, Value},
    Error, Result,
};
use serde::{Deserialize, Serialize};
use std::{mem, sync::Arc, thread, time::Duration};
use tap::Pipe;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub struct Debug;
impl Pure for Debug {
    fn perform(self, return_value: Value) -> Result<Value> {
        println!("{return_value:#?}");
        Ok(return_value)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub struct Sleep;
impl Pure for Sleep {
    fn perform(self, return_value: Value) -> Result<Value> {
        let Value::Float(duration) = return_value else {
            return Err(Error::WrongInstructionInput(
                return_value,
                self.into(),
            ));
        };

        thread::sleep(Duration::from_secs_f64(duration));
        Ok(Value::None)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Program(pub Arc<program::Program>);
impl Pure for Program {
    fn perform(self, return_value: Value) -> Result<Value> {
        let Self(mut arc_prgr) = self;

        arc_prgr
            .pipe_ref_mut(Arc::make_mut)
            .pipe(mem::take)
            .run(return_value)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Cond {
    if_true: Value,
    if_false: Value,
}
impl Pure for Cond {
    fn perform(self, return_value: Value) -> Result<Value> {
        let Value::Bool(value) = return_value else {
            return Err(Error::WrongInstructionInput(
                return_value,
                self.into(),
            ))
        };

        let Self { if_true, if_false } = self;

        Ok(if value { if_true } else { if_false })
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Put(pub value::Value);
impl Pure for Put {
    fn perform(self, _: Value) -> Result<Value> {
        Ok(self.0)
    }
}

pub fn put(value: impl Into<Value>) -> Put {
    Put(value.into())
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Coerce(pub value::Type);
impl Pure for Coerce {
    fn perform(self, return_value: Value) -> Result<Value> {
        return_value.cast(self.0)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Parse(pub value::Type);
impl Pure for Parse {
    fn perform(self, return_value: Value) -> Result<Value> {
        return_value.parse(self.0)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Op(value::Operation, Value);
impl Pure for Op {
    fn perform(self, lhs: Value) -> Result<Value> {
        let Self(operation, rhs) = self;
        operation.apply(lhs, rhs)
    }
}

def_op_fn!(Op, value, Value);

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ToFallible;
impl Pure for ToFallible {
    fn perform(self, return_value: Value) -> Result<Value> {
        let Value::Instruction(boxed_instr) = return_value else {
             return Err(Error::WrongInstructionInput(return_value, self.into()))
        };

        let super::Instruction::Pure(super::Pure::Program(Program(mut arc_prgr))) = *boxed_instr else {
             return Err(Error::WrongInstructionInput(boxed_instr.into(), self.into()))
        };

        if !arc_prgr.is_fallible() {
            let mut_prgr = Arc::make_mut(&mut arc_prgr);
            *mut_prgr = mem::take(mut_prgr).into_fallible();
        }

        Ok(Program(arc_prgr).into_instruction().into())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ToInfallible;
impl Pure for ToInfallible {
    fn perform(self, return_value: Value) -> Result<Value> {
        let Value::Instruction(boxed_instr) = return_value else {
             return Err(Error::WrongInstructionInput(return_value, self.into()))
        };

        let super::Instruction::Pure(super::Pure::Program(Program(mut arc_prgr))) = *boxed_instr else {
             return Err(Error::WrongInstructionInput(boxed_instr.into(), self.into()))
        };

        if arc_prgr.is_fallible() {
            let mut_prgr = Arc::make_mut(&mut arc_prgr);
            *mut_prgr = mem::take(mut_prgr).into_infallible();
        }

        Ok(Program(arc_prgr).into_instruction().into())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub struct Not;
impl Pure for Not {
    fn perform(self, return_value: Value) -> Result<Value> {
        if let Value::Bool(value) = return_value {
            Ok(Value::Bool(!value))
        } else {
            Err(Error::WrongInstructionInput(return_value, self.into()))
        }
    }
}
