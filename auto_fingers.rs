#![feature(step_trait)]
#![feature(impl_trait_in_assoc_type)]
#![feature(associated_type_defaults)]

use std::iter::{ExactSizeIterator, Iterator};
use std::marker::PhantomData as Ph;

pub trait AutoFingers {
    type Powers: Iterator<Item = Self>;
    fn powers(&self) -> Self::Powers;
    type XorFingers: Iterator<Item = Self>;
    fn xor_fingers(&self) -> Self::XorFingers;
    fn wrapping_successor(&self) -> Self;
}

impl<T> AutoFingers for T
where
    T: 'static + Clone + From<u16> + std::iter::Step,
    T: std::ops::Shl<u16, Output = T>,
    T: std::ops::BitXor<Output = T>,
    T: std::ops::Add<Output = T>,
{
    type Powers = impl Iterator<Item = Self>;
    fn powers(&self) -> Self::Powers {
        (0_u16..8191_u16)
            .zip(1_u16..8192_u16)
            .map(|(p, q)| {
                let one = || -> T { 1.into() };
                let a: T = one() << p;
                let b: T = one() << q;
                if a < b {
                    Some(a)
                } else {
                    None
                }
            })
            .take_while(|x| x.is_some())
            .map(|x| x.unwrap())
    }
    type XorFingers = impl Iterator<Item = Self>;
    fn xor_fingers(&self) -> Self::XorFingers {
        let pos = self.clone();
        self.powers().map(move |x| x ^ pos.clone())
    }
    fn wrapping_successor(&self) -> Self {
        let one: T = 1.into();
        let a = one + self.clone();
        if &a == self {
            0.into()
        } else {
            a
        }
    }
}

pub trait Fingers: Sized {
    type Finger = Self;
    type Fingers: Iterator<Item = Self::Finger>;
    fn fingers(&self) -> Self::Fingers;
    fn successor(&self) -> Self::Finger;
}

pub trait Index: PartialOrd + Fingers<Finger = Self> {
    const ZERO: Self;
}

macro_rules! _auto_impls {
    ($T:ty) => {
        impl Fingers for $T {
            type Fingers = impl Iterator<Item = Self>;
            fn fingers(&self) -> Self::Fingers {
                self.xor_fingers()
            }
            fn successor(&self) -> Self::Finger {
                self.wrapping_successor()
            }
        }
        impl Index for $T {
            const ZERO: Self = 0;
        }
    };
}

macro_rules! auto_impls { ($($T:ty)*) => { $(_auto_impls! { $T })* } }

auto_impls! { u64 }

pub trait Coverage: Sized {
    type Item;

    type Index: Index = u64;

    const LAST_INDEX: Self::Index;

    fn cursor_index(&self) -> Self::Index;

    fn cursor_jump(&mut self, idx: &Self::Index);

    fn raw_sample(idx: &Self::Index) -> Self::Item;

    fn predicate(item: &Self::Item) -> bool {
        true
    }

    fn on_hit(&mut self, idx: &Self::Index) {}

    fn on_fail(&mut self, idx: &Self::Index) {}

    fn sample(&mut self, idx: &Self::Index) -> Option<Self::Item> {
        if idx > &Self::LAST_INDEX {
            None
        } else {
            let smp = Self::raw_sample(idx);
            if Self::predicate(&smp) {
                self.on_hit(idx);
                Some(smp)
            } else {
                self.on_fail(idx);
                None
            }
        }
    }

    fn shallow_discover(&mut self) -> Option<Self::Item> {
        let cur = self.cursor_index();
        if let Some(hit) = self.sample(&cur) {
            return Some(hit);
        }
        for idx in self.fingers() {
            if let Some(hit) = self.sample(&idx) {
                self.cursor_jump(&idx);
                return Some(hit);
            }
        }
        self.cursor_jump(&self.successor());
        None
    }
}

impl<T: Coverage> Fingers for T {
    type Finger = T::Index;
    type Fingers = impl Iterator<Item = Self::Finger>;
    fn fingers(&self) -> Self::Fingers {
        self.cursor_index()
            .fingers()
            .take_while(|x| x <= &Self::LAST_INDEX)
    }
    fn successor(&self) -> Self::Finger {
        let a = self.successor();
        if a > Self::LAST_INDEX {
            T::Index::ZERO
        } else {
            a
        }
    }
}
