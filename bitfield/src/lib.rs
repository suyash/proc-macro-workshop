// Crates that have the "proc-macro" crate type are only allowed to export
// procedural macros. So we cannot have one crate that defines procedural macros
// alongside other types of public APIs like traits and structs.
//
// For this project we are going to need a #[bitfield] macro but also a trait
// and some structs. We solve this by defining the trait and structs in this
// crate, defining the attribute macro in a separate bitfield-impl crate, and
// then re-exporting the macro from this crate so that users only have one crate
// that they need to import.
//
// From the perspective of a user of this crate, they get all the necessary APIs
// (macro, trait, struct) through the one bitfield crate.
use seq::seq;

pub use bitfield_impl::bitfield;
pub use bitfield_impl::bitfield_type;
pub use bitfield_impl::BitfieldSpecifier;

pub mod checks {
    use std::marker::PhantomData;

    pub trait CheckTotalSizeIsMultipleOfEightBits where Self::Size: TotalSizeIsMultipleOfEightBits {
        type Size;
    }

    pub trait TotalSizeIsMultipleOfEightBits {}

    impl TotalSizeIsMultipleOfEightBits for PhantomData<[(); 0]> {}

    pub trait CheckDiscriminantInRange<T> where Self::Type: DiscriminantInRange {
        type Type;
    }

    pub trait DiscriminantInRange {}

    impl DiscriminantInRange for PhantomData<[(); 1]> {}

    pub struct BitsCheck<C> {
        pub data: C
    }
}

pub trait Specifier {
    type HoldType;
    const BITS: usize;

    fn to_hold(v: u64) -> Self::HoldType;
    fn from_hold(v: Self::HoldType) -> u64;
}

impl Specifier for bool {
    type HoldType = bool;
    const BITS: usize = 1;

    fn to_hold(v: u64) -> Self::HoldType {
        v & 1 == 1
    }

    fn from_hold(v: Self::HoldType) -> u64 {
        v as u64
    }
}

seq!(N in 1..=8 {
    bitfield_type!(N, u8);
});

seq!(N in 9..=16 {
    bitfield_type!(N, u16);
});

seq!(N in 17..=32 {
    bitfield_type!(N, u32);
});

seq!(N in 33..=64 {
    bitfield_type!(N, u64);
});
