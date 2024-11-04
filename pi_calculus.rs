#![feature(associated_type_defaults)]

use std::marker::PhantomData as Ph;

pub struct _A<T>(T);
pub struct _B<T>(T);

pub trait Name {}

impl Name for () {}
impl<T: Name> Name for _A<T> {}
impl<T: Name> Name for _B<T> {}

pub trait Same<T: Name>: Name {
    type Case<Yes, No> = No;
    type EvalYesCase<Yes: Eval, No> = No;
}

impl<T: Name> Same<()> for _A<T> {}
impl<T: Name> Same<()> for _B<T> {}
impl<T: Name> Same<_A<T>> for () {}
impl<T: Name> Same<_B<T>> for () {}
impl<C: Name, D: Same<C>> Same<_A<C>> for _B<D> {}
impl<C: Name, D: Same<C>> Same<_B<C>> for _A<D> {}

impl Same<()> for () {
    type Case<Yes, No> = Yes;
    type EvalYesCase<Yes: Eval, No> = EvalOf<Yes>;
}

impl<C: Name, D: Same<C>> Same<_A<C>> for _A<D> {
    type Case<Yes, No> = D::Case<Yes, No>;
    type EvalYesCase<Yes: Eval, No> = D::EvalYesCase<Yes, No>;
}

impl<C: Name, D: Same<C>> Same<_B<C>> for _B<D> {
    type Case<Yes, No> = D::Case<Yes, No>;
    type EvalYesCase<Yes: Eval, No> = D::EvalYesCase<Yes, No>;
}

pub struct Comm<Pa, Pb>(Ph<(Pa, Pb)>);

pub struct Send<ChName, Chan>(Ph<(ChName, Chan)>);

pub struct Recv<ChName, Var, Expr>(Ph<(ChName, Var, Expr)>);

pub struct Bind<ChName, Var, Expr>(Ph<(ChName, Var, Expr)>);

pub trait Eval: Sized {
    type Success<Yes, No> = Yes;
    type Oper<This>;
    type Output = Self::Oper<Self>;
}

impl Eval for () {
    type Oper<This> = This;
}

pub type EvalOf<T> = <T as Eval>::Output;
pub type OperOf<T, A> = <T as Eval>::Oper<A>;
pub type SuccessOf<T, Yes, No> = <T as Eval>::Success<Yes, No>;

impl<A: Eval> Eval for Comm<A, ()> {
    type Success<Yes, No> = A::Success<Yes, No>;
    type Oper<This> = A;
}

impl<A, B> Eval for Send<A, B>
{
    type Success<Yes, No> = No;
    type Oper<This> = This;
}

impl<A, B, C, D, N> Eval for Comm<Send<A, B>, Recv<C, D, N>>
where
    C: Name,
    A: Same<C>,
    Bind<B, D, N>: Eval,
{
    type Success<Yes, No> = A::Case<Yes, No>;
    type Oper<This> = A::EvalYesCase<Bind<B, D, N>, This>;
}

impl<A, B, C, D, N> Eval for Comm<Recv<C, D, N>, Send<A, B>>
where
    C: Name,
    A: Same<C>,
    Bind<B, D, N>: Eval,
{
    type Success<Yes, No> = A::Case<Yes, No>;
    type Oper<This> = A::EvalYesCase<Bind<B, D, N>, This>;
}

/*
impl<A, B, C> Eval for Comm<Comm<A, B>, C>
where
    Comm<A, B>: Eval,
    Comm<A, Comm<B, C>>: Eval,
{
    type Success<Yes, No> = SuccessOf<Comm<A, B>, Yes, SuccessOf<Comm<A, Comm<B, C>>, Yes, No>>;
    type Oper<This> =
        SuccessOf<Comm<A, B>, Comm<EvalOf<Comm<A, B>>, C>, EvalOf<Comm<A, Comm<B, C>>>>;
}
*/

impl<A, B, C, D> Eval for Comm<Send<A, B>, Comm<C, D>>
where
    Comm<Send<A, B>, C>: Eval,
    Comm<Send<A, B>, D>: Eval,
{
    type Success<Yes, No> =
        SuccessOf<Comm<Send<A, B>, C>, Yes, SuccessOf<Comm<Send<A, B>, D>, Yes, No>>;
    type Oper<This> = SuccessOf<
        Comm<Send<A, B>, C>,
        Comm<EvalOf<Comm<Send<A, B>, C>>, D>,
        SuccessOf<Comm<Send<A, B>, D>, Comm<EvalOf<Comm<Send<A, B>, D>>, C>, This>,
    >;
}

impl<A, B> Eval for Bind<A, B, ()> {
    type Oper<This> = ();
}

impl<ChName, Var, A, B> Eval for Bind<ChName, Var, Comm<A, B>>
where
    Bind<ChName, Var, A>: Eval,
    Bind<ChName, Var, B>: Eval,
{
    type Oper<This> = Comm<EvalOf<Bind<ChName, Var, A>>, EvalOf<Bind<ChName, Var, B>>>;
}

impl<ChName, Var, A, B> Eval for Bind<ChName, Var, Send<A, B>>
where
    Var: Name,
    A: Same<Var>,
    B: Same<Var>,
{
    type Oper<This> = A::Case<
        B::Case<Send<ChName, ChName>, Send<ChName, B>>,
        B::Case<Send<A, ChName>, Send<A, B>>,
    >;
}

impl<ChName, Var, A, B, E> Eval for Bind<ChName, Var, Recv<A, B, E>>
where
    ChName: Name,
    Var: Name,
    A: Same<Var>,
    B: Same<Var>,
    Bind<ChName, Var, E>: Eval,
{
    type Oper<This> = A::Case<
        B::Case<
            Recv<ChName, ChName, EvalOf<Bind<ChName, Var, E>>>,
            Recv<ChName, B, EvalOf<Bind<ChName, Var, E>>>,
        >,
        B::Case<
            Recv<A, ChName, EvalOf<Bind<ChName, Var, E>>>,
            Recv<A, B, EvalOf<Bind<ChName, Var, E>>>,
        >,
    >;
}

pub type TestA =
    Comm<Send<_A<()>, _B<_A<()>>>, Comm<(), Recv<_A<()>, _B<()>, Comm<(), Send<_B<()>, _B<()>>>>>>;

fn main() {
    println!("{:#?}", std::any::type_name::<EvalOf<TestA>>());
}

/*

Standard Output

"playground::Comm<playground::Comm<(), playground::Send<playground::_B<playground::_A<()>>, playground::_B<playground::_A<()>>>>, ()>"

*/
