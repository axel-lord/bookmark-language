use crate::{
    instruction::{self, Instruction},
    variable, Error, Result,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    mem,
    ops::{Add, Div, Mul, Sub},
    sync::Arc,
};
use tap::{Pipe, Tap};

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(Arc<str>),
    Id(variable::Id),
    Instruction(Arc<Instruction>),
    List(Vec<Value>),
    Map(BTreeMap<Arc<str>, Value>),
    #[default]
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
}

impl Add<Value> for Value {
    type Output = Result<Value>;

    fn add(self, rhs: Value) -> Self::Output {
        match [self, rhs] {
            [Value::Int(lhs), Value::Int(rhs)] => lhs.saturating_add(rhs).pipe(Value::Int),
            [Value::Float(lhs), Value::Float(rhs)] => Value::Float(lhs + rhs),
            [Value::String(lhs), Value::String(rhs)] => {
                Value::String(lhs.to_string().add(&rhs).into_boxed_str().into())
            }
            [Value::Instruction(mut lhs), Value::Instruction(mut rhs)] => vec![
                lhs.pipe_ref_mut(Arc::make_mut).pipe(mem::take),
                rhs.pipe_ref_mut(Arc::make_mut).pipe(mem::take),
            ]
            .into_boxed_slice()
            .pipe(instruction::Meta::List)
            .pipe(Instruction::Meta)
            .pipe(Arc::new)
            .pipe(Value::Instruction),
            [Value::List(lhs), Value::List(rhs)] => lhs
                .tap_mut(|lhs| lhs.extend(rhs.into_iter()))
                .pipe(Value::List),
            [Value::Map(lhs), Value::Map(rhs)] => lhs
                .tap_mut(|lhs| lhs.extend(rhs.into_iter()))
                .pipe(Value::Map),
            [lhs, rhs] => return Error::UnsuppurtedOperation(Operation::Add, lhs, rhs).pipe(Err),
        }
        .pipe(Ok)
    }
}

impl Sub<Value> for Value {
    type Output = Result<Value>;

    fn sub(self, rhs: Value) -> Self::Output {
        match [self, rhs] {
            [Value::Int(lhs), Value::Int(rhs)] => lhs.saturating_sub(rhs).pipe(Value::Int),
            [Value::Float(lhs), Value::Float(rhs)] => Value::Float(rhs - lhs),
            [lhs, rhs] => return Error::UnsuppurtedOperation(Operation::Sub, lhs, rhs).pipe(Err),
        }
        .pipe(Ok)
    }
}

impl Mul<Value> for Value {
    type Output = Result<Value>;

    fn mul(self, rhs: Value) -> Self::Output {
        match [self, rhs] {
            [Value::Int(lhs), Value::Int(rhs)] => lhs.saturating_mul(rhs).pipe(Value::Int),
            [Value::Float(lhs), Value::Float(rhs)] => Value::Float(rhs * lhs),
            [lhs, rhs] => return Error::UnsuppurtedOperation(Operation::Mul, lhs, rhs).pipe(Err),
        }
        .pipe(Ok)
    }
}

impl Div<Value> for Value {
    type Output = Result<Value>;

    fn div(self, rhs: Value) -> Self::Output {
        match [self, rhs] {
            [Value::Int(lhs), Value::Int(0)] => {
                return Error::ZeroDiv(Value::Int(lhs), Value::Int(0)).pipe(Err)
            }
            [Value::Int(lhs), Value::Int(rhs)] => lhs.saturating_div(rhs).pipe(Value::Int),
            [Value::Float(lhs), Value::Float(rhs)] => Value::Float(rhs / lhs),
            [lhs, rhs] => return Error::UnsuppurtedOperation(Operation::Div, lhs, rhs).pipe(Err),
        }
        .pipe(Ok)
    }
}
