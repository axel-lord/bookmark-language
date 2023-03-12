macro_rules! subenum {
    ($([$ename:ident, $([$sname:ident, $ty:ty],)*],)* ) => {
        $(
        #[derive(Debug, Deserialize, Serialize, Clone)]
        pub enum $ename {
            $(
            $sname($ty),
            )*
        }
        $(
        impl IntoInstruction for $ty {
            fn into_instruction(self) -> Instruction {
                Instruction::$ename($ename::$sname(self))
            }
        }
        )*
        )*
    };
}

macro_rules! instr {
    (
        Pure: [$($pu_name:ident($pu_ty:ty)),* $(,)?],
        Reading: [$($re_name:ident($re_ty:ty)),* $(,)?],
        Mutating: [$($mu_name:ident($mu_ty:ty)),* $(,)?],
        Meta: [$($me_name:ident($me_ty:ty)),* $(,)?]
        ) => {

        $crate::instruction::set_macro::subenum!(
            [Mutating, $([$mu_name, $mu_ty],)*],
            [Pure, $([$pu_name, $pu_ty],)*],
            [Reading, $([$re_name, $re_ty],)*],
            [Meta, $([$me_name, $me_ty],)*],
        );

        impl  Mutating {
            pub fn perform(
                self,
                return_value: Value,
                variables: variable::Map
            ) -> Result<(Value, variable::Map)> {
                use traits::Mutating as _;
                match self {
                    $(
                    Self::$mu_name(instr) => instr.perform(return_value, variables),
                    )*
                }
            }
        }

        impl Meta {
            pub fn perform(
                self,
                return_value: Value,
                variables: variable::Map,
                instruction_stack: Stack,
            ) -> Result<(Value, variable::Map, Stack)> {
                use traits::Meta as _;
                match self {
                    $(
                    Self::$me_name(instr) => instr.perform(return_value, variables, instruction_stack),
                    )*
                }
            }
        }

        impl Pure {
            pub fn perform(
                self,
                return_value: Value,
            ) -> Result<Value> {
                // use traits::Pure as _;
                // match self {
                //     $(
                //     Self::$me_name(instr) => instr.perform(return_value),
                //     )*
                // }
                todo!()
            }
        }

        impl Reading {
            pub fn perform(
                self,
                return_value: Value,
                variables: &variable::Map,
            ) -> Result<Value> {
                // use traits::Meta as _;
                // match self {
                //     $(
                //     Self::$me_name(instr) => instr.perform(return_value, variables, instruction_stack),
                //     )*
                // }
                todo!()
            }
        }
    };
}

pub(crate) use instr;
pub(crate) use subenum;
