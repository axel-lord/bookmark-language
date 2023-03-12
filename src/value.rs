use crate::{
    instruction::{self, Instruction},
    variable, Error, Result,
};
use derive_more::IsVariant;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::BTreeMap,
    ops::{Add, Div, Mul, Sub},
    sync::Arc,
};
use strum::EnumDiscriminants;
use tap::{Pipe, Tap};

#[derive(Debug, Default, Deserialize, Serialize, Clone, EnumDiscriminants, IsVariant)]
#[strum_discriminants(name(Type), derive(Serialize, Deserialize, Default, IsVariant))]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(Arc<str>),
    Id(variable::Id),
    Instruction(Box<Instruction>),
    List(Vec<Value>),
    Map(BTreeMap<Arc<str>, Value>),
    Type(Type),
    #[strum_discriminants(default)]
    #[default]
    None,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, IsVariant)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
}

impl Operation {
    pub fn apply(self, lhs: Value, rhs: Value) -> Result<Value> {
        match self {
            Operation::Add => lhs.add(rhs),
            Operation::Sub => lhs.sub(rhs),
            Operation::Mul => lhs.mul(rhs),
            Operation::Div => lhs.div(rhs),
        }
    }
}

impl Value {
    pub fn string(value: impl Into<Arc<str>>) -> Self {
        Self::String(value.into())
    }

    pub fn cast(self, to: Type) -> Result<Self> {
        if Type::from(&self) == to {
            return Ok(self);
        }

        match (self, to) {
            (_, Type::None) => Ok(Value::None),
            (value, Type::Type) => Ok(Value::Type(Type::from(value))),

            (Value::Int(value), Type::Float) => Ok(Value::Float(value as f64)),
            (Value::Float(value), Type::Int) => Ok(Value::Int(value.round() as i64)),

            (Value::Int(value), Type::Bool) => Ok(Value::Bool(value != 0)),
            (Value::Float(value), Type::Bool) => {
                Ok(Value::Bool((value.abs() == 0.0) || value.is_nan()))
            }
            (Value::String(value), Type::Bool) => Ok(Value::Bool(!value.is_empty())),
            (Value::Instruction(value), Type::Bool) => Ok(Value::Bool(!value.is_noop())),
            (Value::List(value), Type::Bool) => Ok(Value::Bool(!value.is_empty())),
            (Value::Map(value), Type::Bool) => Ok(Value::Bool(!value.is_empty())),
            (Value::None, Type::Bool) => Ok(Value::Bool(false)),

            (Value::Bool(value), Type::String) => Ok(Value::String(value.to_string().into())),
            (Value::Int(value), Type::String) => Ok(Value::String(value.to_string().into())),
            (Value::Float(value), Type::String) => Ok(Value::String(value.to_string().into())),

            (value, to) => Err(Error::InvalidCast(Type::from(&value), to, value)),
        }
    }

    pub fn parse(self, to: Type) -> Result<Self> {
        let Value::String(from) = self else {
            return Err(Error::NonStringParse(self));
        };

        if to.is_string() {
            return Ok(Value::String(from));
        }

        fn err<T>(to: Type, from: Arc<str>) -> impl Fn(T) -> Error {
            move |_| Error::FailedParse(to, Value::String(from.clone()))
        }

        match to {
            Type::Bool => from.parse().map(Value::Bool).map_err(err(to, from)),
            Type::Int => from.parse().map(Value::Int).map_err(err(to, from)),
            Type::Float => from.parse().map(Value::Float).map_err(err(to, from)),
            ty => Err(Error::InvalidParse(ty)),
        }
    }

    pub fn get(&self, key: Value) -> Result<&Value> {
        match (self, key) {
            (Value::Map(map), Value::String(key)) => {
                map.get(&key).ok_or_else(|| Error::InvalidAcces {
                    key: Value::String(key),
                    map: Value::Map(map.clone()),
                })
            }
            (Value::List(list), Value::Int(index)) => list
                .get(
                    TryInto::<usize>::try_into(index).map_err(|_| Error::InvalidAcces {
                        key: Value::Int(index),
                        map: Value::List(list.clone()),
                    })?,
                )
                .ok_or_else(|| Error::InvalidAcces {
                    key: Value::Int(index),
                    map: Value::List(list.clone()),
                }),
            (map, key) => Err(Error::WrongKeyType(key, Type::from(map))),
        }
    }

