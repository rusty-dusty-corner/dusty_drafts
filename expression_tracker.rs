#![feature(negative_impls)]
#![feature(auto_traits)]
#![allow(incomplete_features)]
#![feature(unsized_const_params)]

use std::marker::PhantomData as Ph;
use std::ops;

// Type level tracked imaginary value of some type
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct TLtriv<const OP: &'static str, Deps, T: ?Sized>(pub Deps, pub Ph<T>);

pub trait GetType {
    type T;
}
impl<const OP: &'static str, Deps, T> GetType for TLtriv<OP, Deps, T> {
    type T = T;
}

pub auto trait IsNotTLtriv {}
impl<T: IsNotTLtriv> GetType for T {
    type T = T;
}
impl<const OP: &'static str, Deps, T: ?Sized> !IsNotTLtriv for TLtriv<OP, Deps, T> {}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Fix<T>(pub T);

pub fn undef<T: ?Sized>() -> TLtriv<"undef", (), T> {
    TLtriv((), Ph)
}

macro_rules! decl_unary_op {
    ($NAME:literal, $Name:ident, $name:ident) => {
        impl<const OP: &'static str, Deps, T: ops::$Name> ops::$Name for TLtriv<OP, Deps, T> {
            type Output = TLtriv<$NAME, Self, T::Output>;
            fn $name(self) -> Self::Output {
                TLtriv(self, Ph)
            }
        }
    };
}

macro_rules! decl_unary_ops { ($($NAME:literal),* $(,)?) => { paste::paste! {
    $(decl_unary_op!($NAME, [<$NAME:camel>], [<$NAME:lower>]);)*
} } }

decl_unary_ops! { "Not", "Neg" }

macro_rules! decl_binary_op {
    ($NAME:literal, $Name:ident, $name:ident) => {
        impl<const OP: &'static str, Deps, Lhs: ops::$Name<Rhs::T>, Rhs: GetType> ops::$Name<Rhs>
            for Fix<TLtriv<OP, Deps, Lhs>>
        {
            type Output = TLtriv<$NAME, (TLtriv<OP, Deps, Lhs>, Rhs), Lhs::Output>;
            fn $name(self, rhs: Rhs) -> Self::Output {
                TLtriv((self.0, rhs), Ph)
            }
        }

        impl<const OP: &'static str, Deps, Lhs: IsNotTLtriv + ops::$Name<Rhs>, Rhs>
            ops::$Name<TLtriv<OP, Deps, Rhs>> for Fix<Lhs>
        {
            type Output = TLtriv<$NAME, (Lhs, TLtriv<OP, Deps, Rhs>), Lhs::Output>;
            fn $name(self, rhs: TLtriv<OP, Deps, Rhs>) -> Self::Output {
                TLtriv((self.0, rhs), Ph)
            }
        }

        impl<Lhs: IsNotTLtriv + ops::$Name<Rhs>, Rhs: IsNotTLtriv> ops::$Name<Rhs> for Fix<Lhs> {
            type Output = Lhs::Output;
            fn $name(self, rhs: Rhs) -> Self::Output {
                self.0.$name(rhs)
            }
        }
    };
}

macro_rules! decl_binary_ops { ($($NAME:literal),* $(,)?) => { paste::paste! {
    $(decl_binary_op!($NAME, [<$NAME:camel>], [<$NAME:lower>]);)*
} } }

decl_binary_ops! {
    "Add", "Sub",
    "Mul", "Div", "Rem",
    "Shl", "Shr",
    "BitAnd", "BitOr", "BitXor",
}

fn main() {
    let a = undef::<i32>();
    let b = Fix(a) + 123_i32;
    let c = undef::<i32>();
    let d = -(Fix(Fix(b) ^ c) - 1_i32);
    let e = Fix(Fix(2_i32) + 3_i32) + d;
    println!("{:#?}", &e);
}

/*

Standard Output

TLtriv(
    (
        5,
        TLtriv(
            TLtriv(
                (
                    TLtriv(
                        (
                            TLtriv(
                                (
                                    TLtriv(
                                        (),
                                        PhantomData<i32>,
                                    ),
                                    123,
                                ),
                                PhantomData<i32>,
                            ),
                            TLtriv(
                                (),
                                PhantomData<i32>,
                            ),
                        ),
                        PhantomData<i32>,
                    ),
                    1,
                ),
                PhantomData<i32>,
            ),
            PhantomData<i32>,
        ),
    ),
    PhantomData<i32>,
)

*/
