//! This crate provides a `Vec` wrapper (`Vec1`) which guarantees to have at least 1 element.
//!
//! This can be useful if you have a API which accepts one ore more ofe a kind.
//! Instead of accepting a `Vec` and returning an error if it's empty a `Vec1`
//! can be used assuring there is at least 1 element and through this reducing
//! the number of possible error causes.
//!
//! The crate provides an optional `serde` feature, which provides
//! implementations of `serde::Serialize`/`serde::Deserialize`.
//!
//! # Example
//!
//! ```
//! #[macro_use]
//! extern crate vec1;
//!
//! use vec1::Vec1;
//!
//! fn main() {
//!     // vec1![] makes sure at compiler time
//!     // there is at least one element
//!     //let names = vec1! [ ];
//!     let names = vec1! [ "Liz" ];
//!     greet(names);
//! }
//!
//! fn greet(names: Vec1<&str>) {
//!     // methods like first/last which return a Option on Vec do
//!     // directly return the value, we know it's possible
//!     let first = names.first();
//!     println!("hallo {}", first);
//!     for name in names.iter().skip(1) {
//!         println!("  who is also know as {}", name)
//!     }
//! }
//!
//! ```
//!
//! # Features
//!
//! - `std` (default): If disabled this crate will only use `core` and `alloc` but not `std` as dependencies.
//!                    Because of this some traits and method are not available if it is disabled.
//!
//! - `serde`: Implements `Serialize` and `Deserialize` for `Vec1`. Also implements it for
//!            `SmallVec1` if both `serde` and `smallvec-v1` features are enabled. Note that
//!            enabling both `serde` and `smallvec-v1` implements `Serialize` and `Deserialize`
//!            for `SmallVec1` but will *not* enable `smallvec/serde` and as such will not
//!            implement the `serde` traits for `smallvec::SmallVec`.
//!
//! - `smallvec-v1` : Adds support for a vec1 variation backed by the smallvec crate
//!                   version 1.x.y. (In the future there will likely be a additional `smallvec-v2`.).
//!                   Works with no_std, i.e. if the default features are disabled.
//!
//! - `smallvec-v1-write`: Enables `smallvec/write`, this requires std. As we can't tell cargo to
//!                        automatically enable `smallvec/write` if and only if `smallvec-v1` and
//!                        `std` are both enabled this needs to be an extra feature.
//!
//! - `unstable-nightly-try-from-impl` (deprecated) : Was used to enable `TryFrom`/`TryInto` implementations
//!                                                   before the traits became stable. Doesn't do anything by
//!                                                   now, but still exist for compatibility reasons.
//!
//! # Rustdoc
//!
//! To have all intra-(and inter-) doc links working properly it is
//! recommended to generate the documentation with nightly rustdoc.
//! This is _only_ for the links in the documentation, library code
//! and test should run at least on stable-2 (a two versions old stable)
//! and might work on older versions too.
//!
//! # Rust Version / Stability
//!
//! Besides intra-doc links everything else is supposed to work on a
//! two versions old stable release and everything newer, through it
//! might work on older releases.
//!
//! Features which require nightly/beta will be prefixed with `unstable-`.
//!
//! For forwards compatibility the prefixed feature will be kept even if
//! it's no longer unstable, through the code it feature gated is now also
//! either always available or behind a non-prefixed feature gate which the
//! `unstable-` prefixed feature gate enables.
//!
//! While I do try to keep `unstable-` features API stable this might not
//! always be possible so enabling a `unstable-` prefixed features does
//! exclude the stability guarantees normally expected from SemVer for
//! code related to that feature. Still no patch version change will
//! be pushed which brakes any code, even if it's `unstable-` prefixed!
//!
//! Updating dependencies follows following rules
//!
//! SemVer Dep. Update Kind | Publicly exposed dep? | Update of this Crate
//! ------------------------|-----------------------|----------------
//! patch update            | yes                   | patch (or minor)
//! minor update            | yes                   | minor
//! major update            | yes                   | won't happen, smallvec gets a second feature for v2
//! patch update            | no                    | patch (or minor)
//! minor update            | no                    | minor
//! major update            | no                    | minor
//!
//! If `smallvec` gets a major update a additional feature will be added supporting
//! both major versions of it *without* introducing a major update for this crate.
//!
//! I do my best so that I will never have to release a major version update for this crate as
//! this would lead to API incompatibilities for other crates using this crate in their public API.
#![no_std]

extern crate alloc;

#[cfg(any(feature="std", test))]
extern crate std;

#[doc(hidden)]
#[cfg(feature = "smallvec-v1")]
pub extern crate smallvec_v1_;

#[macro_use]
mod shared;


#[cfg(feature = "smallvec-v1")]
pub mod smallvec_v1;

use core::{
    iter::{DoubleEndedIterator, ExactSizeIterator, Extend, IntoIterator, Peekable},
    ops::{Bound, RangeBounds},
    result::Result as StdResult,
};

use alloc::{
    vec,
    collections::BinaryHeap,
    collections::VecDeque,
    rc::Rc,
    string::String,
};

#[cfg(feature="std")]
use std::{
    //TODO tests for io::Write and ffi::CString
    io,
    ffi::CString,
    sync::Arc,
};

#[cfg(any(feature="std", test))]
use std::{
    error::Error,
};

shared_impl! {@IMPORTS}

/// Error returned by operations which would cause `Vec1` to have a length of 0.
#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct Size0Error;

impl fmt::Display for Size0Error {
    fn fmt(&self, fter: &mut fmt::Formatter) -> fmt::Result {
        #[allow(deprecated)]
        write!(fter, "Cannot produce a Vec1 with a length of zero.")
    }
}

#[cfg(any(feature="std", test))]
impl Error for Size0Error {}

/// A macro similar to `vec!` to create a `Vec1`.
///
/// If it is called with less then 1 element a
/// compiler error is triggered (using `compile_error`
/// to make sure you know what went wrong).
#[macro_export]
macro_rules! vec1 {
    () => (
        compile_error!("Vec1 needs at least 1 element")
    );
    ($first:expr $(, $item:expr)* , ) => (
        $crate::vec1!($first $(, $item)*)
    );
    ($first:expr $(, $item:expr)* ) => ({
        #[allow(unused_mut)]
        let mut tmp = $crate::Vec1::new($first);
        $(tmp.push($item);)*
        tmp
    });
}

