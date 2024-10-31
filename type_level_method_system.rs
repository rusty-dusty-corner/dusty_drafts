

#![feature(tuple_trait)]
#![feature(fn_traits)]
#![feature(unboxed_closures)]
#![feature(trait_alias)]
#![feature(generic_const_exprs)]
#![feature(unsized_const_params)]
#![feature(associated_type_defaults)]

/*

This is improvement, work in progress and has some problems to solve

you can see previous working idea and test in on rust playgrounud
https://play.rust-lang.org/?version=nightly&mode=debug&edition=2021&gist=61dae45291daf09905ab47ad9d89cb47

*/

use std::marker::PhantomData as Ph;

pub trait GenTrait {
    type Type = ();
    const USIZE: usize = 0;
    const REF_STATIC_STR: &'static str = "";
}

pub struct GenType<const IMPL: &'static str, T, Deps>(Ph<(T, Deps)>);
impl<const IMPL: &'static str, T, Deps> GenTrait for GenType<IMPL, T, Deps> {
    type Type = T;
}

pub struct GenConstUSize<const A: usize>;
impl<const A: usize> GenTrait for GenConstUSize<A> {
    const USIZE: usize = A;
}

pub struct GenConstRefStaticStr<const A: &'static str>;
impl<const A: &'static str> GenTrait for GenConstRefStaticStr<A> {
    const REF_STATIC_STR: &'static str = A;
}

pub struct _A;
pub struct _B;
pub struct _C;

pub trait Equal<A> {
    type GenTrait<Yes: GenTrait, No: GenTrait>: GenTrait;
    type CommonImpls<Yes: CommonImpls, No: CommonImpls>: CommonImpls;
}

macro_rules! equal_impls { ($({$A:ident $B:ident $C:ident})*) => { $(
    impl Equal<$A> for $B {
        type GenTrait<Yes: GenTrait, No: GenTrait> = $C;
        type CommonImpls<Yes: CommonImpls, No: CommonImpls> = $C;
    }
)* } }

equal_impls! {
    { _A _A Yes } { _A _B No } { _A _C No }
    { _B _B Yes } { _B _A No } { _B _C No }
    { _C _C Yes } { _C _A No } { _C _B No }
}

pub struct ICons<Ident, Gen, Tail>(Ph<(Ident, Gen, Tail)>);
pub struct Nil;

pub trait Lookup<Ident> {
    type Gen: GenTrait = GenType<"Type", (), ()>;
    type Common: CommonImpls = Nil;
}

impl<Ident> Lookup<Ident> for Nil {}

impl<A, Ident, Gen, Tail> Lookup<Ident> for ICons<A, Gen, Tail>
where
    A: Equal<Ident>,
    Gen: GenTrait,
    Tail: Lookup<Ident>,
    Self: CommonImpls,
{
    type Gen = A::GenTrait<Gen, <Tail as Lookup<Ident>>::Gen>;
    type Common = A::CommonImpls<Self, <Tail as Lookup<Ident>>::Common>;
}

pub struct DebugMsg;

pub trait ConstParamTest<Ci: CommonImpls> {
    const MSG: &'static str = Ci::REF_STATIC_STR;
}

impl<Ci: CommonImpls> ConstParamTest<Ci> for DebugMsg {}

pub trait CommonImpls: Sized {
    type Tail: CommonImpls = Nil;
    const USIZE: usize = 0;
    const REF_STATIC_STR: &'static str = "";
    type Type = ();
    type IntoU64: Into<u64> = u64;
    type IntoSelfIntoU64: Into<Self::IntoU64> = Self::IntoU64;
    type ConstParamTest: ConstParamTest<Self::ConstParamTestArg> = DebugMsg;
    type ConstParamTestArg: CommonImpls = Nil;
}

impl CommonImpls for Nil {}

impl<const A: usize, Ident, Tail: CommonImpls> CommonImpls
    for ICons<Ident, GenConstUSize<A>, Tail>
{
    type Tail = Tail;
    const USIZE: usize = A;
}

impl<const A: &'static str, Ident, Tail: CommonImpls> CommonImpls
    for ICons<Ident, GenConstRefStaticStr<A>, Tail>
{
    type Tail = Tail;
    const REF_STATIC_STR: &'static str = A;
}

