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

pub mod checks {
    use std::marker::PhantomData;

    pub trait CheckTotalSizeIsMultipleOfEightBits where Self::Size: TotalSizeIsMultipleOfEightBits {
        type Size;
    }

    pub trait TotalSizeIsMultipleOfEightBits {}

    impl TotalSizeIsMultipleOfEightBits for PhantomData<[(); 0]> {}
}

pub trait Specifier {
    const BITS: usize;
}

seq!(N in 1..=64 {
    bitfield_type!(N);
});
