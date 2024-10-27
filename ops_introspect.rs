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

    pub trait Repr {
        type T;
        type Unref: Repr = Value<!>;
    }

    #[repr(transparent)]
    pub struct Value<T>(pub T);

    impl<T> Repr for Value<T> {
        type T = T;
    }

    #[repr(transparent)]
    pub struct Undef<T>(pub Ph<Box<T>>);

    impl<T> Repr for Undef<T> {
        type T = T;
    }

    #[repr(transparent)]
    pub struct Ref<'a, This: Repr>(pub &'a This);

    impl<'a, This: Repr> Repr for Ref<'a, This> {
        type T = &'a This::T;
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
        use super::Repr;

        pub struct Not<This: Repr>(pub This);

        impl<This: Repr> Repr for Not<This>
        where
            This::T: std::ops::Not,
        {
            type T = <This::T as std::ops::Not>::Output;
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
                Not(self)
            }
        }
    }

    pub mod partial_eq {
        use super::{logic::Not, IsRef, PromiseCast, Ref, Repr, UnrefT};

        pub struct OpEq<'a, This: Repr, Rhs: Repr>(Ref<'a, This>, Ref<'a, Rhs>);

        impl<'a, This: Repr, Rhs: Repr> Repr for OpEq<'a, This, Rhs>
        where
        //UnrefT<This>: std::cmp::PartialEq<UnrefT<Rhs>>,
        {
            type T = bool;
        }

        pub trait OpsTrait<Rhs: Repr = Self>: Repr
        where
            UnrefT<Rhs>: ?Sized,
        {
            // Required method
            type EqOutput<'a>: Repr<T = bool>
            where
                Self: IsRef<'a>,
                Rhs: IsRef<'a>,
                Self::Unref: 'a,
                Rhs::Unref: 'a;
            fn eq<'a>(self, other: Rhs) -> Self::EqOutput<'a>
            where
                Self: IsRef<'a>,
                Rhs: IsRef<'a>,
                Self::Unref: 'a,
                Rhs::Unref: 'a;

            // Provided method
            type NeOutput<'a>: Repr<T = bool>
                = Not<Self::EqOutput<'a>>
            where
                Self: IsRef<'a>,
                Rhs: IsRef<'a>,
                Self::Unref: 'a,
                Rhs::Unref: 'a;
            fn ne<'a>(self, other: Rhs) -> Self::NeOutput<'a>
            where
                Self: Sized + IsRef<'a>,
                Rhs: IsRef<'a>,
                Self::Unref: 'a,
                Rhs::Unref: 'a,
            {
                Not(self.eq(other)).promise_default_method_output()
            }
        }

        impl<This: Repr, Rhs: Repr> OpsTrait<Rhs> for This
        where
            UnrefT<This>: std::cmp::PartialEq<UnrefT<Rhs>>,
        {
            type EqOutput<'a>
                = impl Repr<T = bool>
            where
                Self: IsRef<'a>,
                Rhs: IsRef<'a>,
                Self::Unref: 'a,
                Rhs::Unref: 'a;
            fn eq<'a>(self, other: Rhs) -> Self::EqOutput<'a>
            where
                Self: IsRef<'a>,
                Rhs: IsRef<'a>,
                Self::Unref: 'a,
                Rhs::Unref: 'a,
            {
                OpEq(self.into_ref(), other.into_ref())
            }
        }
    }
}

