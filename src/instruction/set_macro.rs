macro_rules! subenum {
    ($ename:ident, $($sname:ident, $ty:ty,)*) => {
        #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
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
    };
}

macro_rules! subenum_gen {
    ($ename:ident, $($sname:ident),*) => {
        paste::paste! {
            $crate::instruction::set_macro::subenum!(
                $ename,
                $(
                    $sname,
                    [< $ename:lower >] :: $sname,
                )*
            );
        }
    };
}

macro_rules! instr {
    (
        $(
            $sect:ident($($in_n:ident: $in_ty:ty),* $(,)? ) -> $out_ty:ty: [
                $($name:ident),+ $(,)?
            ]
        ),* $(,)?
        ) => {

        $(
        $crate::instruction::set_macro::subenum_gen!(
            $sect, $($name),*
        );

        impl $sect {
            pub fn perform(self, $($in_n: $in_ty),*) -> Result<$out_ty> {
                 use instr_traits::$sect as _;
                 let perf_in = ($($in_n,)*);

                 match self {
                     $(
                     Self::$name(instr) => instr.perform_tup(perf_in),
                     )*
                 }
            }
        }
        )*
        pub mod instr_traits {
            use super::*;

            $(
            pub trait $sect: Debug + Serialize + Deserialize<'static> + Clone + PartialEq {
                fn perform(self, $($in_n: $in_ty),*) -> Result<$out_ty>;

                fn perform_tup(self, tup: ($($in_ty,)*)) -> Result<$out_ty> {
                    let ($($in_n,)*): ($($in_ty,)*) = tup;
                    self.perform($($in_n),*)
                }
            }
            )*
        }
    };
}

pub(crate) use instr;
pub(crate) use subenum;
pub(crate) use subenum_gen;
