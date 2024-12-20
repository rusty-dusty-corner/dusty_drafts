#![feature(unboxed_closures)]
#![allow(incomplete_features)]
#![feature(unsized_const_params)]

use std::marker::PhantomData as Ph;
use std::{fmt, ops};

use lazy_static::lazy_static;
use regex::Regex;

// Very interesting that rustfmt is supported, and rust code inside "const _: ()" can be formatted :)
macro_rules! shared_generics {
    ([$($t:tt)*]) => {};

    (
        [[$($Gen:tt)*][$($Whp:tt)*][$($Als:tt)*]]
        impl __<[$($Trt:tt)*]> for $Typ:ty $(where $($Wht:ty: __<[$($Wtr:tt)*]>),* $(,)? )? { $($code:tt)* }
        $($rest:tt)*
    ) => {
        const _: () = {
            $($Als)*
            impl $($Gen)* $($Trt)* for $Typ where $($( $Wht: $($Wtr)*, )*)? $($Whp)* { $($code)* }
        };
        shared_generics! { [[$($Gen)*][$($Whp)*][$($Als)*]] $($rest)* }
    };

    (
        [[$($Gen:tt)*][$($Whp:tt)*][$($Als:tt)*]]
        $vis:vis fn $name:ident<__> ($($args:tt)*) $( -> $Typ:ty )? { $($code:tt)* }
        $($rest:tt)*
    ) => {
        paste::paste! {
            mod [<__ $name _with_aliases__>] {
                use super::*;
                $($Als)*
                pub fn $name $($Gen)* ($($args)*) $( -> $Typ )? where $($Whp)* { $($code)* }
            }
            $vis use [<__ $name _with_aliases__>]::$name;
        }
        shared_generics! { [[$($Gen)*][$($Whp)*][$($Als)*]] $($rest)* }
    };

    (
        #[params($($Gen:tt)*)]
        $(#[_where($($Whp:tt)*)])*
        $(#[__([ $Alias:ident!() = $Aty:ty ]__)])*
        const _: () = {$($code:tt)*};
    ) => {
        shared_generics! {
            [
                [$($Gen)*]
                [$($($Whp)*,)*]
                [$(
                    #[allow(unused_macros)]
                    macro_rules! $Alias { () => { $Aty } }
                )*]
            ]
            $($code)*
        }
    };
}

pub trait Meta<T: ?Sized> {
    type Inner;
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Repr<M: Meta<T>, T: ?Sized> {
    pub inner: M::Inner,
}

pub trait ReprTrait {
    type M: Meta<Self::T>;
    type T: ?Sized;
    fn repr(self) -> Repr<Self::M, Self::T>;
}

impl<M: Meta<T>, T> ReprTrait for Repr<M, T> {
    type M = M;
    type T = T;
    fn repr(self) -> Repr<Self::M, Self::T> {
        self
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Node<const NAME: &'static str, Deps: ?Sized>(Ph<Box<Deps>>);

pub type InnerOf<R> = <<R as ReprTrait>::M as Meta<<R as ReprTrait>::T>>::Inner;

// So very interesting, i use rustfmt here and it works, also addition `where` predicates is used in `impl` :)
shared_generics!(
    #[params(< const NAME: &'static str, Deps: ?Sized, T >)]
    #[_where( Node<NAME, Deps>: Meta<T> )]
    #[__([ Repr!() = Repr<Node<NAME, Deps>, T> ]__)]
    const _: () = {
        pub fn repr<__>(inner: InnerOf<Repr!()>) -> Repr!() {
            Repr { inner }
        }

        impl __<[fmt::Debug]> for Repr!()
        where
            InnerOf<Self>: __<[fmt::Debug]>,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(pretty_type::<Self>(0).as_str())
                    .field("inner", &self.inner)
                    .finish()
            }
        }
    };
);

impl<T> Meta<T> for Node<"Value", ()> {
    type Inner = T;
}

impl<T: ?Sized> Meta<T> for Node<"Undef", ()> {
    type Inner = Ph<T>;
}

shared_generics!(
    #[params(< 'a, M: Meta<T>, T >)]
    #[_where( M::Inner: 'a )]
    #[__([ Node!() = Node<"Ref", (&'a (), M)> ]__)]
    const _: () = {
        impl __<[Meta<&'a T>]> for Node!() {
            type Inner = &'a M::Inner;
        }

        fn new_ref<__>(repr: &'a Repr<M, T>) -> Repr<Node!(), &'a T> {
            Repr { inner: &repr.inner }
        }
    };
);

macro_rules! decl_unary_op {
    ($NAME:literal, $Name:ident, $name:ident) => {
        shared_generics!(
            #[params(< M: Meta<T>, T: ops::$Name >)]
            #[__([ Node!() = Node<$NAME, (M, T)> ]__)]
            const _: () = {
                impl __<[Meta<T::Output>]> for Node!() {
                    type Inner = M::Inner;
                }

                impl __<[ops::$Name]> for Repr<M, T> {
                    type Output = Repr<Node!(), T::Output>;
                    fn $name(self) -> Self::Output {
                        Repr { inner: self.inner }
                    }
                }
            };
        );
    };
}

macro_rules! decl_binary_op {
    ($NAME:literal, $Name:ident, $name:ident) => {
        shared_generics!(
            #[params(< ThisM: Meta<ThisT>, ThisT, RhsM: Meta<RhsT>, RhsT >)]
            #[_where( ThisT: ops::$Name<RhsT> )]
            #[__([ Node!() = Node<$NAME, (ThisM, ThisT, RhsM, RhsT)> ]__)]
            #[__([ Rhs!() = Repr<RhsM, RhsT> ]__)]
            const _: () = {
                impl __<[Meta<ThisT::Output>]> for Node!() {
                    type Inner = (ThisM::Inner, RhsM::Inner);
                }

                impl __<[ops::$Name<Rhs!()>]> for Repr<ThisM, ThisT> {
                    type Output = Repr<Node!(), ThisT::Output>;
                    fn $name(self, rhs: Rhs!()) -> Self::Output {
                        Repr {
                            inner: (self.inner, rhs.inner),
                        }
                    }
                }
            };
        );
    };
}

macro_rules! decl_unary_ops { ($($NAME:literal),* $(,)?) => { paste::paste! {
    $(decl_unary_op! { $NAME, [<$NAME:camel>], [<$NAME:lower>] })*
} } }

macro_rules! decl_binary_ops { ($($NAME:literal),* $(,)?) => { paste::paste! {
    $(decl_binary_op! { $NAME, [<$NAME:camel>], [<$NAME:lower>] })*
} } }

decl_unary_ops! {
  "Not", "Neg",
}

decl_binary_ops! {
    "Add", "Sub",
    "Mul", "Div", "Rem",
    "Shl", "Shr",
    "BitAnd", "BitOr", "BitXor",
}

#[repr(transparent)]
pub struct DebugLambda<A, B, F>(F, Ph<(A, B)>);

impl<A, B, F> fmt::Debug for DebugLambda<A, B, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "impl FnOnce\n    (\n{}\n    ) ->\n{}",
            pretty_type::<A>(2),
            pretty_type::<B>(2)
        ))
    }
}

