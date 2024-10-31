
#![feature(trait_alias)]
#![feature(associated_type_defaults)]
#![allow(incomplete_features)]
#![feature(unsized_const_params)]

use std::marker::PhantomData as Ph;

pub trait NameTrait {
    const NAME: &'static str;
}

pub struct Name<const NAME: &'static str>;

impl<const NAME: &'static str> NameTrait for Name<NAME> {
    const NAME: &'static str = NAME;
}

pub type DepA_<T> = <T as Accessor>::DepA;
pub type DepB_<T> = <T as Accessor>::DepB;

pub type Basic_<'a, T> = <T as Accessor>::Basic<'a>;
pub type Rotation_<'a, T> = <T as Accessor>::Rotation<'a>;

pub type Ground_<T> = <T as Accessor>::Ground;
pub type Mapping_<'a, T, B> = <T as Accessor>::Mapping<'a, B>;
pub type GroundMapping_<'a, T, B> = Mapping_<'a, Ground_<T>, B>;

pub type Place_<'a, T, G, P> = <T as Transformable<G>>::Place<'a, P>;
pub type PlaceBasic_<'a, T, G> = Place_<'a, T, G, Basic_<'a, T>>;
pub type PlaceRotation_<'a, T, G> = Place_<'a, T, G, Rotation_<'a, T>>;

pub type BasicMapping_<'a, T, B> = Basic_<'a, Mapping_<'a, T, B>>;
pub type RotationMapping_<'a, T, B> = Rotation_<'a, Mapping_<'a, T, B>>;

pub type PlaceBasicGroundMapping<'a, T, B> =
    PlaceBasic_<'a, GroundMapping_<'a, T, B>, Mapping_<'a, T, B>>;

pub type PlaceRotationGroundMapping<'a, T, B> =
    PlaceRotation_<'a, GroundMapping_<'a, T, B>, Mapping_<'a, T, B>>;

pub mod mapping_method_constraints {
    use super::*;

    pub trait LifetimeC<'a, B: 'a> = Accessor
    where
        DepA_<Self>: 'a,
        DepA_<Ground_<Self>>: 'a,
        DepA_<Mapping_<'a, Self, B>>: 'a,
        DepA_<GroundMapping_<'a, Self, B>>: 'a,
        DepB_<Self>: 'a,
        DepB_<Ground_<Self>>: 'a,
        DepB_<Mapping_<'a, Self, B>>: 'a,
        DepB_<GroundMapping_<'a, Self, B>>: 'a;

    pub trait TransformableC<'a, B: 'a> = LifetimeC<'a, B>
    where
        Ground_<Self>: Transformable<Self>,
        GroundMapping_<'a, Self, B>: Transformable<Mapping_<'a, Self, B>>;

    pub trait ConversionC<'a, B: 'a> = TransformableC<'a, B>
    where
        PlaceBasic_<'a, Ground_<Self>, Self>: From<Basic_<'a, Self>>,
        PlaceRotation_<'a, Ground_<Self>, Self>: Into<Rotation_<'a, Self>>,
        PlaceRotationGroundMapping<'a, Self, B>: From<RotationMapping_<'a, Self, B>>,
        PlaceBasicGroundMapping<'a, Self, B>: Into<BasicMapping_<'a, Self, B>>;

    pub trait AllC<'a, F, B: 'a> = ConversionC<'a, B>
    where
        F: FnOnce(Rotation_<'a, Self>) -> Rotation_<'a, Mapping_<'a, Self, B>>;
}

pub trait Accessor: Sized {
    type Id: NameTrait;
    type Field;
    type DepA = Self::Field;
    type DepB = Self::DepA;

    type Basic<'a>: 'a
        = Self::DepA
    where
        Self::DepA: 'a,
        Self::DepB: 'a;

    type Rotation<'a>: 'a
        = Self::Basic<'a>
    where
        Self::DepA: 'a,
        Self::DepB: 'a;

    type Ground: Accessor<Field = Self::Field>;
    type Mapping<'a, B: 'a>: Accessor<Id = Self::Id, Field = B>;

    fn mapping<'a, F, B: 'a>(f: F, a: Self::Basic<'a>) -> BasicMapping_<'a, Self, B>
    where
        Self: mapping_method_constraints::AllC<'a, F, B>,
    {
        let rotation = Self::Ground::rotate(From::from(a));
        let output = From::from(f(rotation.into()));
        GroundMapping_::<'a, Self, B>::restore(output).into()
    }
}

pub trait Transformable<M: Accessor<Field = Self::Field>>: Accessor {
    type Place<'a, Focus: 'a>: 'a = Focus;
    fn rotate<'a>(a: PlaceBasic_<'a, Self, M>) -> PlaceRotation_<'a, Self, M>
    where
        Self::DepA: 'a,
        Self::DepB: 'a;
    fn restore<'a>(a: PlaceRotation_<'a, Self, M>) -> PlaceBasic_<'a, Self, M>
    where
        Self::DepA: 'a,
        Self::DepB: 'a;
}

pub struct Meta<Id: NameTrait, Extra>(Ph<(Id, Extra)>);

pub type IdentM<Field> = Meta<Name<"Ident">, Field>;

