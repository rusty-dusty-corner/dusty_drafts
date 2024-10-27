#![feature(more_maybe_bounds)]
#![feature(never_type)]
#![feature(unsized_const_params)]
//#![feature(adt_const_params)]
#![feature(min_specialization)]
#![feature(rustc_attrs)]
#![feature(fn_traits)]
#![feature(unboxed_closures)]
#![feature(step_trait)]
#![feature(impl_trait_in_assoc_type)]
#![feature(associated_type_defaults)]

use std::iter::{ExactSizeIterator, Iterator};
use std::marker::PhantomData as Ph;

use num::traits::bounds::Bounded;

pub mod introspect {
    use super::Ph;

    pub use promise_cast::PromiseCast;
    pub mod promise_cast {
        use std::mem::ManuallyDrop as Md;

        #[rustc_unsafe_specialization_marker]
        pub trait SameType<A> {}

        // Output of align_of will be same, also size.
        // It is imposible to create other impl of `Same`.
        impl<A> SameType<A> for A {}

        pub union CastTool<A, B> {
            a: Md<A>,
            b: Md<B>,
        }

        pub trait PromiseCast<A>: Sized {
            fn promise_cast<const MSG: &'static str>(self) -> A;
            fn promise_default_method_output(self) -> A {
                self.promise_cast::<"
                    Output type of default method implementation is different,
                    in this case you must implement this method to satisfy this type
                ">()
            }
        }

        impl<A, B> PromiseCast<A> for B {
            default fn promise_cast<const MSG: &'static str>(self) -> A {
                unimplemented!("promise_cast: {}", MSG)
            }
        }