shared_impl! {
    base_bounds_macro = ,
    item_ty_macro = I,

    /// `std::vec::Vec` wrapper which guarantees to have at least 1 element.
    ///
    /// `Vec1<T>` dereferences to `&[T]` and `&mut [T]` as functionality
    /// exposed through this can not change the length.
    ///
    /// Methods of `Vec` which can be called without reducing the length
    /// (e.g. `capacity()`, `reserve()`) are exposed through wrappers
    /// with the same function signature.
    ///
    /// Methods of `Vec` which could reduce the length to 0
    /// are implemented with a `try_` prefix returning a `Result`.
    /// (e.g. `try_pop(&self)`, `try_truncate()`, etc.).
    ///
    /// Methods with returned `Option<T>` with `None` if the length was 0
    /// (and do not reduce the length) now return T. (e.g. `first`,
    /// `last`, `first_mut`, etc.).
    ///
    /// All stable traits and methods implemented on `Vec<T>` _should_ also
    /// be implemented on `Vec1<T>` (except if they make no sense to implement
    /// due to the len 1 guarantee). Be aware implementations may lack behind a bit,
    /// fell free to open a issue/make a PR, but please search closed and open
    /// issues for duplicates first.
    // #[derive(Debug, Clone, Eq, Hash, PartialOrd, Ord)]
    // #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    pub struct Vec1<I>(Vec<I>);
}

impl<T> IntoIterator for Vec1<T> {
    type Item = T;
    type IntoIter = vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> Vec1<T> {
    /// Tries to create a `Vec1<T>` from a `Vec<T>`.
    ///
    /// The fact that the input is returned _as error_ if it's empty,
    /// means that it doesn't work well with the `?` operator. It naming
    /// is also semantic sub-optimal as it's not a "from" but "try from"
    /// conversion. Which is why this method is now deprecated. Instead
    /// use `try_from_vec` and once `TryFrom` is stable it will be possible
    /// to use `try_from`, too.
    ///
    /// # Errors
    ///
    /// If the input is empty the input is returned _as error_.
    #[deprecated(
        since = "1.2.0",
        note = "does not work with `?` use Vec1::try_from_vec() instead"
    )]
    pub fn from_vec(vec: Vec<T>) -> StdResult<Self, Vec<T>> {
        if vec.is_empty() {
            Err(vec)
        } else {
            Ok(Vec1(vec))
        }
    }

    /// Turns this `Vec1` into a `Vec`.
    pub fn into_vec(self) -> Vec<T> {
        self.0
    }

    /// Return a reference to the underlying `Vec`.
    pub fn as_vec(&self) -> &Vec<T> {
        &self.0
    }

    /// Create a new `Vec1` by consuming `self` and mapping each element.
    ///
    /// This is useful as it keeps the knowledge that the length is >= 1,
    /// even through the old `Vec1` is consumed and turned into an iterator.
    ///
    /// # Example
    ///
    /// ```
    /// # #[macro_use]
    /// # extern crate vec1;
    /// # use vec1::Vec1;
    /// # fn main() {
    /// let data = vec1![1u8,2,3];
    ///
    /// let data = data.mapped(|x|x*2);
    /// assert_eq!(data, vec![2,4,6]);
    ///
    /// // without mapped
    /// let data = Vec1::try_from_vec(data.into_iter().map(|x|x*2).collect::<Vec<_>>()).unwrap();
    /// assert_eq!(data, vec![4,8,12]);
    /// # }
    /// ```
    pub fn mapped<F, N>(self, map_fn: F) -> Vec1<N>
    where
        F: FnMut(T) -> N,
    {
        Vec1(self.into_iter().map(map_fn).collect::<Vec<_>>())
    }

    /// Create a new `Vec1` by mapping references to the elements of `self`.
    ///
    /// The benefit to this compared to `Iterator::map` is that it's known
    /// that the length will still be at least 1 when creating the new `Vec1`.
    pub fn mapped_ref<F, N>(&self, map_fn: F) -> Vec1<N>
    where
        F: FnMut(&T) -> N,
    {
        Vec1(self.iter().map(map_fn).collect::<Vec<_>>())
    }

    /// Create a new `Vec1` by mapping mutable references to the elements of `self`.
    ///
    /// The benefit to this compared to `Iterator::map` is that it's known
    /// that the length will still be at least 1 when creating the new `Vec1`.
    pub fn mapped_mut<F, N>(&mut self, map_fn: F) -> Vec1<N>
    where
        F: FnMut(&mut T) -> N,
    {
        Vec1(self.iter_mut().map(map_fn).collect::<Vec<_>>())
    }

    /// Create a new `Vec1` by consuming `self` and mapping each element
    /// to a `Result`.
    ///
    /// This is useful as it keeps the knowledge that the length is >= 1,
    /// even through the old `Vec1` is consumed and turned into an iterator.
    ///
    /// As this method consumes self, returning an error means that this
    /// vec is dropped. I.e. this method behaves roughly like using a
    /// chain of `into_iter()`, `map`, `collect::<Result<Vec<N>,E>>` and
    /// then converting the `Vec` back to a `Vec1`.
    ///
    ///
    /// # Errors
    ///
    /// Once any call to `map_fn` returns a error that error is directly
    /// returned by this method.
    ///
    /// # Example
    ///
    /// ```
    /// # #[macro_use]
    /// # extern crate vec1;
    /// # use vec1::Vec1;
    /// # fn main() {
    /// let data = vec1![1,2,3];
    ///
    /// let data: Result<Vec1<u8>, &'static str> = data.try_mapped(|x| Err("failed"));
    /// assert_eq!(data, Err("failed"));
    /// # }
    /// ```
    pub fn try_mapped<F, N, E>(self, map_fn: F) -> Result<Vec1<N>, E>
    where
        F: FnMut(T) -> Result<N, E>,
    {
        let mut map_fn = map_fn;
        // ::collect<Result<Vec<_>>>() is uses the iterators size hint's lower bound
        // for with_capacity, which is 0 as it might fail at the first element
        let mut out = Vec::with_capacity(self.len());
        for element in self {
            out.push(map_fn(element)?);
        }
        Ok(Vec1(out))
    }

    /// Create a new `Vec1` by mapping references to the elements of `self`
    /// to `Result`s.
    ///
    /// The benefit to this compared to `Iterator::map` is that it's known
    /// that the length will still be at least 1 when creating the new `Vec1`.
    ///
    /// # Errors
    ///
    /// Once any call to `map_fn` returns a error that error is directly
    /// returned by this method.
    ///
    pub fn try_mapped_ref<F, N, E>(&self, map_fn: F) -> Result<Vec1<N>, E>
    where
        F: FnMut(&T) -> Result<N, E>,
    {
        let mut map_fn = map_fn;
        let mut out = Vec::with_capacity(self.len());
        for element in self.iter() {
            out.push(map_fn(element)?);
        }
        Ok(Vec1(out))
    }

    /// Create a new `Vec1` by mapping mutable references to the elements of
    /// `self` to `Result`s.
    ///
    /// The benefit to this compared to `Iterator::map` is that it's known
    /// that the length will still be at least 1 when creating the new `Vec1`.
    ///
    /// # Errors
    ///
    /// Once any call to `map_fn` returns a error that error is directly
    /// returned by this method.
    ///
    pub fn try_mapped_mut<F, N, E>(&mut self, map_fn: F) -> Result<Vec1<N>, E>
    where
        F: FnMut(&mut T) -> Result<N, E>,
    {
        let mut map_fn = map_fn;
        let mut out = Vec::with_capacity(self.len());
        for element in self.iter_mut() {
            out.push(map_fn(element)?);
        }
        Ok(Vec1(out))
    }

    /// Calls `split_off` on the inner vec if both resulting parts have length >= 1.
    ///
    /// # Errors
    ///
    /// If after the split any part would be empty an error is returned as the
    /// length >= 1 constraint must be uphold.
    pub fn try_split_off(&mut self, at: usize) -> Result<Vec1<T>, Size0Error> {
        if at == 0 || at >= self.len() {
            Err(Size0Error)
        } else {
            let out = self.0.split_off(at);
            Ok(Vec1(out))
        }
    }

    /// Calls `splice` on the underlying vec if it will not produce an empty vec.
    ///
    /// # Errors
    ///
    /// If range covers the whole vec and the replacement iterator doesn't yield
    /// any value an error is returned.
    ///
    /// This means that if an error is returned `next` might still have been called
    /// once on the `replace_with` iterator.
    pub fn splice<R, I>(
        &mut self,
        range: R,
        replace_with: I,
    ) -> Result<Splice<<I as IntoIterator>::IntoIter>, Size0Error>
    where
        I: IntoIterator<Item = T>,
        R: RangeBounds<usize>,
    {
        let mut replace_with = replace_with.into_iter().peekable();
        let range_covers_all = range_covers_vec1(&range, self.len());

        if range_covers_all && replace_with.peek().is_none() {
            Err(Size0Error)
        } else {
            let vec_splice = self.0.splice(range, replace_with);
            Ok(Splice { vec_splice })
        }
    }
}