impl<Ident, T, Tail: CommonImpls> CommonImpls for ICons<Ident, GenType<"IntoU64", T, ()>, Tail>
where
    T: Into<u64>,
{
    type Tail = Tail;
    type IntoU64 = T;
}

impl<Ident, T, A, Tail: CommonImpls> CommonImpls
    for ICons<Ident, GenType<"IntoSelfIntoU64", T, (A,)>, Tail>
where
    Tail: Lookup<A>,
    <Tail::Gen as GenTrait>::Type: Into<u64>,
    T: Into<<Tail::Gen as GenTrait>::Type>,
{
    type Tail = Tail;
    type IntoU64 = <Tail::Gen as GenTrait>::Type;
    type IntoSelfIntoU64 = T;
}

impl<Ident, T, A, Tail: CommonImpls> CommonImpls
    for ICons<Ident, GenType<"ConstParamTest", T, (A,)>, Tail>
where
    Tail: Lookup<A>,
    T: ConstParamTest<Tail::Common>,
{
    type Tail = Tail;
    type ConstParamTest = T;
    type ConstParamTestArg = Tail::Common;
}

macro_rules! constraints { ($($t:tt)*) => {
    $($t)*
    where
        [(); <Tail<Tail<Ci>> as CommonImpls>::USIZE]:,
    ;
} }

pub trait Constraints = CommonImpls where [(); <Tail<Tail<Self>> as CommonImpls>::USIZE]:;

pub trait Method<const NAME: &'static str> {
    constraints!(
        type Hint<Ci: CommonImpls>: CommonImpls
    );
    constraints!(
        type Args<Ci: CommonImpls>: std::marker::Tuple
    );
    type Output: CommonImpls;
    constraints!(
        fn method<Ci: CommonImpls>(self, args: Self::Args<Ci>) -> Self::Output
    );
}

pub type Tail<Ci> = <Ci as CommonImpls>::Tail;

pub fn example<const _A: usize, _B: Into<u64>, _C: Into<_B>>(_a: (), _args: Nil) -> Nil {
    Nil
}

impl Method<"example"> for () {
    type Hint<Ci: CommonImpls>
        = ICons<
        _C,
        GenType<"IntoSelfIntoU64", Ci::IntoSelfIntoU64, (_B,)>,
        ICons<
            _B,
            GenType<"IntoU64", <Tail<Ci> as CommonImpls>::IntoU64, ()>,
            ICons<_A, GenConstUSize<{ <Tail<Tail<Ci>> as CommonImpls>::USIZE }>, Nil>,
        >,
    >
    where
        [(); <Tail<Tail<Ci>> as CommonImpls>::USIZE]:;
    type Args<Ci: CommonImpls>
        = (
        GenConstUSize<{ <Tail<Tail<Ci>> as CommonImpls>::USIZE }>,
        <Tail<Ci> as CommonImpls>::IntoU64,
        Ci::IntoSelfIntoU64,
    )
    where
        [(); <Tail<Tail<Ci>> as CommonImpls>::USIZE]:;
    type Output = Nil;
    fn method<Ci: CommonImpls>(self, args: Self::Args<Ci>) -> Self::Output
    where
        [(); <Tail<Tail<Ci>> as CommonImpls>::USIZE]:,
    {
        Nil
    }
}

pub struct CallOwn<const NAME: &'static str, M: Method<NAME>, Ci: CommonImpls>(
    M,
    Ph<(Ci, M::Hint<Ci>)>,
)
where
    [(); <Tail<Tail<Ci>> as CommonImpls>::USIZE]:;
impl<const NAME: &'static str, M: Method<NAME>, Ci: CommonImpls> FnOnce<M::Args<Ci>>
    for CallOwn<NAME, M, Ci>
where
    [(); <Tail<Tail<Ci>> as CommonImpls>::USIZE]:,
{
    type Output = M::Output;
    extern "rust-call" fn call_once(self, args: M::Args<Ci>) -> Self::Output {
        self.0.method::<Ci>(args)
    }
}

pub const fn hint<A>() -> Ph<(A, A)> {
    Ph
}

#[test]
fn test_example() {
    type Call<Ci> = CallOwn<"example", (), Ci>;
    let call: Call<_> = CallOwn((), hint());
    call(GenConstUSize::<234>, 123_u64, 321_u32);
}
