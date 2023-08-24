//! Count values processed in each iteration to measure throughput.
//!
//! # Examples
//!
//! The following example measures throughput of converting
//! [`&[i32]`](prim@slice) into [`Vec<i32>`](Vec) by providing [`Bytes`] via
//! [`Bencher::counter`](crate::Bencher::counter):
//!
//! ```
//! use divan::counter::Bytes;
//!
//! #[divan::bench]
//! fn slice_into_vec(bencher: divan::Bencher) {
//!     let ints: &[i32] = &[
//!         // ...
//!     ];
//!
//!     let bytes = Bytes::of_slice(ints);
//!
//!     bencher
//!         .counter(bytes)
//!         .bench(|| -> Vec<i32> {
//!             divan::black_box(ints).into()
//!         });
//! }
//! ```

use std::any::Any;

mod any_counter;
mod collection;
mod into_counter;
mod sealed;
mod uint;

pub(crate) use self::{
    any_counter::{AnyCounter, KnownCounterKind},
    collection::{CounterCollection, CounterSet},
    sealed::Sealed,
    uint::{CountUInt, MaxCountUInt},
};
pub use into_counter::IntoCounter;

/// Counts the number of values processed in each iteration of a benchmarked
/// function.
///
/// This is used via:
/// - [`#[divan::bench(counters = ...)]`](macro@crate::bench#counters)
/// - [`#[divan::bench_group(counters = ...)]`](macro@crate::bench_group#counters)
/// - [`Bencher::counter`](crate::Bencher::counter)
/// - [`Bencher::input_counter`](crate::Bencher::input_counter)
#[doc(alias = "throughput")]
pub trait Counter: Sized + Any + Sealed {}

/// Process N bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bytes {
    count: MaxCountUInt,
}

/// Process N [`char`s](char).
///
/// This is beneficial when comparing benchmarks between ASCII and Unicode
/// implementations, since the number of code points is a common baseline
/// reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Chars {
    count: MaxCountUInt,
}

/// Process N items.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Items {
    count: MaxCountUInt,
}

impl Sealed for Bytes {}
impl Sealed for Chars {}
impl Sealed for Items {}

impl Counter for Bytes {}
impl Counter for Chars {}
impl Counter for Items {}

impl Bytes {
    /// Count N bytes.
    #[inline]
    pub fn new<N: CountUInt>(count: N) -> Self {
        Self { count: count.into_max_uint() }
    }

    /// Counts the size of a type with [`std::mem::size_of`].
    #[inline]
    #[doc(alias = "size_of")]
    pub const fn of<T>() -> Self {
        Self { count: std::mem::size_of::<T>() as MaxCountUInt }
    }

    /// Counts the size of a value with [`std::mem::size_of_val`].
    #[inline]
    #[doc(alias = "size_of_val")]
    pub fn of_val<T: ?Sized>(val: &T) -> Self {
        // TODO: Make const, https://github.com/rust-lang/rust/issues/46571
        Self { count: std::mem::size_of_val(val) as MaxCountUInt }
    }

    /// Counts the bytes of a [`&str`].
    ///
    /// This is like [`Bytes::of_val`] with the convenience of behaving as
    /// expected for [`&String`](String) and other types that convert to
    /// [`&str`].
    ///
    /// [`&str`]: prim@str
    #[inline]
    pub fn of_str<S: ?Sized + AsRef<str>>(s: &S) -> Self {
        Self::of_val(s.as_ref())
    }

    /// Counts the bytes of a [slice](prim@slice).
    ///
    /// This is like [`Bytes::of_val`] with the convenience of behaving as
    /// expected for [`&Vec<T>`](Vec) and other types that convert to
    /// [`&[T]`](prim@slice).
    #[inline]
    pub fn of_slice<T, S: ?Sized + AsRef<[T]>>(s: &S) -> Self {
        Self::of_val(s.as_ref())
    }
}

impl Chars {
    /// Count N [`char`s](char).
    #[inline]
    pub fn new<N: CountUInt>(count: N) -> Self {
        Self { count: count.into_max_uint() }
    }

    /// Counts the [`char`s](prim@char) of a [`&str`](prim@str).
    #[inline]
    pub fn of_str<S: ?Sized + AsRef<str>>(s: &S) -> Self {
        Self::new(s.as_ref().chars().count())
    }
}

impl Items {
    /// Count N items.
    #[inline]
    pub fn new<N: CountUInt>(count: N) -> Self {
        Self { count: count.into_max_uint() }
    }
}

/// The numerical base for [`Bytes`] in benchmark outputs.
///
/// See [`Divan::bytes_format`](crate::Divan::bytes_format) for more info.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum BytesFormat {
    /// Powers of 1000, starting with KB (kilobyte). This is the default.
    #[default]
    Decimal,

    /// Powers of 1024, starting with KiB (kibibyte).
    Binary,
}

/// Private `BytesFormat` that prevents leaking trait implementations we don't
/// want to publicly commit to.
#[derive(Clone, Copy)]
pub(crate) struct PrivBytesFormat(pub BytesFormat);

impl clap::ValueEnum for PrivBytesFormat {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self(BytesFormat::Decimal), Self(BytesFormat::Binary)]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        let name = match self.0 {
            BytesFormat::Decimal => "decimal",
            BytesFormat::Binary => "binary",
        };
        Some(clap::builder::PossibleValue::new(name))
    }
}
