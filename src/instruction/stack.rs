use super::Instruction;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Stack(Vec<Instruction>);

impl Stack {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) -> &mut Self {
        self.0.clear();
        self
    }

    pub fn push(&mut self, instr: impl Into<Instruction>) -> &mut Self {
        self.0.push(instr.into());
        self
    }

    pub fn extend(&mut self, instrs: impl IntoIterator<Item = Instruction>) -> &mut Self {
        self.0.extend(instrs.into_iter());
        self
    }

    pub fn pop(&mut self) -> Option<Instruction> {
        self.0.pop()
    }
}

impl From<Vec<Instruction>> for Stack {
    fn from(value: Vec<Instruction>) -> Self {
        Self(value)
    }
}
