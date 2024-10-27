#![feature(unboxed_closures)]
#![feature(unsized_const_params)]

use std::marker::PhantomData as Ph;
use std::{fmt, ops};

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref REPR_FMT_REGEX: Regex = { Regex::new(r"([A-Za-z]+[0-9A-Za-z]*::)Node").unwrap() };
}

pub fn pretty_type<T: ?Sized>() -> std::borrow::Cow<'static, str> {
    let tstr_large_a = std::any::type_name::<T>();
    let tstr_a = REPR_FMT_REGEX.replace_all(tstr_large_a, "Node");
    tstr_a
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
        f.debug_struct(format!("Repr<Node<\"{}\", {}>>", NAME, pretty_type::<Deps>()).as_str())
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T> Meta<T> for Node<"Value", ()> {
    type Inner = T;
}

impl<T: ?Sized> Meta<T> for Node<"Undef", ()> {
    type Inner = Ph<Box<T>>;
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
            "FnOnce({}) -> {}",
            pretty_type::<A>(),
            pretty_type::<B>()
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
    println!("{:#?}", c);
    let d = new_ref(&c);
    println!("{:#?}", d);
    let f = new_lambda(|x| !x);
    let e = new_apply(f, c);
    println!("{:#?}", e);
}

/*

Standard Output
Repr<Node<"Not", (Node<"Add", (Node<"Value", ()>, i32, Node<"Undef", ()>, i32)>, i32)>> {
    inner: (
        123,
        PhantomData<alloc::boxed::Box<i32>>,
    ),
}
Repr<Node<"Ref", (&(), Node<"Not", (Node<"Add", (Node<"Value", ()>, i32, Node<"Undef", ()>, i32)>, i32)>)>> {
    inner: (
        123,
        PhantomData<alloc::boxed::Box<i32>>,
    ),
}
Repr<Node<"Apply", (Node<"Not", (Node<"Add", (Node<"Value", ()>, i32, Node<"Undef", ()>, i32)>, i32)>, i32, playground::main::{{closure}})>> {
    inner: (
        FnOnce(playground::Repr<Node<"Not", (Node<"Add", (Node<"Value", ()>, i32, Node<"Undef", ()>, i32)>, i32)>, i32>) -> playground::Repr<Node<"Not", (Node<"Not", (Node<"Add", (Node<"Value", ()>, i32, Node<"Undef", ()>, i32)>, i32)>, i32)>, i32>,
        (
            123,
            PhantomData<alloc::boxed::Box<i32>>,
        ),
    ),
}

*/