shared_generics!(
    #[params(< M: Meta<T>, T, F >)]
    #[_where( F: FnOnce<(Repr<M, T>,)> )]
    #[_where( F::Output: ReprTrait )]
    #[__([ NodeLambda!() = Node<"Lambda", (M, T)> ]__)]
    #[__([ NodeApply!() = Node<"Apply", (M, T, F)> ]__)]
    #[__([ DebugLambda!() = DebugLambda<Repr<M, T>, F::Output, F> ]__)]
    const _: () = {
        impl __<[Meta<F>]> for NodeLambda!() {
            type Inner = DebugLambda!();
        }

        fn new_lambda<__>(inner: F) -> Repr<NodeLambda!(), F> {
            Repr {
                inner: DebugLambda(inner, Ph),
            }
        }

        impl __<[Meta<F::Output>]> for NodeApply!() {
            type Inner = (DebugLambda!(), M::Inner);
        }

        fn new_apply<__>(
            lam: Repr<NodeLambda!(), F>,
            arg: Repr<M, T>,
        ) -> Repr<NodeApply!(), F::Output> {
            Repr {
                inner: (lam.inner, arg.inner),
            }
        }
    };
);

fn main() {
    let a = repr::<"Value", (), _>(123_i32);
    let b = repr::<"Undef", (), i32>(Ph);
    let c = !(a + b);
    println!("{:#?}\n\n", c);
    let d = new_ref(&c);
    println!("{:#?}\n\n", d);
    let f = new_lambda(|x| !x);
    println!("{:#?}\n\n", f);
    let e = new_apply(f, c);
    println!("{:#?}\n\n", e);
    let g = (c - a) * b % a ^ b & a / repr::<"Undef", (), i32>(Ph);
    println!("{:#?}\n\n", g);
}

