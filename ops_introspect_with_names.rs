#![feature(unboxed_closures)]
#![allow(incomplete_features)]
#![feature(unsized_const_params)]

use std::marker::PhantomData as Ph;
use std::{fmt, ops};

use lazy_static::lazy_static;
use regex::Regex;

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
    impl<'ast> Visit<'ast> for Indent {
        fn visit_type_tuple(&mut self, node: &'ast TypeTuple) {
            if node.elems.len() > 0 {
                self.inside_tuple = true;
                self.output.push_str("\n");
                self.output.push_str("    ".repeat(self.indent).as_str());
                self.output.push_str("(");
                self.indent += 1;
                visit::visit_type_tuple(self, node);
                self.indent -= 1;
                self.output.push_str("\n");
                self.output.push_str("    ".repeat(self.indent).as_str());
                self.output.push_str(")");
                self.inside_tuple = false;
            }
        }
        fn visit_type(&mut self, node: &'ast Type) {
            if self.inside_tuple || self.start {
                let backup = self.inside_tuple;
                self.inside_tuple = false;
                let tstr = quote::quote!(#node).to_string();
                let tstr2 = REPR_FMT_REGEX_TUPLE.replace(tstr.as_str(), "_");
                if !self.start {
                    self.output.push_str("\n");
                }
                self.start = false;
                self.output.push_str("    ".repeat(self.indent).as_str());
                self.output.push_str(&tstr2);
                self.output.push_str(",");
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
    //quote::quote!(#typ).to_string()
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

pub fn repr<const NAME: &'static str, Deps: ?Sized, T>(
    inner: InnerOf<Repr<Node<NAME, Deps>, T>>,
) -> Repr<Node<NAME, Deps>, T>
where
    Node<NAME, Deps>: Meta<T>,
{
    Repr { inner }
}

impl<const NAME: &'static str, Deps: ?Sized, T> fmt::Debug for Repr<Node<NAME, Deps>, T>
where
    Node<NAME, Deps>: Meta<T>,
    InnerOf<Self>: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(pretty_type::<Self>(0).as_str())
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T> Meta<T> for Node<"Value", ()> {
    type Inner = T;
}

impl<T: ?Sized> Meta<T> for Node<"Undef", ()> {
    type Inner = Ph<T>;
}

impl<'a, M: Meta<T>, T> Meta<&'a T> for Node<"Ref", (&'a (), M)>
where
    M::Inner: 'a,
{
    type Inner = &'a M::Inner;
}

fn new_ref<'a, M: Meta<T>, T>(repr: &'a Repr<M, T>) -> Repr<Node<"Ref", (&'a (), M)>, &'a T>
where
    M::Inner: 'a,
{
    Repr { inner: &repr.inner }
}

impl<M: Meta<T>, T: ops::Not> Meta<T::Output> for Node<"Not", (M, T)> {
    type Inner = M::Inner;
}

impl<M: Meta<T>, T: ops::Not> ops::Not for Repr<M, T> {
    type Output = Repr<Node<"Not", (M, T)>, T::Output>;
    fn not(self) -> Self::Output {
        Repr { inner: self.inner }
    }
}

impl<ThisM: Meta<ThisT>, ThisT, RhsM: Meta<RhsT>, RhsT> Meta<ThisT::Output>
    for Node<"Add", (ThisM, ThisT, RhsM, RhsT)>
where
    ThisT: ops::Add<RhsT>,
{
    type Inner = (ThisM::Inner, RhsM::Inner);
}

impl<ThisM: Meta<ThisT>, ThisT, RhsM: Meta<RhsT>, RhsT> ops::Add<Repr<RhsM, RhsT>>
    for Repr<ThisM, ThisT>
where
    ThisT: ops::Add<RhsT>,
{
    type Output = Repr<Node<"Add", (ThisM, ThisT, RhsM, RhsT)>, ThisT::Output>;
    fn add(self, rhs: Repr<RhsM, RhsT>) -> Self::Output {
        Repr {
            inner: (self.inner, rhs.inner),
        }
    }
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

impl<M: Meta<T>, T, F> Meta<F> for Node<"Lambda", (M, T)>
where
    F: FnOnce<(Repr<M, T>,)>,
    F::Output: ReprTrait,
{
    type Inner = DebugLambda<Repr<M, T>, F::Output, F>;
}

fn new_lambda<M: Meta<T>, T, F>(inner: F) -> Repr<Node<"Lambda", (M, T)>, F>
where
    F: FnOnce<(Repr<M, T>,)>,
    F::Output: ReprTrait,
{
    Repr {
        inner: DebugLambda(inner, Ph),
    }
}

impl<M: Meta<T>, T, F> Meta<F::Output> for Node<"Apply", (M, T, F)>
where
    F: FnOnce<(Repr<M, T>,)>,
    F::Output: ReprTrait,
{
    type Inner = (DebugLambda<Repr<M, T>, F::Output, F>, M::Inner);
}

fn new_apply<M: Meta<T>, T, F>(
    lam: Repr<Node<"Lambda", (M, T)>, F>,
    arg: Repr<M, T>,
) -> Repr<Node<"Apply", (M, T, F)>, F::Output>
where
    F: FnOnce<(Repr<M, T>,)>,
    F::Output: ReprTrait,
{
    Repr {
        inner: (lam.inner, arg.inner),
    }
}

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

*/