impl Vec1<u8> {
    /// Works like `&[u8].to_ascii_uppercase()` but returns a `Vec1<T>` instead of a `Vec<T>`
    pub fn to_ascii_uppercase(&self) -> Vec1<u8> {
        Vec1(self.0.to_ascii_uppercase())
    }

    /// Works like `&[u8].to_ascii_lowercase()` but returns a `Vec1<T>` instead of a `Vec<T>`
    pub fn to_ascii_lowercase(&self) -> Vec1<u8> {
        Vec1(self.0.to_ascii_lowercase())
    }
}

fn range_covers_vec1(range: &impl RangeBounds<usize>, vec1_len: usize) -> bool {
    // As this is only used for vec1 we don't need the if vec_len == 0.
    // if vec_len == 0 { return true; }
    range_covers_vec_start(range) && range_covers_vec_end(range, vec1_len)
}

fn range_covers_vec_start(range: &impl RangeBounds<usize>) -> bool {
    match range.start_bound() {
        Bound::Included(idx) => *idx == 0,
        // there is no idx before 0, so if you start from a excluded index
        // you can not cover 0
        Bound::Excluded(_idx) => false,
        Bound::Unbounded => true,
    }
}

fn range_covers_vec_end(range: &impl RangeBounds<usize>, len: usize) -> bool {
    match range.end_bound() {
        Bound::Included(idx) => {
            // covers all if it goes up to the last idx which is len-1
            *idx >= len - 1
        }
        Bound::Excluded(idx) => {
            // len = max_idx + 1, so if excl_end = len it's > max_idx, so >= is correct
            *idx >= len
        }
        Bound::Unbounded => true,
    }
}

pub struct Splice<'a, I: Iterator + 'a> {
    vec_splice: vec::Splice<'a, Peekable<I>>,
}

impl<'a, I> fmt::Debug for Splice<'a, I>
where
    I: Iterator + 'a,
    vec::Splice<'a, Peekable<I>>: fmt::Debug,
{
    fn fmt(&self, fter: &mut fmt::Formatter) -> fmt::Result {
        fter.debug_tuple("Splice").field(&self.vec_splice).finish()
    }
}

impl<'a, I> Iterator for Splice<'a, I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.vec_splice.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.vec_splice.size_hint()
    }
}

impl<'a, I> ExactSizeIterator for Splice<'a, I> where I: Iterator {}

impl<'a, I> DoubleEndedIterator for Splice<'a, I>
where
    I: Iterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.vec_splice.next_back()
    }
}

impl<T> Vec1<T>
where
    T: PartialEq<T>,
{
    pub fn dedub(&mut self) {
        self.0.dedup()
    }
}