lazy_static! {
    static ref REPR_FMT_REGEX_PATH: Regex = Regex::new(r"([A-Za-z][0-9A-Za-z]*::)*").unwrap();
    static ref REPR_FMT_REGEX_CLOSURE: Regex = Regex::new(r"\{\{closure\}\}").unwrap();
    static ref REPR_FMT_REGEX_TUPLE: Regex = Regex::new(r"\(.+\)").unwrap();
}

pub fn pretty_type<T: ?Sized>(indent: usize) -> String {
    use syn::visit::{self, Visit};
    use syn::{self, Type, TypeTuple};
    let tstr_long = std::any::type_name::<T>();
    let tstr1 = REPR_FMT_REGEX_PATH.replace_all(tstr_long, "");
    let tstr2 = REPR_FMT_REGEX_CLOSURE.replace_all(&tstr1, "_Closure_");
    let typ = syn::parse_str::<syn::Type>(&tstr2).unwrap();
    struct Indent {
        indent: usize,
        output: String,
        start: bool,
        inside_tuple: bool,
    }
    impl Indent {
        fn out(&mut self, a: &[&str]) {
            for b in a {
                self.output.push_str(b);
            }
        }
    }
    impl<'ast> Visit<'ast> for Indent {
        fn visit_type_tuple(&mut self, node: &'ast TypeTuple) {
            if node.elems.len() > 0 {
                self.out(&["\n", "    ".repeat(self.indent).as_str(), "("]);
                self.indent += 1;
                self.inside_tuple = true;
                visit::visit_type_tuple(self, node);
                self.indent -= 1;
                self.inside_tuple = false;
                self.out(&["\n", "    ".repeat(self.indent).as_str(), ")"]);
            }
        }
        fn visit_type(&mut self, node: &'ast Type) {
            if self.inside_tuple || self.start {
                let tstr = quote::quote!(#node).to_string();
                let hstr = REPR_FMT_REGEX_TUPLE.replace(tstr.as_str(), "_");
                if !self.start {
                    self.out(&["\n"]);
                }
                self.out(&["    ".repeat(self.indent).as_str(), &hstr, ","]);
                self.start = false;
                let backup = self.inside_tuple;
                self.inside_tuple = false;
                visit::visit_type(self, node);
                self.inside_tuple = backup;
            } else {
                visit::visit_type(self, node);
            }
        }
    }
    let mut indent = Indent {
        indent,
        output: String::new(),
        start: true,
        inside_tuple: false,
    };
    indent.visit_type(&typ);
    indent.output
}

/*

Standard Output

Repr < Node < "Not" , _ > , i32 >,
(
    Node < "Add" , _ >,
    (
        Node < "Value" , () >,
        i32,
        Node < "Undef" , () >,
        i32,
    )
    i32,
) {
    inner: (
        123,
        PhantomData<i32>,
    ),
}


Repr < Node < "Ref" , _ > , & i32 >,
(
    & (),
    Node < "Not" , _ >,
    (
        Node < "Add" , _ >,
        (
            Node < "Value" , () >,
            i32,
            Node < "Undef" , () >,
            i32,
        )
        i32,
    )
) {
    inner: (
        123,
        PhantomData<i32>,
    ),
}


Repr < Node < "Lambda" , _ > , _Closure_ >,
(
    Node < "Not" , _ >,
    (
        Node < "Add" , _ >,
        (
            Node < "Value" , () >,
            i32,
            Node < "Undef" , () >,
            i32,
        )
        i32,
    )
    i32,
) {
    inner: impl FnOnce
        (
            Repr < Node < "Not" , _ > , i32 >,
            (
                Node < "Add" , _ >,
                (
                    Node < "Value" , () >,
                    i32,
                    Node < "Undef" , () >,
                    i32,
                )
                i32,
            )
        ) ->
            Repr < Node < "Not" , _ > , i32 >,
            (
                Node < "Not" , _ >,
                (
                    Node < "Add" , _ >,
                    (
                        Node < "Value" , () >,
                        i32,
                        Node < "Undef" , () >,
                        i32,
                    )
                    i32,
                )
                i32,
            ),
}


Repr < Node < "Apply" , _ > , i32 > >,
(
    Node < "Not" , _ >,
    (
        Node < "Add" , _ >,
        (
            Node < "Value" , () >,
            i32,
            Node < "Undef" , () >,
            i32,
        )
        i32,
    )
    i32,
    _Closure_,
)
(
    Node < "Not" , _ >,
    (
        Node < "Add" , _ >,
        (
            Node < "Value" , () >,
            i32,
            Node < "Undef" , () >,
            i32,
        )
        i32,
    )
    i32,
) {
    inner: (
        impl FnOnce
            (
                Repr < Node < "Not" , _ > , i32 >,
                (
                    Node < "Add" , _ >,
                    (
                        Node < "Value" , () >,
                        i32,
                        Node < "Undef" , () >,
                        i32,
                    )
                    i32,
                )
            ) ->
                Repr < Node < "Not" , _ > , i32 >,
                (
                    Node < "Not" , _ >,
                    (
                        Node < "Add" , _ >,
                        (
                            Node < "Value" , () >,
                            i32,
                            Node < "Undef" , () >,
                            i32,
                        )
                        i32,
                    )
                    i32,
                ),
        (
            123,
            PhantomData<i32>,
        ),
    ),
}


Repr < Node < "BitXor" , _ > , i32 >,
(
    Node < "Rem" , _ >,
    (
        Node < "Mul" , _ >,
        (
            Node < "Sub" , _ >,
            (
                Node < "Not" , _ >,
                (
                    Node < "Add" , _ >,
                    (
                        Node < "Value" , () >,
                        i32,
                        Node < "Undef" , () >,
                        i32,
                    )
                    i32,
                )
                i32,
                Node < "Value" , () >,
                i32,
            )
            i32,
            Node < "Undef" , () >,
            i32,
        )
        i32,
        Node < "Value" , () >,
        i32,
    )
    i32,
    Node < "BitAnd" , _ >,
    (
        Node < "Undef" , () >,
        i32,
        Node < "Div" , _ >,
        (
            Node < "Value" , () >,
            i32,
            Node < "Undef" , () >,
            i32,
        )
        i32,
    )
    i32,
) {
    inner: (
        (
            (
                (
                    (
                        123,
                        PhantomData<i32>,
                    ),
                    123,
                ),
                PhantomData<i32>,
            ),
            123,
        ),
        (
            PhantomData<i32>,
            (
                123,
                PhantomData<i32>,
            ),
        ),
    ),
}

*/