impl<Field> Accessor for IdentM<Field> {
    type Id = Name<"Ident">;
    type Field = Field;
    type Ground = Self;
    type Mapping<'a, B: 'a> = IdentM<B>;
}

impl<Field> Transformable<Self> for IdentM<Field> {
    fn rotate<'a>(a: PlaceBasic_<'a, Self, Self>) -> PlaceRotation_<'a, Self, Self>
    where
        Self::DepA: 'a,
        Self::DepB: 'a,
    {
        a
    }
    fn restore<'a>(a: PlaceRotation_<'a, Self, Self>) -> PlaceBasic_<'a, Self, Self>
    where
        Self::DepA: 'a,
        Self::DepB: 'a,
    {
        a
    }
}

pub type RefM<Ground> = Meta<Name<"Ref">, Ground>;

impl<Ground: Accessor> Accessor for RefM<Ground> {
    type Id = Name<"Ref">;
    type Field = Ground::Field;
    type DepA = Ground::DepA;
    type DepB = Ground::DepB;

    type Basic<'a>
        = &'a Ground::Basic<'a>
    where
        Self::DepA: 'a,
        Self::DepB: 'a;

    type Rotation<'a>
        = &'a Ground::Rotation<'a>
    where
        Self::DepA: 'a,
        Self::DepB: 'a;

    type Ground = Ground;
    type Mapping<'a, B: 'a> = RefM<GroundMapping_<'a, Self, B>>;
}

impl<Field> Transformable<RefM<IdentM<Field>>> for IdentM<Field> {
    type Place<'a, Focus: 'a> = &'a Focus;
    fn rotate<'a>(a: Self::Place<'a, Self::Basic<'a>>) -> Self::Place<'a, Self::Rotation<'a>>
    where
        Self::DepA: 'a,
        Self::DepB: 'a,
    {
        a
    }
    fn restore<'a>(a: Self::Place<'a, Self::Rotation<'a>>) -> Self::Place<'a, Self::Basic<'a>>
    where
        Self::DepA: 'a,
        Self::DepB: 'a,
    {
        a
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapping() {
        type IdentityType = IdentM<i32>;

        let value = 42;
        let mapping_fn = |x: i32| x * 2;

        let result = IdentityType::mapping(mapping_fn, value);
        assert_eq!(result, 84);
    }

    #[test]
    fn test_complex_transformation() {
        type BaseType = IdentM<(u8, u32)>;
        type RefType = RefM<BaseType>;

        let value = (1_u8, 2_u32);
        let reference = &value;

        let transformed = RefType::mapping(|x| &x.0, reference);
        assert_eq!(*transformed, 1_u8);
    }

    #[repr(transparent)]
    pub struct Marker<Name: NameTrait, T>(T, Ph<Name>);

    impl<T> From<Marker<Name<"Unfocused">, T>> for Marker<Name<"Focusing on first">, T> {
        fn from(a: Marker<Name<"Unfocused">, T>) -> Self {
            Marker(a.0, Ph)
        }
    }

    impl<T> From<Marker<Name<"Focusing on first">, T>> for Marker<Name<"Unfocused">, T> {
        fn from(a: Marker<Name<"Focusing on first">, T>) -> Self {
            Marker(a.0, Ph)
        }
    }

    impl<P, Q> Accessor for Marker<Name<"Focusing on first">, (P, Q)> {
        type Id = Name<"Marker of pair">;
        type Field = P;
        type DepA = P;
        type DepB = Q;

        type Basic<'a>
            = Marker<Name<"Unfocused">, (P, Q)>
        where
            Self::DepA: 'a,
            Self::DepB: 'a;

        type Rotation<'a>
            = Marker<Name<"Focusing on first">, (P, Q)>
        where
            Self::DepA: 'a,
            Self::DepB: 'a;

        type Ground = IdentM<P>;
        type Mapping<'a, B: 'a> = Marker<Name<"Focusing on first">, (B, Q)>;
    }

    // FIXME, we must solve problem with lifetime deps, so `Q` is not needed to be stateic lifetime
    impl<P, Q: 'static> Transformable<Marker<Name<"Focusing on first">, (P, Q)>> for IdentM<P> {
        type Place<'a, Focus: 'a> = Marker<Name<"Focusing on first">, (Focus, Q)>;
        fn rotate<'a>(a: Self::Place<'a, Self::Basic<'a>>) -> Self::Place<'a, Self::Rotation<'a>>
        where
            Self::DepA: 'a,
            Self::DepB: 'a,
        {
            a
        }
        fn restore<'a>(a: Self::Place<'a, Self::Rotation<'a>>) -> Self::Place<'a, Self::Basic<'a>>
        where
            Self::DepA: 'a,
            Self::DepB: 'a,
        {
            a
        }
    }

    #[test]
    fn test_marker() {
        type MarkerType = Marker<Name<"Focusing on first">, (u8, u16)>;

        let value = Marker::<Name<"Unfocused">, _>((1_u8, 2_u16), Ph);

        let transformed = MarkerType::mapping(|x| Marker((x.0 .0 + 2, x.0 .1), x.1), value);
        assert_eq!(transformed.0 .0, 3_u8);
    }
}
