use super::{pure::op_fn, traits::Reading};
use crate::{
    value::{self, Operation, Value},
    variable, Result,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Clone(pub variable::Id);
impl Reading for Clone {
    fn perform(self, _return_value: Value, variables: &variable::Map) -> Result<Value> {
        let Self(id) = self;

        variables.read(id).cloned()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct GetClone(pub variable::Id);
impl Reading for GetClone {
    fn perform(self, return_value: Value, variables: &variable::Map) -> Result<Value> {
        let Self(id) = self;

        variables.read(id)?.get(return_value).cloned()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct OpClone(pub value::Operation, pub variable::Id);
impl Reading for OpClone {
    fn perform(self, return_value: Value, variables: &variable::Map) -> Result<Value> {
        let Self(operation, id) = self;

        operation.apply(return_value, variables.read(id)?.clone())
    }
}

op_fn![
    (OpClone, id, variable::Id, clone),
    (add, Operation::Add),
    (sub, Operation::Sub),
    (mul, Operation::Mul),
    (div, Operation::Div),
];
