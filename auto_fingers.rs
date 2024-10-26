#![feature(fn_traits)]
#![feature(unboxed_closures)]
#![feature(step_trait)]
#![feature(impl_trait_in_assoc_type)]
#![feature(associated_type_defaults)]

use std::iter::{ExactSizeIterator, Iterator};
use std::marker::PhantomData as Ph;

use num::traits::bounds::Bounded;

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
    T: Bounded,
    T: std::ops::Shl<u16, Output = T>,
    T: std::ops::Shr<u16, Output = T>,
    T: std::ops::BitXor<Output = T>,
    T: std::ops::Add<Output = T>,
{
    type Powers = impl Iterator<Item = Self>;
    fn powers(&self) -> Self::Powers {
        (1_u16..8191_u16)
            .map(|p| {
                let one = || -> T { 1.into() };
                let a: T = one() << p;
                if a > T::max_value() >> 1_u16 {
                    None
                } else {
                    Some(a)
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
            self.cursor_jump(&self.successor());
            return Some(hit);
        }
        for idx in self.fingers() {
            if let Some(hit) = self.sample(&idx) {
                self.cursor_jump(&idx);
                self.cursor_jump(&self.successor());
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
        let a = self.cursor_index().successor();
        if a > Self::LAST_INDEX {
            T::Index::ZERO
        } else {
            a
        }
    }
}

pub struct SimpleCov<Item, Fgen, Pred> {
    cursor_index: u64,
    _marker: Ph<(Item, Fgen, Pred)>,
}

impl<Fgen, Pred> SimpleCov<u64, Fgen, Pred>
where
    Fgen: Default + for<'a> FnOnce(&'a u64) -> u64,
    Pred: Default + for<'a> FnOnce(&'a u64) -> bool,
{
    fn new(fgen: Fgen, pred: Pred) -> Self {
        Self {
            cursor_index: 0,
            _marker: Ph,
        }
    }
}

impl<Fgen, Pred> Coverage for SimpleCov<u64, Fgen, Pred>
where
    Fgen: Default + for<'a> FnOnce(&'a u64) -> u64,
    Pred: Default + for<'a> FnOnce(&'a u64) -> bool,
{
    type Item = u64;

    const LAST_INDEX: Self::Index = u64::MAX;

    fn cursor_index(&self) -> Self::Index {
        self.cursor_index
    }

    fn cursor_jump(&mut self, idx: &Self::Index) {
        println!("cursor_jump {:#?}", idx);
        self.cursor_index = *idx;
    }

    fn raw_sample(idx: &Self::Index) -> Self::Item {
        println!("raw_sample {:#?}", idx);
        Fgen::default()(idx)
    }

    fn predicate(item: &Self::Item) -> bool {
        Pred::default()(item)
    }
}

fn main() {
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

    let mut cov = SimpleCov::new(Fgen, Pred);

    for _ in 0..10 {
        println!("trying");
        let x = cov.shallow_discover();
        println!("debug");
        println!("{:#?}", x)
    }
}

/*

it output something :)

Standard Output
trying
raw_sample 0
raw_sample 2
raw_sample 4
raw_sample 8
raw_sample 16
raw_sample 32
raw_sample 64
raw_sample 128
raw_sample 256
raw_sample 512
raw_sample 1024
raw_sample 2048
raw_sample 4096
raw_sample 8192
raw_sample 16384
raw_sample 32768
raw_sample 65536
raw_sample 131072
raw_sample 262144
raw_sample 524288
raw_sample 1048576
raw_sample 2097152
raw_sample 4194304
raw_sample 8388608
raw_sample 16777216
raw_sample 33554432
raw_sample 67108864
raw_sample 134217728
raw_sample 268435456
raw_sample 536870912
raw_sample 1073741824
raw_sample 2147483648
raw_sample 4294967296
raw_sample 8589934592
raw_sample 17179869184
raw_sample 34359738368
raw_sample 68719476736
raw_sample 137438953472
raw_sample 274877906944
raw_sample 549755813888
raw_sample 1099511627776
raw_sample 2199023255552
raw_sample 4398046511104
raw_sample 8796093022208
raw_sample 17592186044416
raw_sample 35184372088832
raw_sample 70368744177664
raw_sample 140737488355328
raw_sample 281474976710656
raw_sample 562949953421312
raw_sample 1125899906842624
raw_sample 2251799813685248
raw_sample 4503599627370496
raw_sample 9007199254740992
raw_sample 18014398509481984
raw_sample 36028797018963968
raw_sample 72057594037927936
raw_sample 144115188075855872
raw_sample 288230376151711744
raw_sample 576460752303423488
raw_sample 1152921504606846976
raw_sample 2305843009213693952
raw_sample 4611686018427387904
cursor_jump 1
debug
None
trying
raw_sample 1
raw_sample 3
raw_sample 5
raw_sample 9
raw_sample 17
raw_sample 33
raw_sample 65
raw_sample 129
raw_sample 257
cursor_jump 257
cursor_jump 258
debug
Some(
    34,
)
trying
raw_sample 258
cursor_jump 259
debug
Some(
    33,
)
trying
raw_sample 259
cursor_jump 260
debug
Some(
    32,
)
trying
raw_sample 260
raw_sample 262
raw_sample 256
raw_sample 268
raw_sample 276
raw_sample 292
cursor_jump 292
cursor_jump 293
debug
Some(
    7,
)
trying
raw_sample 293
cursor_jump 294
debug
Some(
    6,
)
trying
raw_sample 294
cursor_jump 295
debug
Some(
    5,
)
trying
raw_sample 295
cursor_jump 296
debug
Some(
    4,
)
trying
raw_sample 296
cursor_jump 297
debug
Some(
    11,
)
trying
raw_sample 297
cursor_jump 298
debug
Some(
    10,
)

*/