impl<A, B> PartialEq<Vec1<B>> for Vec1<A>
where
    A: PartialEq<B>,
{
    fn eq(&self, other: &Vec1<B>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a, T> Extend<&'a T> for Vec1<T>
where
    T: 'a + Copy,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = &'a T>,
    {
        self.0.extend(iter)
    }
}

impl<T> From<Vec1<T>> for Rc<[T]> {
    fn from(vec: Vec1<T>) -> Self {
        vec.0.into()
    }
}


impl<T> From<Vec1<T>> for Box<[T]> {
    fn from(vec: Vec1<T>) -> Self {
        vec.0.into()
    }
}

#[cfg(feature="std")]
impl<T> From<Vec1<T>> for Arc<[T]> {
    fn from(vec: Vec1<T>) -> Self {
        vec.0.into()
    }
}

macro_rules! wrapper_from_to_try_from {
    (impl Into + impl[$($tv:tt)*] TryFrom<$tf:ty> for Vec1<$et:ty> $($tail:tt)*) => (

        wrapper_from_to_try_from!(impl[$($tv),*] TryFrom<$tf> for Vec1<$et> $($tail)*);

        impl<$($tv)*> From<Vec1<$et>> for $tf $($tail)* {
            fn from(vec: Vec1<$et>) -> Self {
                vec.0.into()
            }
        }
    );
    (impl[$($tv:tt)*] TryFrom<$tf:ty> for Vec1<$et:ty> $($tail:tt)*) => (
        impl<$($tv)*> TryFrom<$tf> for Vec1<$et> $($tail)* {
            type Error = Size0Error;

            fn try_from(inp: $tf) -> StdResult<Self, Self::Error> {
                if inp.is_empty() {
                    Err(Size0Error)
                } else {
                    Ok(Vec1(inp.into()))
                }
            }
        }
    );
}

wrapper_from_to_try_from!(impl[T] TryFrom<BinaryHeap<T>> for Vec1<T>);
wrapper_from_to_try_from!(impl[] TryFrom<String> for Vec1<u8>);
wrapper_from_to_try_from!(impl['a] TryFrom<&'a str> for Vec1<u8>);
wrapper_from_to_try_from!(impl['a, T] TryFrom<&'a mut [T]> for Vec1<T> where T: Clone);
wrapper_from_to_try_from!(impl Into + impl[T] TryFrom<VecDeque<T>> for Vec1<T>);

#[cfg(feature="std")]
impl TryFrom<CString> for Vec1<u8> {
    type Error = Size0Error;

    /// Like `Vec`'s `From<CString>` this will treat the `'\0'` as not part of the string.
    fn try_from(string: CString) -> StdResult<Self, Self::Error> {
        if string.as_bytes().is_empty() {
            Err(Size0Error)
        } else {
            Ok(Vec1(string.into()))
        }
    }
}

#[cfg(feature="std")]
impl io::Write for Vec1<u8> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.0.write_all(buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

#[cfg(test)]
mod test {
    #![allow(non_snake_case)]

    #[macro_export]
    macro_rules! assert_ok {
        ($val:expr) => {{
            match $val {
                Ok(res) => res,
                Err(err) => panic!("expected Ok(..) got Err({:?})", err),
            }
        }};
        ($val:expr, $ctx:expr) => {{
            match $val {
                Ok(res) => res,
                Err(err) => panic!("expected Ok(..) got Err({:?}) [ctx: {:?}]", err, $ctx),
            }
        }};
    }

    mod Size0Error {
        #![allow(non_snake_case)]
        use super::super::*;
        use std::error::{Error as StdError};

        #[test]
        fn implements_std_error() {
            fn comp_check<T: StdError>() {}
            comp_check::<Size0Error>();
        }
    }

    #[test]
    fn range_covers_vec() {
        use super::range_covers_vec1;

        let len = 3;
        // common slicesa
        assert!(range_covers_vec1(&(..), len));
        assert!(range_covers_vec1(&(..3), len));
        assert!(!range_covers_vec1(&(..2), len));
        assert!(!range_covers_vec1(&(1..3), len));
        assert!(range_covers_vec1(&(0..3), len));
        assert!(range_covers_vec1(&(0..), len));
        assert!(!range_covers_vec1(&(1..), len));
        assert!(!range_covers_vec1(&(len..), len));

        // unusual slices
        assert!(!range_covers_vec1(&(..0), len));
        assert!(!range_covers_vec1(&(2..1), len));
    }

    mod Vec1 {
        use std::panic::catch_unwind;

        use super::super::*;

        // prevent a type from causing us to use the wrong type
        #[allow(unused_macros)]
        macro_rules! vec {
            ($($any:tt)*) => (
                compile_error!("typo? vec! => vec1!")
            );
        }

        #[test]
        fn new_vec1_macro() {
            let a = vec1![1u8, 10u8, 3u8];
            assert_eq!(a, &[1,10,3]);

            let a = vec1![40u8];
            assert_eq!(a, &[40]);

            //TODO comptest vec1![] => compiler error
        }

        #[test]
        fn new() {
            let a = Vec1::new(1u8);
            assert_eq!(a.len(), 1);
            assert_eq!(a.first(), &1u8);
        }

        #[test]
        fn with_capacity() {
            let a = Vec1::with_capacity(2u8, 10);
            assert_eq!(a.len(), 1);
            assert_eq!(a.first(), &2u8);
            assert_eq!(a.capacity(), 10);
        }

        #[test]
        fn capacity() {
            let a = Vec1::with_capacity(2u8, 123);
            assert_eq!(a.capacity(), 123);
        }

        #[test]
        fn reserve() {
            let mut a = Vec1::with_capacity(1u8, 1);
            assert_eq!(a.capacity(), 1);
            a.reserve(15);
            assert!(a.capacity() > 10);
        }

        #[test]
        fn reserve_exact() {
            let mut a = Vec1::with_capacity(1u8, 1);
            assert_eq!(a.capacity(), 1);
            a.reserve_exact(11);
            assert_eq!(a.capacity(), 12);
        }

        #[test]
        fn shrink_to_fit() {
            let mut a = Vec1::with_capacity(1u8, 20);
            a.push(13u8);
            a.shrink_to_fit();
            assert_eq!(a.capacity(), 2);
        }

        #[ignore = "not yet implemented"]
        #[test]
        fn into_boxed_slice() {
            let a = vec1![32u8, 12u8];
            let boxed: Box<[u8]> = a.into_boxed_slice();
            assert_eq!(&*boxed, &[32u8, 12u8]);
        }

        #[test]
        fn try_truncate() {
            let mut a = vec1![42u8, 32, 1];
            a.try_truncate(1).unwrap();
            assert_eq!(a.len(), 1);
            assert_eq!(a, &[42u8]);

            a.try_truncate(0).unwrap_err();
        }

        #[test]
        fn as_slice() {
            let a = vec1![22u8, 12, 9];
            let b: &[u8] = a.as_slice();
            assert_eq!(b, &[22u8, 12, 9]);
        }

        #[test]
        fn as_mut_slice() {
            let mut a = vec1![22u8, 12, 9];
            let b: &mut [u8] = a.as_mut_slice();
            assert_eq!(b, &mut [22u8, 12, 9]);
        }

        #[test]
        fn as_ptr() {
            let a = vec1![22u8, 12, 9];
            let a_ptr = a.as_ptr();
            let a = a.into_vec();
            let a_ptr2 = a.as_ptr();
            assert_eq!(a_ptr, a_ptr2);
        }

        #[test]
        fn as_mut_ptr() {
            let mut a = vec1![22u8, 12, 9];
            let a_ptr = a.as_mut_ptr();
            let mut a = a.into_vec();
            let a_ptr2 = a.as_mut_ptr();
            assert_eq!(a_ptr, a_ptr2);
        }

        #[test]
        fn try_swap_remove() {
            let mut a = vec1![1u8,2, 4];
            a.try_swap_remove(0).unwrap();
            assert_eq!(a, &[4u8, 2]);
            a.try_swap_remove(0).unwrap();
            assert_eq!(a, &[2u8]);
            a.try_swap_remove(0).unwrap_err();
        }

        #[test]
        fn insert() {
            // we only test that it's there as we only
            // forward to the underlying Vec so this test
            // is enough
            let mut a = vec1![9u8, 7, 3];
            a.insert(1, 22);
            assert_eq!(a, &[9u8, 22, 7, 3]);
        }

        #[test]
        fn try_remove() {
            // we only test that it's there as we only
            // forward to the underlying Vec so this test
            // is enough
            let mut a = vec1![9u8, 7, 3];
            a.try_remove(1).unwrap();
            assert_eq!(a, &[9u8, 3]);
            a.try_remove(1).unwrap();
            assert_eq!(a, &[9u8]);
            a.try_remove(0).unwrap_err();
        }

        #[should_panic]
        #[test]
        fn try_remove_still_panics_if_index_is_out_of_bounds() {
            let mut a = vec1![9u8, 7, 3];
            let _ = a.try_remove(200);
        }

        #[ignore = "not implemented, might never be implemented"]
        #[test]
        fn try_retain() {
            // let mut a = vec1![9u8, 7, 3];
            // a.try_retain()
        }

        #[test]
        fn dedup_by_key() {
            let mut a = vec1![0xA3u16, 0x10F, 0x20F];
            a.dedup_by_key(|f| *f & 0xFF);
            assert_eq!(a, &[0xA3, 0x10F]);
        }

        #[test]
        fn dedup_by() {
            let mut a = vec1![1u8, 7u8, 12u8, 10u8];
            a.dedup_by(|l,r| (*l%2==0) == (*r%2==0));
            assert_eq!(a, &[1u8, 12u8]);
        }

        #[test]
        fn push() {
            let mut a = vec1![1u8, 2, 10];
            a.push(1);
            assert_eq!(a, &[1u8, 2, 10, 1]);
        }

        #[test]
        fn try_pop() {
            let mut a = vec1![3u8, 10, 2];
            a.try_pop().unwrap();
            assert_eq!(a, &[3u8, 10]);
            a.try_pop().unwrap();
            assert_eq!(a, &[3u8]);
            a.try_pop().unwrap_err();
        }

        #[test]
        fn append() {
            let mut a = vec1![9u8, 12, 93];
            a.append(&mut std::vec![33, 12]);
            assert_eq!(a, &[9u8, 12, 93, 33, 12]);
        }

        #[ignore = "not yet implemented"]
        #[test]
        fn try_drain() {
            // let mut a = vec1![1u8, 2, 4, 4, 5];
            // let out = a.try_drain(3..).unwrap().collect::<Vec<_>>();
            // assert_eq!(a, &[1u8, 2, 4]);
            // assert_eq!(out, &[4u8, 5])
            // TODO ..2  TODO x..y TODO x..=y TODO ...
        }

        // #[test]
        // fn clear() {
        //     //TODO comptest a.clear() must not compile
        //     let mut a = vec1![1u8,2,3];
        //     a.clear();
        // }

        #[test]
        fn len() {
            let a = vec1![12u8, 4, 6, 2, 3];
            assert_eq!(a.len(), 5);
        }

        #[test]
        fn is_empty() {
            let a = vec1![12u8];
            //we don't impl. it but slice does
            assert_eq!(a.is_empty(), false);
        }

        #[test]
        fn try_split_off() {
            let mut left = vec1![88u8, 73, 12, 6];
            let mut right = left.try_split_off(1).unwrap();
            assert_eq!(left, &[88u8]);
            assert_eq!(right, &[73u8, 12, 6]);

            right.try_split_off(0).unwrap_err();
            right.try_split_off(right.len()).unwrap_err();
        }

        #[test]
        fn try_split_off_and_out_of_bounds_panic() {
            let mut a = vec1![32u8];
            //FIXME[BUG] the implementation is wrong but stabilized :(
            //It should still panic... or return a different error.
            let Size0Error = a.try_split_off(200).unwrap_err();
        }

        #[test]
        fn resize_with() {
            let mut a = vec1![1u8];
            a.try_resize_with(3, || 3u8).unwrap();
            assert_eq!(a, &[1u8, 3, 3]);
            a.try_resize_with(0, || 0u8).unwrap_err();
        }

        #[ignore = "not yet implemented"]
        #[test]
        fn leak() {
            // let mut a = vec1![1u8, 3];
            // let s: &'static mut [u8] = a.leak();
            // assert_eq!(s, &[1u8, 3]);
        }

        #[test]
        fn try_resize() {
            let mut a = vec1![1u8, 2];
            a.try_resize(4, 19).unwrap();
            assert_eq!(a, &[1u8, 2, 19, 19]);
            a.try_resize(0, 19).unwrap_err();
        }

        #[test]
        fn extend_from_slice() {
            let mut a = vec1![1u8];
            a.extend_from_slice(&[2u8, 3, 4]);
            assert_eq!(a, &[1u8, 2, 3, 4]);
        }

        #[test]
        fn dedup() {
            let mut a = vec1![1u8, 1, 2, 2];
            a.dedup();
            assert_eq!(a, &[1u8, 2]);
        }

        #[test]
        fn splice() {
            let mut a = vec1![1u8, 2, 3, 4];
            let out: Vec<u8> = a.splice(1..3, std::vec![11, 12, 13]).unwrap().collect();
            assert_eq!(a, &[1u8, 11, 12, 13, 4]);
            assert_eq!(out, &[2u8, 3]);
            let out: Vec<u8> = a.splice(2.., std::vec![7, 8]).unwrap().collect();
            assert_eq!(a, &[1u8, 11, 7, 8]);
            assert_eq!(out, &[12u8, 13, 4]);
            let out: Vec<u8> = a.splice(..2, std::vec![100, 200]).unwrap().collect();
            assert_eq!(a, &[100u8, 200, 7, 8]);
            assert_eq!(out, &[1u8, 11]);
            let out: Vec<u8> = a.splice(.., std::vec![10, 220]).unwrap().collect();
            assert_eq!(a, &[10u8, 220]);
            assert_eq!(out, &[100u8, 200, 7, 8]);
            let out: Vec<u8> = a.splice(1.., Vec::<u8>::new()).unwrap().collect();
            assert_eq!(a, &[10u8]);
            assert_eq!(out, &[220u8]);

            a.splice(.., Vec::<u8>::new()).unwrap_err();
        }

        #[ignore = "not yet renamed, deprecate splice"]
        #[test]
        fn try_splice() {
            // let mut a = vec1![1u8, 2, 3, 4];
            // let out: Vec<u8> = a.try_splice(1..3, std::vec![11, 12, 13]).unwrap().collect();
            // assert_eq!(a, &[1u8, 11, 12, 13, 4]);
            // assert_eq!(out, &[2u8, 3]);
            // let out: Vec<u8> = a.try_splice(2.., std::vec![7, 8]).unwrap().collect();
            // assert_eq!(a, &[1u8, 11, 7, 8]);
            // assert_eq!(out, &[12u8, 13, 4]);
            // let out: Vec<u8> = a.try_splice(..2, std::vec![100, 200]).unwrap().collect();
            // assert_eq!(a, &[100u8, 200, 7, 8]);
            // assert_eq!(out, &[1u8, 11]);

            // a.try_splice(.., Vec::<u8>::new()).unwrap_err();
        }

        #[test]
        fn splice_still_panics_if_out_of_bounds() {
            let res = catch_unwind(|| {
                let mut a = vec1![1u8, 2, 3, 4];
                let _ = a.splice(3..2, vec1![32u8]);
            });
            assert!(res.is_err());

            let res = catch_unwind(|| {
                let mut a = vec1![1u8, 2, 3, 4];
                let _ = a.splice(..100, vec1![32u8]);
            });
            assert!(res.is_err());
        }

        #[ignore = "not yet renamed"]
        #[test]
        fn try_splice_still_panics_if_out_of_bounds() {
            // let res = catch_unwind(|| {
            //     let mut a = vec1![1u8, 2, 3, 4];
            //     a.try_splice(3..2, vec1![32u8]);
            // });
            // assert!(res.is_err());

            // let res = catch_unwind(|| {
            //     let mut a = vec1![1u8, 2, 3, 4];
            //     a.try_splice(..100, vec1![32u8]);
            // });
            // assert!(res.is_err());
        }

        #[test]
        fn first() {
            let a = vec1![12u8, 13];
            assert_eq!(a.first(), &12u8);
        }

        #[test]
        fn first_mut() {
            let mut a = vec1![12u8, 13];
            assert_eq!(a.first_mut(), &mut 12u8);
        }

        #[test]
        fn last() {
            let a = vec1![12u8, 13];
            assert_eq!(a.last(), &13u8);
        }

        #[test]
        fn last_mut() {
            let mut a = vec1![12u8, 13];
            assert_eq!(a.last_mut(), &mut 13u8);
        }

        mod AsMut {
            use crate::*;

            #[test]
            fn of_slice() {
                let mut a = vec1![33u8, 123];
                let s: &mut [u8] = a.as_mut();
                assert_eq!(s, &mut [33u8, 123]);
            }

            #[test]
            fn of_self() {
                //TODO check smallvec
                let mut a = vec1![33u8, 123];
                let v: &mut Vec1<u8> = a.as_mut();
                assert_eq!(v, &mut vec1![33u8, 123]);
            }

            //TODO comptest AsMut of Vec must not compile
        }

        mod AsRef {
            use crate::*;

            #[test]
            fn of_slice() {
                let a = vec1![32u8, 103];
                let s: &[u8] = a.as_ref();
                assert_eq!(s, &[32u8, 103]);
            }

            #[test]
            fn of_vec() {
                let a = vec1![33u8];
                let v: &Vec<u8> = a.as_ref();
                assert_eq!(v, &std::vec![33u8]);
            }

            #[test]
            fn of_self() {
                let a = vec1![211u8];
                let v: &Vec1<u8> = a.as_ref();
                assert_eq!(v, &vec1![211u8]);
            }
        }

        mod Borrow {
            use crate::*;

            #[test]
            fn of_slice() {
                let a = vec1![32u8, 103];
                let s: &[u8] = a.borrow();
                assert_eq!(s, &[32u8, 103]);
            }

            #[test]
            fn of_vec() {
                let a = vec1![33u8];
                let v: &Vec<u8> = a.borrow();
                assert_eq!(v, &std::vec![33u8]);
            }
        }

        mod BorrowMut {
            use crate::*;

            #[test]
            fn of_slice() {
                let mut a = vec1![32u8, 103];
                let s: &mut [u8] = a.borrow_mut();
                assert_eq!(s, &mut [32u8, 103]);
            }

            #[ignore = "not yet implemented"]
            #[test]
            fn of_vec() {
                // let a = vec1![33u8];
                // let v: &mut Vec<u8> = a.borrow_mut();
                // assert_eq!(v, &mut std::vec![33u8]);
            }
        }

        mod Clone {
            #[test]
            fn clone() {
                let a = vec1![41u8, 12, 33];
                let b = a.clone();
                assert_eq!(a, b);
            }
        }

        mod Debug {
            #[test]
            fn fmt() {
                let a = vec1![2u8, 3, 1];
                assert_eq!(std::format!("{:?}", a), "[2, 3, 1]");
            }
        }

        mod Default {
            use crate::*;

            #[test]
            fn default() {
                let a = Vec1::<u8>::default();
                assert_eq!(a, &[0u8]);
            }
        }

        mod Deref {
            use crate::*;

            #[test]
            fn deref() {
                let a = vec1![99, 73];
                let d: &[u8] = <Vec1<u8> as Deref>::deref(&a);
                assert_eq!(d, &[99, 73]);
            }
        }

        mod DerefMut {
            use crate::*;

            #[test]
            fn deref() {
                let mut a = vec1![99, 73];
                let d: &mut [u8] = <Vec1<u8> as DerefMut>::deref_mut(&mut a);
                assert_eq!(d, &mut [99, 73]);
            }
        }

        mod Eq {
            use crate::*;

            #[test]
            fn eq() {
                let a = vec1![41u8, 12, 33];
                let b = a.clone();
                assert_eq!(a, b);

                fn impls_eq<A: Eq>(){}
                impls_eq::<Vec1<u8>>();
            }
        }

        mod Extend {
            use std::borrow::ToOwned;

            #[test]
            fn by_value_ref() {
                let mut a = vec1![0];
                a.extend(vec1![33u8].iter());
                assert_eq!(a, &[0, 33]);
            }

            #[test]
            fn by_value() {
                let mut a = vec1!["hy".to_owned()];
                a.extend(vec1!["ho".to_owned()].into_iter());
                assert_eq!(a, &["hy".to_owned(), "ho".to_owned()]);
            }
        }

        mod TryFrom_ {
            use std::{borrow::ToOwned, convert::TryFrom};
            use crate::*;

            #[test]
            fn from_slice_ref() {
                let slice: &[String] = &["hy".to_owned()];
                let vec = Vec1::try_from(slice).unwrap();
                assert_eq!(vec, slice);

                let slice: &[String] = &[];
                Vec1::try_from(slice).unwrap_err();
            }

            #[test]
            fn from_slice_mut() {
                let slice: &mut [String] = &mut ["hy".to_owned()];
                let vec = Vec1::try_from(&mut *slice).unwrap();
                assert_eq!(vec, slice);

                let slice: &mut [String] = &mut [];
                Vec1::try_from(slice).unwrap_err();
            }

            #[test]
            fn from_str() {
                let vec = Vec1::<u8>::try_from("hy").unwrap();
                //TODO remove all unnecessary
                assert_eq!(vec, "hy".as_bytes());

                Vec1::<u8>::try_from("").unwrap_err();
            }

            #[ignore = "not yet implemented"]
            #[test]
            fn from_array() {
                // we just test if there is a impl for a arbitrary len
                // which here is good enough but far from complete coverage!

                // let array = [11u8; 100];
                // let vec = Vec1::try_from(array).unwrap();
                // assert_eq!(vec.iter().sum(), 110);

                // Vec1::try_from([0u8;0]).unwrap_err()
            }

            #[test]
            fn from_binary_heap() {
                use std::collections::BinaryHeap;
                let mut heap = BinaryHeap::new();
                heap.push(1u8);
                heap.push(100);
                heap.push(3);

                let vec = Vec1::try_from(heap).unwrap();
                assert_eq!(vec.len(), 3);
                assert_eq!(vec.first(), &100);
                assert!(vec.contains(&3));
                assert!(vec.contains(&1));

                Vec1::<u8>::try_from(BinaryHeap::new()).unwrap_err();
            }

            #[test]
            fn from_boxed_slice() {
                let boxed = Box::new([20u8; 10]) as Box<[u8]>;
                let vec = Vec1::try_from(boxed).unwrap();
                assert_eq!(vec, &[20u8; 10]);
            }

            #[cfg(feature="std")]
            #[test]
            fn from_cstring() {
                let cstring = CString::new("ABA").unwrap();
                let vec = Vec1::<u8>::try_from(cstring).unwrap();
                assert_eq!(vec, &[65, 66, 65]);

                let cstring = CString::new("").unwrap();
                Vec1::<u8>::try_from(cstring).unwrap_err();
            }

            #[ignore = "not yet implemented"]
            #[test]
            fn from_cow() {
                // let slice: &[u8] = &[12u8, 33];
                // let cow = Cow::Borrowed(slice);
                // let vec = Vec1::try_from(cow).unwrap();
                // assert_eq!(vec, slice);

                // let slice: &[u8] = &[];
                // let cow = Cow::Borrowed(slice);
                // Vec1::try_from(cow).unwrap_err();
            }

            #[test]
            fn from_string() {
                let vec = Vec1::<u8>::try_from("ABA".to_owned()).unwrap();
                assert_eq!(vec, &[65, 66, 65]);

                Vec1::<u8>::try_from("".to_owned()).unwrap_err();
            }

            #[test]
            fn from_vec_deque() {
                let queue = VecDeque::from(std::vec![1u8, 2, 3]);
                let vec = Vec1::try_from(queue).unwrap();
                assert_eq!(vec, &[1u8, 2, 3]);

                Vec1::<u8>::try_from(VecDeque::new()).unwrap_err();
            }

        }

        mod Hash {
            use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}};
            use crate::*;

            #[test]
            fn hash() {
                let a = vec1![1u8, 10, 33, 12];
                let mut hasher = DefaultHasher::new();
                a.hash(&mut hasher);
                let a_state = hasher.finish();

                let b = a.into_vec();
                let mut hasher = DefaultHasher::new();
                b.hash(&mut hasher);
                let b_state = hasher.finish();

                assert_eq!(a_state, b_state);
            }

            #[test]
            fn hash_slice() {
                let a: &[_] = &[vec1![1u8, 10, 33, 12], vec1![22, 12]];
                let mut hasher = DefaultHasher::new();
                <Vec1<u8> as Hash>::hash_slice(a, &mut hasher);
                let a_state = hasher.finish();

                let b: &[_] = &[std::vec![1u8, 10, 33, 12], std::vec![22, 12]];
                let mut hasher = DefaultHasher::new();
                <Vec<u8> as Hash>::hash_slice(b, &mut hasher);
                let b_state = hasher.finish();

                assert_eq!(a_state, b_state);
            }
        }

        mod Index {
            use std::ops::Index;

            #[test]
            fn index() {
                let vec = vec1![34u8, 99, 10, 73];
                assert_eq!(vec.index(1..3), &[99, 10]);
                assert_eq!(&vec[1..3], &[99, 10]);
                assert_eq!(vec[0], 34u8);
            }
        }

        mod IndexMut {
            use std::ops::IndexMut;

            #[test]
            fn index_mut() {
                let mut vec = vec1![34u8, 99, 10, 73];
                assert_eq!(vec.index_mut(1..3), &mut [99, 10]);
                assert_eq!(&mut vec[1..3], &mut [99, 10]);
            }
        }

        mod IntoIterator {
            #[test]
            fn of_self() {
                let vec = vec1![1u8, 33u8, 57];
                let mut iter = vec.into_iter();
                assert_eq!(iter.size_hint(), (3, Some(3)));
                // impl. ExactSizedIterator
                assert_eq!(iter.len(), 3);
                assert_eq!(iter.next(), Some(1));
                // impl. DoubleEndedIterator
                assert_eq!(iter.next_back(), Some(57));
                assert_eq!(iter.next(), Some(33));
                assert_eq!(iter.next(), None);
            }

            #[test]
            fn of_self_ref() {
                let vec = vec1![1u8, 33u8, 57];
                let mut iter = (&vec).into_iter();
                assert_eq!(iter.size_hint(), (3, Some(3)));
                // impl. ExactSizedIterator
                assert_eq!(iter.len(), 3);
                assert_eq!(iter.next(), Some(&1));
                // impl. DoubleEndedIterator
                assert_eq!(iter.next_back(), Some(&57));
                assert_eq!(iter.next(), Some(&33));
                assert_eq!(iter.next(), None);
            }

            #[test]
            fn of_self_mut() {
                let mut vec = vec1![1u8, 33u8, 57];
                let mut iter = (&mut vec).into_iter();
                assert_eq!(iter.size_hint(), (3, Some(3)));
                // impl. ExactSizedIterator
                assert_eq!(iter.len(), 3);
                assert_eq!(iter.next(), Some(&mut 1));
                // impl. DoubleEndedIterator
                assert_eq!(iter.next_back(), Some(&mut 57));
                assert_eq!(iter.next(), Some(&mut 33));
                assert_eq!(iter.next(), None);
            }
        }

        mod  Ord {
            use std::cmp::Ordering;

            #[test]
            fn cmp() {
                // just make sure we implemented it
                // we will forward to Vec's impl. anyway
                // so no reasone to test if cmp works correctly
                // (it doing so is desired sue proptest!).
                let a = vec1![1u8, 3, 4];
                let b = vec1![1u8, 4, 2];
                assert_eq!(a.cmp(&b), Ordering::Less);
            }
        }

        mod PartialEq {
            use std::borrow::ToOwned;

            #[test]
            fn to_array_ref() {
                let vec = vec1![67u8, 73, 12];
                let array: &[u8; 3] = &[67, 73, 12];
                let array2: &[u8; 3] = &[67, 73, 33];
                assert_eq!(vec.eq(&array), true);
                assert_eq!(vec.eq(&array2), false);
            }

            #[test]
            fn to_slice_ref() {
                let vec = vec1![67u8, 73, 12];
                let array: &[u8] = &[67, 73, 12];
                let array2: &[u8] = &[67, 73, 33];
                assert_eq!(vec.eq(&array), true);
                assert_eq!(vec.eq(&array2), false);
            }

            #[test]
            fn to_slice_mut() {
                let vec = vec1![67u8, 73, 12];
                let array: &mut [u8] =  &mut [67, 73, 12];
                let array2: &mut [u8] = &mut [67, 73, 33];
                assert_eq!(vec.eq(&array), true);
                assert_eq!(vec.eq(&array2), false);
            }

            #[test]
            fn to_array() {
                let vec = vec1![67u8, 73, 12];
                let array: [u8; 3] = [67, 73, 12];
                let array2: [u8; 3] = [67, 73, 33];
                assert_eq!(vec.eq(&array), true);
                assert_eq!(vec.eq(&array2), false);
            }

            #[ignore = "not yet implemented?"]
            #[test]
            fn to_slice() {
                // let vec = vec1![67u8, 73, 12];
                // let array: &[u8] = &[67, 73, 12];
                // let array2: &[u8] = &[67, 73, 33];
                // assert_eq!(vec.eq(array), true);
                // assert_eq!(vec.eq(array2), false);
            }

            #[test]
            fn to_self_kind() {
                let a = vec1!["hy".to_owned()];
                let b = vec1!["hy"];
                assert_eq!(a, b);
            }
        }

        mod PartialOrd {
            use std::cmp::Ordering;


            #[test]
            fn with_self_kind() {
                let a = vec1!["b"];
                let b = vec1!["a"];
                assert_eq!(a.partial_cmp(&b), Some(Ordering::Greater));
            }
        }

        #[cfg(feature="std")]
        mod Write {
            use std::io::Write;

            #[test]
            fn for_bytes() {
                let mut v = vec1![1u8];
                v.write(&[65, 100, 12]).unwrap();
                assert_eq!(v, &[1u8, 65, 100, 12]);
            }
        }

        #[cfg(feature = "serde")]
        mod serde {
            use crate::*;

            #[test]
            fn empty() {
                let result: Result<Vec1<u8>, _> = serde_json::from_str("[]");
                assert!(result.is_err());
            }

            #[test]
            fn one_element() {
                let vec: Vec1<u8> = serde_json::from_str("[1]").unwrap();
                assert_eq!(vec, vec1![1]);
                let json = serde_json::to_string(&vec).unwrap();
                assert_eq!(json, "[1]");
            }

            #[test]
            fn multiple_elements() {
                let vec: Vec1<u8> = serde_json::from_str("[1, 2, 3]").unwrap();
                assert_eq!(vec, vec1![1, 2, 3]);
                let json = serde_json::to_string(&vec).unwrap();
                assert_eq!(json, "[1,2,3]");
            }
        }
    }

    mod Cow {

        mod From {
            // use std::borrow::{Cow, ToOwned};
            // use crate::*;

            #[ignore = "not yet implemented"]
            #[test]
            fn from_vec1() {
                // let vec = vec1!["ho".to_owned()];
                // match Cow::<'_, [String]>::from(&vec) {
                //     Cow::Borrowed(vec_ref) => assert_eq!(&vec, vec_ref),
                //     Cow::Owned(_) => panic!("unexpected conversion") ,
                // }
            }

            //FIXME wait two times Cow<'a, [T]> from vec1 ??
        }

        mod PartialEq {
            // use std::borrow::Cow;

            #[ignore = "not yet implemented"]
            #[test]
            fn to_vec1() {
                // let cow: Cow<'_, [u8]> = Cow::Borrowed(&[1u8, 3, 4]);
                // assert_eq!(cow.eq(&vec1![1u8, 3, 4]), true);
                // assert_eq!(cow.eq(&vec1![2u8, 3, 4]), false);
            }
        }
    }

    mod CString {
        mod From {
            // use std::{ffi::CString, num::NonZeroU8};

            #[ignore = "not yet implemented"]
            #[test]
            fn from_vec1_non_zero_u8() {
                // let vec = vec1![NonZeroU8::new(67).unwrap()];
                // let cstring = CString::from(vec);
                // assert_eq!(cstring, CString::new("C").unwrap());
            }
        }
    }

    mod BoxedSlice {

        mod From {
            use std::boxed::Box;

            #[test]
            fn from_vec1() {
                let boxed = Box::<[u8]>::from(vec1![99u8, 23, 4]);
                assert_eq!(&*boxed, &[99u8, 23, 4]);
            }
        }
    }

    mod BinaryHeap {
        mod From {
            // use std::collections::BinaryHeap;

            #[ignore = "not yet implemented"]
            #[test]
            fn from_vec1() {
                // let vec = vec1![1u8, 99, 23];
                // let heap = BinaryHeap::from(vec);
                // assert_eq!(heap.pop(), Some(99));
                // assert_eq!(heap.pop(), Some(23));
                // assert_eq!(heap.pop(), Some(1));
                // assert_eq!(heap.pop(), None);
            }
        }
    }

    mod Rc {
        mod From {
            use std::rc::Rc;

            #[test]
            fn from_vec1() {
                let rced = Rc::<[u8]>::from(vec1![8u8, 7, 33]);
                assert_eq!(&*rced, &[8u8, 7, 33]);
            }
        }
    }

    #[cfg(feature="std")]
    mod Arc {
        mod From {
            use std::sync::Arc;

            #[test]
            fn from_vec1() {
                let arced = Arc::<[u8]>::from(vec1![8u8, 7, 33]);
                assert_eq!(&*arced, &[8u8, 7, 33]);
            }
        }
    }

    mod VecDeque {

        mod From {
            use alloc::collections::VecDeque;

            #[test]
            fn from_vec1() {
                let queue = VecDeque::from(vec1![32u8, 2, 10]);
                assert_eq!(queue, &[32, 2, 10]);
            }
        }

        mod PartialEq {
            // use alloc::collections::VecDeque;

            #[ignore = "not yet implemented"]
            #[test]
            fn to_vec1() {
                // let queue = VecDeque::from(vec1![1u8, 2]);

                // assert!(queue.eq(&vec1![1u8, 2]), true);
                // assert!(queue.eq(&vec1![1u8, 3]), false);
            }
        }
    }

    mod slice {

        mod PartialEq {

            #[ignore = "not yet implemented"]
            #[test]
            fn slice_mut_to_vec1() {
                // let slice = &mut [77u8];
                // assert_eq!(slice.eq(&vec1![77u8]), true);
                // assert_eq!(slice.eq(&vec1![0u8]), false);
            }

            #[test]
            fn slice_to_vec1() {
                // let slice = &[77u8];
                // assert_eq!(<[_] as Eq>::eq(slice, &vec1![77u8]), true);
                // assert_eq!(<[_] as Eq>::eq(slice, &vec1![1u8]), false);
            }

            #[test]
            fn slice_ref_to_vec1() {
                // let slice = &[77u8];
                // assert_eq!(<&[_] as Eq>::eq(&slice, &vec1![77u8]), true);
                // assert_eq!(<&[_] as Eq>::eq(&slice, &vec1![0u8]), false);
            }
        }
    }

    mod array {

        mod TryFrom {

            #[ignore = "not yet implemented"]
            #[test]
            fn from_vec1() {
                // let v = vec1![1u8, 10, 23];

                // let a = <[u8; 3]>::try_from(v).unwrap();
                // <[u8; 3]>::try_from(vec1![1u8, 2]).unwrap_err();
            }
        }
    }
}
