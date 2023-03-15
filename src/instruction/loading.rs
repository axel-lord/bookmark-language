use super::instr_traits::Loading;
use super::traits::Loader;
use crate::{program, value::Value, Result};
use serde::{Deserialize, Serialize};
use std::{mem, sync::Arc};
use tap::Pipe;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Program(pub Arc<program::Program>);
impl Loading for Program {
    fn perform(self, return_value: Value, loader: &dyn Loader) -> Result<Value> {
        let Self(mut arc_prgr) = self;

        arc_prgr
            .pipe_ref_mut(Arc::make_mut)
            .pipe(mem::take)
            .run(return_value, loader)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub struct Load;
impl Loading for Load {
    fn perform(self, return_value: Value, loader: &dyn Loader) -> Result<Value> {
        loader.load(return_value)
    }
}
