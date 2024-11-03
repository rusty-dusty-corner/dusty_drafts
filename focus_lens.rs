

#![feature(impl_trait_in_assoc_type)]
#![feature(associated_type_defaults)]
#![feature(unsized_const_params)]
#![allow(dead_code)]

use std::marker::PhantomData as Ph;

pub struct Ident<const IDENT: &'static str>;

pub trait IdentT {
    const IDENT: &'static str;
}

impl<const IDENT: &'static str> IdentT for Ident<IDENT> {
    const IDENT: &'static str = IDENT;
}

pub struct Field<const NAME: &'static str>;

pub trait Path {}

impl<const NAME: &'static str> Path for Field<NAME> {}

pub trait FocusOp<P: Path>: Sized {
    type Id: IdentT;
    type T;
    type Side;
    type Focus<T>: FocusMapOp<T = T> = Focus<(T, Self::Side)>;
    fn focus<B>(self) -> Self::Focus<Self::T>
    where
        <Self::Focus<Self::T> as FocusMapOp>::Output<B>: RebuildFromFocus<P, Self, T<B> = B>;
}

pub trait RebuildFromFocus<P: Path, Fc: FocusOp<P>> {
    type T<A> = Fc::T;
    type Output<A>: FocusOp<P, Id = Fc::Id, T = Self::T<A>, Side = Fc::Side>;
    fn rebuild_from_focus<A>(self) -> Self::Output<Self::T<A>>;
}

pub trait FocusMapOp {
    type T;
    fn focus_take(self) -> Self::T;
    fn focus_ref(&self) -> &Self::T;
    fn focus_mut(&mut self) -> &mut Self::T;
    type Output<B>: FocusMapOp<T = B>;
    fn focus_map<B, F: FnOnce(Self::T) -> B>(self, f: F) -> Self::Output<B>;
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Focus<T>(T);

impl<A, Side> FocusMapOp for Focus<(A, Side)> {
    type T = A;
    fn focus_take(self) -> Self::T {
        self.0 .0
    }
    fn focus_ref(&self) -> &Self::T {
        &self.0 .0
    }
    fn focus_mut(&mut self) -> &mut Self::T {
        &mut self.0 .0
    }
    type Output<B> = Focus<(B, Side)>;
    fn focus_map<B, F: FnOnce(Self::T) -> B>(self, f: F) -> Self::Output<B> {
        Focus((f(self.0 .0), self.0 .1))
    }
}

#[derive(Debug)]
struct Tag<V> {
    name: String,
    value: V,
}

impl<V> FocusOp<Field<"name">> for Tag<V> {
    type Id = Ident<"Tag">;
    type T = String;
    type Side = V;
    fn focus<B>(self) -> Self::Focus<Self::T>
    where
        <Self::Focus<Self::T> as FocusMapOp>::Output<B>:
            RebuildFromFocus<Field<"name">, Self, T<B> = B>,
    {
        Focus((self.name, self.value))
    }
}

impl<V> RebuildFromFocus<Field<"name">, Tag<V>> for Focus<(String, V)> {
    type Output<A> = Tag<V>;
    fn rebuild_from_focus<A>(self) -> Self::Output<A> {
        Tag {
            name: self.0 .0,
            value: self.0 .1,
        }
    }
}

fn main() {
    let tag = Tag {
        name: "test".to_string(),
        value: 123_usize,
    };
    println!("{:#?}", &tag);
    let focus = tag.focus();
    println!("{:#?}", &focus);
    let mapped = focus.focus_map(|x| x + "2");
    println!("{:#?}", &mapped);
    let rebuilded = mapped.rebuild_from_focus::<()>();
    println!("{:#?}", &rebuilded);
}