        impl<A: SameType<B>, B> PromiseCast<A> for B {
            #[inline(always)]
            fn promise_cast<const MSG: &'static str>(self) -> A {
                let tool = CastTool::<A, Self> { b: Md::new(self) };
                Md::into_inner(unsafe { tool.a })
            }
        }
    }

    pub trait Wrap {
        type Output<Impl> = Ph<Box<Impl>>;
        fn wrap<Impl>(imp: Impl) -> Self::Output<Impl> {
            Ph::<Box<Impl>>.promise_default_method_output()
        }
    }

    pub struct Direct;
    impl Wrap for Direct {
        type Output<Impl> = Impl;
        fn wrap<Impl>(imp: Impl) -> Self::Output<Impl> {
            imp
        }
    }

    pub struct Phantom;
    impl Wrap for Phantom {}

    pub trait Repr {
        type T;
        type W: Wrap = Phantom;
        type Unref: Repr = Value<!>;
    }

    pub type WrapOf<R, Impl> = <<R as Repr>::W as Wrap>::Output<Impl>;

    pub fn wrap<R: Repr, Impl>(imp: Impl) -> WrapOf<R, Impl> {
        <R::W as Wrap>::wrap(imp)
    }

    #[repr(transparent)]
    pub struct Value<T>(pub T);

    impl<T> Repr for Value<T> {
        type T = T;
        type W = Direct;
    }

    #[repr(transparent)]
    pub struct Undef<T>(pub Ph<Box<T>>);

    impl<T> Repr for Undef<T> {
        type T = T;
    }

    #[repr(transparent)]
    pub struct Ref<'a, This: Repr>(pub WrapOf<This, &'a This>);

    impl<'a, This: Repr> Repr for Ref<'a, This> {
        type T = &'a This::T;
        type W = This::W;
        type Unref = This;
    }
    
    type UnrefT<R> = <<R as Repr>::Unref as Repr>::T;

    pub trait IsRef<'a>: Repr {
        fn into_ref(self) -> Ref<'a, Self::Unref>;
    }

    impl<'a, This: Repr> IsRef<'a> for Ref<'a, This> {
        fn into_ref(self) -> Ref<'a, This> {
            self
        }
    }

    pub mod logic {
        use super::{wrap, Repr, WrapOf};

        pub struct Not<This: Repr>(pub WrapOf<This, This>);

        impl<This: Repr> Repr for Not<This>
        where
            This::T: std::ops::Not,
        {
            type T = <This::T as std::ops::Not>::Output;
            type W = This::W;
        }

        pub trait OpNot: Repr {
            type Output: Repr;

            // Required method
            fn not(self) -> Self::Output;
        }

        impl<This: Repr> OpNot for This
        where
            This::T: std::ops::Not,
        {
            type Output = impl Repr;
            fn not(self) -> Self::Output {
                Not::<This>(wrap::<This, This>(self))
            }
        }
    }

    pub mod partial_eq {
        use super::{logic::Not, IsRef, PromiseCast, Ref, Repr, WrapOf, UnrefT, wrap};

        pub struct OpEq<'a, This: Repr, Rhs: Repr>(
            WrapOf<This, Ref<'a, This>>,
            WrapOf<Rhs, Ref<'a, Rhs>>,
        );

        impl<'a, This: Repr, Rhs: Repr> Repr for OpEq<'a, This, Rhs>
        where
            UnrefT<This>: std::cmp::PartialEq<UnrefT<Rhs>>,
        {
            type T = bool;
            type W = This::W;
        }

        pub trait OpsTrait<Rhs: Repr = Self>: Repr
        where
            UnrefT<Rhs>: ?Sized,
        {
            // Required method
            type EqOutput: Repr<T = bool>;
            fn eq<'a>(self, other: Rhs) -> Self::EqOutput
            where
                Self: IsRef<'a>,
                Rhs: IsRef<'a>;

            // Provided method
            //type NeOutput<'a>: Repr<T = bool> = Not<Self::EqOutput<'a>>;
            //fn ne<'a>(&'a self, other: &'a Rhs) -> Self::NeOutput<'a> {
            //    Not(self.eq(other)).promise_default_method_output()
            //}
        }

        impl<This: Repr, Rhs: Repr> OpsTrait<Rhs> for This
        where
          UnrefT<This>: std::cmp::PartialEq<UnrefT<Rhs>>
          {
              type EqOutput = impl Repr<T = bool>;
              fn eq<'a>(self, other: Rhs) -> Self::EqOutput
            where
                Self: IsRef<'a>,
                Rhs: IsRef<'a> {
                OpEq::<'a, Self, Rhs>(wrap::<Self, _>(self.into_ref()), wrap(other.into_ref()))
            }
          }
    }
}

/*
macro_rules! skip

pub trait PartialEq<Rhs = Self>
where
    Rhs: ?Sized,
{
    // Required method
    fn eq(&self, other: &Rhs) -> bool;

    // Provided method
    fn ne(&self, other: &Rhs) -> bool { ... }
}
*/

/*

pub trait BitXor<Rhs = Self> {
    type Output;

    // Required method
    fn bitxor(self, rhs: Rhs) -> Self::Output;
}

pub trait PartialOrd<Rhs = Self>: PartialEq<Rhs>
where
    Rhs: ?Sized,
{
    // Required method
    fn partial_cmp(&self, other: &Rhs) -> Option<Ordering>;

pub trait PartialEq<Rhs = Self>
where
    Rhs: ?Sized,
{
    // Required method
    fn eq(&self, other: &Rhs) -> bool;

#[derive(Default)]
struct Fgen;
impl<'a> FnOnce<(&'a u64,)> for Fgen {
    type Output = u64;
    extern "rust-call" fn call_once(self, b: (&'a u64,)) -> Self::Output {
        *b.0 ^ 0x123
    }
}

#[derive(Default)]
struct Pred;
impl<'a> FnOnce<(&'a u64,)> for Pred {
    type Output = bool;
    extern "rust-call" fn call_once(self, b: (&'a u64,)) -> Self::Output {
        *b.0 < 35 || *b.0 % 31 == 0
    }
}
*/