    pub fn get_mut(&mut self, key: Value) -> Result<&mut Value> {
        match (self, key) {
            (Value::Map(map), Value::String(key)) => {
                map.get_mut(&key).ok_or_else(|| Error::InvalidAcces {
                    key: Value::String(key),
                    map: Value::Type(Type::Map),
                })
            }
            (Value::List(list), Value::Int(index)) => list
                .get_mut(
                    TryInto::<usize>::try_into(index).map_err(|_| Error::InvalidAcces {
                        key: Value::Int(index),
                        map: Value::None,
                    })?,
                )
                .ok_or_else(|| Error::InvalidAcces {
                    key: Value::Int(index),
                    map: Value::Type(Type::List),
                }),
            (map, key) => Err(Error::WrongKeyType(key, <Type as From<&Value>>::from(map))),
        }
    }

    pub fn get_take(&mut self, key: Value) -> Result<Value> {
        match (self, key) {
            (Value::Map(ref mut map), Value::String(key)) => {
                map.remove(&key).ok_or_else(|| Error::InvalidAcces {
                    key: Value::String(key),
                    map: Value::Type(Type::Map),
                })
            }
            (Value::List(list), Value::Int(index)) => {
                let index_usize =
                    TryInto::<usize>::try_into(index).map_err(|_| Error::InvalidAcces {
                        key: Value::Int(index),
                        map: Value::None,
                    })?;
                if index_usize < list.len() {
                    Ok(list.remove(index_usize))
                } else {
                    Err(Error::InvalidAcces {
                        key: Value::Int(index),
                        map: Value::Type(Type::List),
                    })
                }
            }
            (map, key) => Err(Error::WrongKeyType(key, <Type as From<&Value>>::from(map))),
        }
    }
}

impl Add<Value> for Value {
    type Output = Result<Value>;

    fn add(self, rhs: Value) -> Self::Output {
        match [self, rhs] {
            [Value::Int(lhs), Value::Int(rhs)] => lhs
                .checked_add(rhs)
                .ok_or(Error::IntegerOverOrUnderFlow {
                    op: Operation::Add,
                    lhs: lhs.into(),
                    rhs: rhs.into(),
                })?
                .pipe(Value::Int),
            [Value::Float(lhs), Value::Float(rhs)] => Value::Float(lhs + rhs),
            [Value::String(lhs), Value::String(rhs)] => {
                Value::String(lhs.to_string().add(&rhs).into_boxed_str().into())
            }
            [Value::Instruction(lhs), Value::Instruction(rhs)] => vec![*lhs, *rhs]
                .pipe(instruction::meta::List)
                .pipe(Instruction::from)
                .pipe(Box::new)
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
            [Value::Int(lhs), Value::Int(rhs)] => lhs
                .checked_sub(rhs)
                .ok_or(Error::IntegerOverOrUnderFlow {
                    op: Operation::Sub,
                    lhs: lhs.into(),
                    rhs: rhs.into(),
                })?
                .pipe(Value::Int),
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
            [Value::Int(lhs), Value::Int(rhs)] => lhs
                .checked_mul(rhs)
                .ok_or(Error::IntegerOverOrUnderFlow {
                    op: Operation::Mul,
                    lhs: lhs.into(),
                    rhs: rhs.into(),
                })?
                .pipe(Value::Int),
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
            [Value::Int(lhs), Value::Int(rhs)] => lhs
                .checked_div(rhs)
                .ok_or(Error::IntegerOverOrUnderFlow {
                    op: Operation::Div,
                    lhs: lhs.into(),
                    rhs: rhs.into(),
                })?
                .pipe(Value::Int),
            [Value::Float(lhs), Value::Float(rhs)] => Value::Float(rhs / lhs),
            [lhs, rhs] => return Error::UnsuppurtedOperation(Operation::Div, lhs, rhs).pipe(Err),
        }
        .pipe(Ok)
    }
}

// Value from impls
impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<Instruction> for Value {
    fn from(value: Instruction) -> Self {
        Self::Instruction(Box::new(value))
    }
}

impl From<Box<Instruction>> for Value {
    fn from(value: Box<Instruction>) -> Self {
        Self::Instruction(value)
    }
}

impl From<Arc<str>> for Value {
    fn from(value: Arc<str>) -> Self {
        Self::String(value)
    }
}

impl From<&Arc<str>> for Value {
    fn from(value: &Arc<str>) -> Self {
        Self::String(Arc::clone(value))
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value.into())
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl From<Cow<'_, str>> for Value {
    fn from(value: Cow<'_, str>) -> Self {
        Self::String(value.into())
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<Vec<Value>> for Value {
    fn from(value: Vec<Value>) -> Self {
        Self::List(value)
    }
}

impl From<BTreeMap<Arc<str>, Value>> for Value {
    fn from(value: BTreeMap<Arc<str>, Value>) -> Self {
        Self::Map(value)
    }
}
