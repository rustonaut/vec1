//! A alternative `Vec1` implementation backed by an `SmallVec1`.
//!
//! # Construction Macro
//!
//! A macro similar to `vec!` or `vec1!` does exist and is
//! re-exported in this module as `smallvec1`.
//!
//! Due to limitations in rust we can't properly document it
//! directly without either giving it strange names or ending
//! up with name collisions once we support smallvec v2 in the
//! future (without introducing a braking change).
//!
//! ## Example
//!
//! ```rust
//! use vec1::smallvec_v1::{smallvec1, SmallVec1};
//! let v: SmallVec1<[u8; 4]> = smallvec1![1u8, 2];
//! assert_eq!(&*v, &*vec![1u8,2]);
//! ```

use std::{
    fmt::{self, Debug},
    ops::{Deref, DerefMut, Index, IndexMut},
    cmp::{Ord, Ordering, PartialEq, Eq},
    hash::{Hash, Hasher},
    convert::{TryFrom, TryInto},
    borrow::{Borrow, BorrowMut},
    slice::SliceIndex
};
use super::Size0Error;

use smallvec_v1_ as smallvec;
use smallvec::*;

pub use crate::__smallvec1_macro_v1 as smallvec1;

type Result<T> = std::result::Result<T, Size0Error>;

#[doc(hidden)]
#[macro_export]
macro_rules! __smallvec1_macro_v1 {
    () => (
        compile_error!("SmallVec1 needs at least 1 element")
    );
    ($first:expr $(, $item:expr)* , ) => (
        $crate::smallvec1!($first $(, $item)*)
    );
    ($first:expr $(, $item:expr)* ) => ({
        let smallvec = $crate::smallvec_v1_::smallvec!($first $(, $item)*);
        SmallVec1::try_from_smallvec(smallvec).unwrap()
    });
}

/// `smallvec::SmallVec` wrapper which guarantees to have at least 1 element.
///
/// `SmallVec1<T>` dereferences to `&[T]` and `&mut [T]` as functionality
/// exposed through this can not change the length.
///
/// Methods of `SmallVec` which can be called without reducing the length
/// (e.g. `capacity()`, `reserve()`) are exposed through wrappers
/// with the same function signature.
///
/// Methods of `SmallVec` which could reduce the length to 0
/// are implemented with a `try_` prefix returning a `Result`.
/// (e.g. `try_pop(&self)`, `try_truncate()`, etc.).
///
/// Methods with returned `Option<T>` with `None` if the length was 0
/// (and do not reduce the length) now return T. (e.g. `first`,
/// `last`, `first_mut`, etc.).
///
/// All stable traits and methods implemented on `SmallVec<T>` _should_ also
/// be implemented on `SmallVec1<T>` (except if they make no sense to implement
/// due to the len 1 guarantee). Be aware implementations may lack behind a bit,
/// fell free to open a issue/make a PR, but please search closed and open
/// issues for duplicates first.
pub struct SmallVec1<A>(SmallVec<A>)
where
    A: smallvec::Array;


impl<A> SmallVec1<A>
where
    A: Array
{
    /// Creates a new `SmallVec1` instance containing a single element.
    ///
    /// This is roughly `SmallVec1(smallvec![first])`.
    pub fn new(first: A::Item) -> Self {
        SmallVec1(smallvec::smallvec![first])
    }

    /// Creates a new `SmallVec1` with a given capacity and a given "first" element.
    ///
    /// Note that the minimal capacity is that of the inline array. Using a smaller
    /// capacity will still lead to the capacity of the inline array. This is a property
    /// of the underlying `SmallVec`.
    pub fn with_capacity(first: A::Item, capacity: usize) -> Self {
        let mut vec = SmallVec::with_capacity(capacity);
        vec.push(first);
        SmallVec1(vec)
    }

    /// Tries to create a `SmallVec1<[T; _]>` from a normal `Vec<T>`.
    ///
    /// The size of the buffer is inferred, which means you will likely
    /// need type annotations when calling this method;
    ///
    /// # Errors
    ///
    /// This will fail if the input `Vec<T>` is empty.
    /// The returned error is a `Size0Error` instance, as
    /// such this means the _input vector will be dropped if
    /// it's empty_. But this is normally fine as it only
    /// happens if the `Vec<T>` is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use vec1::smallvec_v1::{smallvec1, SmallVec1};
    /// let sv1 = SmallVec1::<[u8; 4]>::try_from_vec(vec![3, 2, 4]);
    /// let b: SmallVec1<[u8; 4]> = smallvec1![3u8, 2, 4];
    /// assert_eq!(sv1, Ok(b));
    /// ```
    pub fn try_from_vec(vec: Vec<A::Item>) -> Result<Self> {
        if vec.is_empty() {
            Err(Size0Error)
        } else {
            Ok(SmallVec1(SmallVec::from_vec(vec)))
        }
    }

    /// Tries to create a `SmallVec1<A>` from a normal `SmallVec<A>`.
    ///
    /// # Errors
    ///
    /// This will fail if the input `Vec<T>` is empty.
    /// The returned error is a `Size0Error` instance, as
    /// such this means the _input vector will be dropped if
    /// it's empty_. But this is normally fine as it only
    /// happens if the `Vec<T>` is empty.
    ///
    pub fn try_from_smallvec(smallvec: SmallVec<A>) -> Result<Self> {
        if smallvec.is_empty() {
            Err(Size0Error)
        } else {
            Ok(SmallVec1(smallvec))
        }
    }

    /// See [`SmallVec::from_buf()`] but fails if the `buf` is empty.
    pub fn try_from_buf(buf: A) -> Result<Self> {
        Self::try_from_smallvec(SmallVec::from_buf(buf))
    }

    /// See [`SmallVec::from_buf_and_len()`] but fails if the buf and len are empty.
    ///
    /// # Panic
    ///
    /// Like [`SmallVec::from_buf_and_len()`] this fails if the length is > the
    /// size of the buffer. I.e. `SmallVec1::try_from_buf_and_len([] as [u8;0],2)` will
    /// panic.
    pub fn try_from_buf_and_len(buf: A, len: usize) -> Result<Self> {
        Self::try_from_smallvec(SmallVec::from_buf_and_len(buf, len))
    }

    /// Converts this instance into the underlying [`SmallVec<A>`] instance.
    pub fn into_smallvec(self) -> SmallVec<A> {
        self.0
    }

    /// Converts this instance into a [`Vec<A::Item>`] instance.
    pub fn into_vec(self) -> Vec<A::Item> {
        self.0.into_vec()
    }

    /// Converts this instance into the underlying buffer/array.
    ///
    /// This fails if the `SmallVec1` has not the exact length of
    /// the underlying buffers/arrays capacity.
    ///
    /// This matches [`SmallVec::into_inner()`] in that if the
    //  length is to large or small self is returned as error.
    pub fn into_inner(self) -> std::result::Result<A, Self> {
        self.0.into_inner().map_err(SmallVec1)
    }

    /// Forwards to [`SmallVec::into_boxed_slice()`].
    pub fn into_boxed_slice(self) -> Box<[A::Item]> {
        self.0.into_boxed_slice()
    }

    /// Returns a reference to the last element.
    ///
    /// As `SmallVec1` always contains at least one element there is always a last element.
    pub fn last(&self) -> &A::Item {
        //UNWRAP_SAFE: len is at least 1
        self.0.last().unwrap()
    }

    /// Returns a mutable reference to the last element.
    ///
    /// As `SmallVec1` always contains at least one element there is always a last element.
    pub fn last_mut(&mut self) -> &mut A::Item {
        //UNWRAP_SAFE: len is at least 1
        self.0.last_mut().unwrap()
    }

    /// Returns a reference to the first element.
    ///
    /// As `SmallVec1` always contains at least one element there is always a first element.
    pub fn first(&self) -> &A::Item {
        //UNWRAP_SAFE: len is at least 1
        self.0.first().unwrap()
    }

    /// Returns a mutable reference to the first element.
    ///
    /// As `SmallVec1` always contains at least one element there is always a first element.
    pub fn first_mut(&mut self) -> &mut A::Item {
        //UNWRAP_SAFE: len is at least 1
        self.0.first_mut().unwrap()
    }

    /// Return a reference to the underlying `SmallVec`.
    pub fn as_smallvec(&self) -> &SmallVec<A> {
        &self.0
    }

    /// Truncates the `SmalVec1` to given length.
    ///
    /// # Errors
    ///
    /// If len is 0 an error is returned as the
    /// length >= 1 constraint must be uphold.
    ///
    pub fn try_truncate(&mut self, len: usize) -> Result<()> {
        if len > 0 {
            self.0.truncate(len);
            Ok(())
        } else {
            Err(Size0Error)
        }
    }

    /// Calls `swap_remove` on the inner smallvec if length >= 2.
    ///
    /// # Errors
    ///
    /// If len is 1 an error is returned as the
    /// length >= 1 constraint must be uphold.
    pub fn try_swap_remove(&mut self, index: usize) -> Result<A::Item> {
        if self.len() > 1 {
            Ok(self.0.swap_remove(index))
        } else {
            Err(Size0Error)
        }
    }

    /// Calls `remove` on the inner smallvec if length >= 2.
    ///
    /// # Errors
    ///
    /// If len is 1 an error is returned as the
    /// length >= 1 constraint must be uphold.
    pub fn try_remove(&mut self, index: usize) -> Result<A::Item> {
        if self.len() > 1 {
            Ok(self.0.remove(index))
        } else {
            Err(Size0Error)
        }
    }

    /// See [`SmallVec::insert_many()`].
    pub fn insert_many<I: IntoIterator<Item = A::Item>>(
        &mut self,
        index: usize,
        iterable: I
    ) {
        self.0.insert_many(index, iterable)
    }

    /// Calls `dedup_by_key` on the inner smallvec.
    ///
    /// While this can remove elements it will
    /// never produce a empty vector from an non
    /// empty vector.
    pub fn dedup_by_key<F, K>(&mut self, key: F)
    where
        F: FnMut(&mut A::Item) -> K,
        K: PartialEq<K>,
    {
        self.0.dedup_by_key(key)
    }

    /// Calls `dedup_by_key` on the inner smallvec.
    ///
    /// While this can remove elements it will
    /// never produce a empty vector from an non
    /// empty vector.
    pub fn dedup_by<F>(&mut self, same_bucket: F)
    where
        F: FnMut(&mut A::Item, &mut A::Item) -> bool,
    {
        self.0.dedup_by(same_bucket)
    }

    /// Tries to remove the last element from this `SmallVec1`.
    ///
    /// Returns an error if the length is currently 1 (so the `try_pop` would reduce
    /// the length to 0).
    ///
    /// # Errors
    ///
    /// If len is 1 an error is returned as the
    /// length >= 1 constraint must be uphold.
    pub fn try_pop(&mut self) -> Result<A::Item> {
        if self.len() > 1 {
            //UNWRAP_SAFE: pop on len > 1 can not be none
            Ok(self.0.pop().unwrap())
        } else {
            Err(Size0Error)
        }
    }


    /// See [`SmallVec::resize_with()`] but fails if it would resize to length 0.
    pub fn try_resize_with<F>(&mut self, new_len: usize, f: F) -> Result<()>
    where
        F: FnMut() -> A::Item
    {
        if new_len > 0 {
            self.0.resize_with(new_len, f);
            Ok(())
        } else {
            Err(Size0Error)
        }
    }

    /// Splits off the first element of this vector and returns it together with the rest of the
    /// vector.
    ///
    /// # Examples
    ///
    /// ```
    /// # use vec1::smallvec_v1::{smallvec1, SmallVec1};
    /// # use vec1::smallvec_v1_::{smallvec, SmallVec};
    /// let v: SmallVec1<[u8; 4]> = smallvec1![32u8];
    /// assert_eq!((32, SmallVec::new()), v.split_off_first());
    ///
    /// let v: SmallVec1<[u8; 4]> = smallvec1![0, 1, 2, 3];
    /// assert_eq!((0, smallvec![1, 2, 3]), v.split_off_first());
    /// ```
    pub fn split_off_first(self) -> (A::Item, SmallVec<A>) {
        let mut smallvec = self.0;
        let first = smallvec.remove(0);
        (first, smallvec)
    }

    /// Splits off the last element of this vector and returns it together with the rest of the
    /// vector.
    pub fn split_off_last(self) -> (SmallVec<A>, A::Item) {
        let mut smallvec = self.0;
        let last = smallvec.remove(smallvec.len() - 1);
        (smallvec, last)
    }
}


macro_rules! impl_wrapper {
    (pub $A:ident>
        $(fn $name:ident(&$($m:ident)* $(, $param:ident: $tp:ty)*) -> $rt:ty);*) => (
            impl<$A> SmallVec1<$A>
            where
                $A: Array
            {$(
                #[inline]
                pub fn $name(self: impl_wrapper!{__PRIV_SELF &$($m)*} $(, $param: $tp)*) -> $rt {
                    (self.0).$name($($param),*)
                }
            )*}
    );
    (__PRIV_SELF &mut self) => (&mut Self);
    (__PRIV_SELF &self) => (&Self);
}

// methods in Vec not in &[] which can be directly exposed
impl_wrapper! {
    pub A>
        fn append(&mut self, other: &mut SmallVec<A>) -> ();
        fn reserve(&mut self, additional: usize) -> ();
        fn reserve_exact(&mut self, additional: usize) -> ();
        fn try_reserve(&mut self, additional: usize) -> std::result::Result<(), CollectionAllocErr>;
        fn try_reserve_exact(&mut self, additional: usize) -> std::result::Result<(), CollectionAllocErr>;
        fn shrink_to_fit(&mut self) -> ();
        fn as_mut_slice(&mut self) -> &mut [A::Item];
        fn push(&mut self, value: A::Item) -> ();
        fn insert(&mut self, idx: usize, val: A::Item) -> ();
        fn len(&self) -> usize;
        fn inline_size(&self) -> usize;
        fn spilled(&self) -> bool;
        fn capacity(&self) -> usize;
        fn as_slice(&self) -> &[A::Item];
        fn grow(&mut self, len: usize) -> ();
        fn try_grow(&mut self, len: usize) -> std::result::Result<(), CollectionAllocErr>
}

impl<A> SmallVec1<A>
where
    A: Array,
    A::Item: PartialEq<A::Item>,
{
    pub fn dedup(&mut self) {
        self.0.dedup()
    }
}

impl<A> SmallVec1<A>
where
    A: Array,
    A::Item: Copy
{
    pub fn try_from_slice(slice: &[A::Item]) -> Result<Self> {
        if slice.is_empty() {
            Err(Size0Error)
        } else {
            Ok(SmallVec1(SmallVec::from_slice(slice)))
        }
    }

    pub fn insert_from_slice(&mut self, index: usize, slice: &[A::Item]) {
        self.0.insert_from_slice(index, slice)
    }

    pub fn extend_from_slice(&mut self, slice: &[A::Item]) {
        self.0.extend_from_slice(slice)
    }
}

impl<A> SmallVec1<A>
where
    A: Array,
    A::Item: Clone
{
    pub fn try_resize(&mut self, len: usize, value: A::Item) -> Result<()> {
        if len == 0 {
            Err(Size0Error)
        } else {
            self.0.resize(len, value);
            Ok(())
        }
    }

    pub fn try_from_elem(element: A::Item, len: usize) -> Result<Self> {
        if len == 0 {
            Err(Size0Error)
        } else {
            Ok(SmallVec1(SmallVec::from_elem(element, len)))
        }
    }
}

impl<A> Into<SmallVec<A>> for SmallVec1<A>
where
    A: Array
{
    fn into(self) -> SmallVec<A> {
        self.into_smallvec()
    }
}

impl<A> Into<Vec<A::Item>> for SmallVec1<A>
where
    A: Array
{
    fn into(self) -> Vec<A::Item> {
        self.into_vec()
    }
}

impl<A> Into<Box<[A::Item]>> for SmallVec1<A>
where
    A: Array
{
    fn into(self) -> Box<[A::Item]> {
        self.into_boxed_slice()
    }
}

impl<A, T> TryFrom<Vec<T>> for SmallVec1<A>
where
    A: Array<Item=T>
{
    type Error = Size0Error;
    fn try_from(vec: Vec<T>) -> Result<Self> {
        Self::try_from_vec(vec)
    }
}

impl<A> TryFrom<SmallVec<A>> for SmallVec1<A>
where
    A: Array
{
    type Error = Size0Error;
    fn try_from(vec: SmallVec<A>) -> Result<Self> {
        Self::try_from_smallvec(vec)
    }
}


impl<A> TryFrom<&'_ [A::Item]> for SmallVec1<A>
where
    A: Array,
    A::Item: Clone
{
    type Error = Size0Error;
    fn try_from(slice: &'_ [A::Item]) -> Result<Self> {
        if slice.is_empty() {
            Err(Size0Error)
        } else {
            Ok(SmallVec1(SmallVec::from(slice)))
        }
    }
}

macro_rules! impl_try_from_into_buf_trait {
    ($($size:expr),*) => ($(
        impl<T> TryFrom<[T; $size]> for SmallVec1<[T; $size]> {
            type Error = Size0Error;
            fn try_from(vec: [T; $size]) -> Result<Self> {
                Self::try_from_buf(vec)
            }
        }

        impl<T> TryInto<[T; $size]> for SmallVec1<[T; $size]> {
            type Error = Self;
            fn try_into(self) -> std::result::Result<[T; $size], Self> {
                self.into_inner()
            }
        }
    )*);
}

//FIXME support const_generics feature
impl_try_from_into_buf_trait!(
    // values from smallvec crate
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
    17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
    32, 36, 0x40, 0x60, 0x80, 0x100, 0x200, 0x400, 0x600, 0x800,
    0x1000, 0x2000, 0x4000, 0x6000, 0x8000, 0x10_000, 0x20_000,
    0x40_000, 0x60_000, 0x80_000, 0x100_000
);

impl<A> Debug for SmallVec1<A>
where
    A: Array,
    A::Item: Debug
{
    #[inline]
    fn fmt(&self, fter: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.0, fter)
    }
}

impl<A> Clone for SmallVec1<A>
where
    A: Array,
    A::Item: Clone
{
    #[inline]
    fn clone(&self) -> Self {
        SmallVec1(self.0.clone())
    }
}

impl<A, B> PartialEq<SmallVec1<B>> for SmallVec1<A>
where
    A: Array,
    B: Array,
    A::Item: PartialEq<B::Item>
{
    #[inline]
    fn eq(&self, other: &SmallVec1<B>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<A, B> PartialEq<B> for SmallVec1<A>
where
    A: Array,
    SmallVec<A>: PartialEq<B>,
{
    #[inline]
    fn eq(&self, other: &B) -> bool {
        self.0.eq(other)
    }
}

impl<A> Eq for SmallVec1<A>
where
    A: Array,
    A::Item: Eq,
{}

impl<A> Hash for SmallVec1<A>
where
    A: Array,
    A::Item: Hash
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<A> PartialOrd for SmallVec1<A>
where
    A: Array,
    A::Item: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &SmallVec1<A>) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<A> Ord for SmallVec1<A>
where
    A: Array,
    A::Item: Ord,
{
    #[inline]
    fn cmp(&self, other: &SmallVec1<A>) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<A> Deref for SmallVec1<A>
where
    A: Array
{
    type Target = [A::Item];

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<A> DerefMut for SmallVec1<A>
where
    A: Array
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

impl<A> IntoIterator for SmallVec1<A>
where
    A: Array
{
    type Item = A::Item;
    type IntoIter = smallvec::IntoIter<A>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, A> IntoIterator for &'a SmallVec1<A>
where
    A: Array
{
    type Item = &'a A::Item;
    type IntoIter = std::slice::Iter<'a, A::Item>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl<'a, A> IntoIterator for &'a mut SmallVec1<A>
where
    A: Array
{
    type Item = &'a mut A::Item;
    type IntoIter = std::slice::IterMut<'a, A::Item>;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.0).into_iter()
    }
}

impl<A> Default for SmallVec1<A>
where
    A: Array,
    A::Item: Default
{
    fn default() -> Self {
        SmallVec1::new(Default::default())
    }
}

impl<A> AsRef<[A::Item]> for SmallVec1<A>
where
    A: Array
{
    fn as_ref(&self) -> &[A::Item] {
        self.0.as_ref()
    }
}


impl<A> AsRef<SmallVec<A>> for SmallVec1<A>
where
    A: Array
{
    fn as_ref(&self) -> &SmallVec<A>{
        &self.0
    }
}


impl<A> AsMut<[A::Item]> for SmallVec1<A>
where
    A: Array
{
    fn as_mut(&mut self) -> &mut [A::Item] {
        self.0.as_mut()
    }
}

impl<A> Borrow<[A::Item]> for SmallVec1<A>
where
    A: Array
{
    fn borrow(&self) -> &[A::Item] {
        self.0.as_ref()
    }
}


impl<A> Borrow<SmallVec<A>> for SmallVec1<A>
where
    A: Array
{
    fn borrow(&self) -> &SmallVec<A>{
        &self.0
    }
}

impl<A, I> Index<I> for SmallVec1<A>
where
    A: Array,
    I: SliceIndex<[A::Item]>
{
    type Output = I::Output;

    fn index(&self, index: I) -> &I::Output {
        self.0.index(index)
    }
}

impl<A, I> IndexMut<I> for SmallVec1<A>
where
    A: Array,
    I: SliceIndex<[A::Item]>
{
    fn index_mut(&mut self, index: I) -> &mut I::Output {
        self.0.index_mut(index)
    }
}


impl<A> BorrowMut<[A::Item]> for SmallVec1<A>
where
    A: Array
{
    fn borrow_mut(&mut self) -> &mut [A::Item] {
        self.0.as_mut()
    }
}

impl<A: Array> Extend<A::Item> for SmallVec1<A> {
    fn extend<I: IntoIterator<Item = A::Item>>(&mut self, iterable: I) {
        self.0.extend(iterable)
    }
}

//Note: We can not (simply) have if feature serde and feature smallvec enable
//      dependency smallvec/serde, but we can mirror the serde implementation.
#[cfg(feature = "serde")]
const _: () = {
    use std::{marker::PhantomData, result::Result};
    use serde::{
        de::{SeqAccess,Deserialize, Visitor, Deserializer, Error as _},
        ser::{Serialize, Serializer, SerializeSeq}
    };

    impl<A> Serialize for SmallVec1<A>
    where
        A: Array,
        A::Item: Serialize,
    {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut seq_ser = serializer.serialize_seq(Some(self.len()))?;
            for item in self {
                seq_ser.serialize_element(&item)?;
            }
            seq_ser.end()
        }
    }

    impl<'de, A> Deserialize<'de> for SmallVec1<A>
    where
        A: Array,
        A::Item: Deserialize<'de>,
    {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            deserializer.deserialize_seq(SmallVec1Visitor {
                _type_carry: PhantomData,
            })
        }
    }
    struct SmallVec1Visitor<A> {
        _type_carry: PhantomData<A>,
    }

    impl<'de, A> Visitor<'de> for SmallVec1Visitor<A>
    where
        A: Array,
        A::Item: Deserialize<'de>,
    {
        type Value = SmallVec1<A>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a sequence")
        }

        fn visit_seq<B>(self, mut seq: B) -> Result<Self::Value, B::Error>
        where
            B: SeqAccess<'de>,
        {
            let len = seq.size_hint().unwrap_or(0);
            let mut smallvec = SmallVec::new();
            smallvec.try_reserve(len).map_err(B::Error::custom)?;

            while let Some(value) = seq.next_element()? {
                smallvec.push(value);
            }

            SmallVec1::try_from(smallvec).map_err(B::Error::custom)
        }
    }
};




#[cfg(test)]
mod tests {
    use std::collections::hash_map::DefaultHasher;
    use super::*;

    #[test]
    fn impl_clone() {
        let a: SmallVec1<[u8; 4]> = smallvec1![1,2,3];
        let b = a.clone();
        assert_eq!(a,b);
    }

    #[test]
    fn impl_eq() {
        let a: SmallVec1<[u8; 4]>  = smallvec1![1,2,3];
        let b: SmallVec1<[u8; 4]>  = smallvec1![1,2,3];
        let c: SmallVec1<[u8; 4]>  = smallvec1![2,2,3];

        assert_eq!(a,b);
        assert_ne!(a,c);
        //make sure Eq is supported and not only PartialEq
        fn cmp<A: Eq>(){}
        cmp::<SmallVec1<[u8; 4]>>();
    }

    #[test]
    fn impl_partial_eq() {
        let a: SmallVec1<[String; 4]>  = smallvec1!["hy".to_owned()];
        let b: SmallVec1<[&'static str; 4]>  = smallvec1!["hy"];
        assert_eq!(a,b);

        let a: SmallVec1<[u8; 4]> = smallvec1![1,2,3,4,5];
        let b: SmallVec1<[u8; 8]> = smallvec1![1,2,3,4,5];
        assert_eq!(a,b);
    }

    #[test]
    fn impl_ord() {
        let a: SmallVec1<[u8; 4]> = smallvec1![1, 2];
        let b: SmallVec1<[u8; 4]> = smallvec1![1, 3];
        assert_eq!(Ord::cmp(&a,&b), Ordering::Less);
    }

    #[test]
    fn impl_hash() {
        let a: SmallVec1<[u8; 4]> = smallvec1![1, 3];
        let b = vec![1u8, 3];
        assert_eq!(compute_hash(&a), compute_hash(&b));

        /// -------------------
        fn compute_hash<T: Hash>(value: &T) -> u64 {
            let mut hasher = DefaultHasher::new();
            value.hash(&mut hasher);
            hasher.finish()
        }
    }

    #[test]
    fn impl_debug() {
        let a: SmallVec1<[u8; 4]> = smallvec1![1, 2];
        assert_eq!(format!("{:?}", a), "[1, 2]");
    }

    #[test]
    fn impl_default() {
        let a = SmallVec1::<[u8; 4]>::default();
        assert_eq!(a.as_slice(), &[0u8] as &[u8]);
    }

    #[test]
    fn impl_deref() {
        let a: SmallVec1<[u8; 4]> = smallvec1![1, 2];
        let _: &SmallVec<_> = a.as_smallvec();
        let b: &[u8] = &*a;
        assert_eq!(b, &[1u8, 2] as &[u8]);
    }

    #[test]
    fn impl_deref_mut() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![1, 2];
        let b: &mut [u8] = &mut *a;
        assert_eq!(b, &[1u8, 2] as &[u8]);
    }

    #[test]
    fn impl_into_iter() {
        let a: SmallVec1<[u8; 4]> = smallvec1![12, 23];
        let a_ = a.clone();
        let b = a.into_iter().collect::<Vec<_>>();
        assert_eq!(&a_[..], &b[..]);
    }

    #[test]
    fn impl_as_ref() {
        let a: SmallVec1<[u8; 4]> = smallvec1![12, 23];
        let _: &[u8] = a.as_ref();
        let _: &SmallVec<[u8; 4]> = a.as_ref();
    }

    #[test]
    fn impl_as_mut() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![12, 23];
        let _: &mut [u8] = a.as_mut();
    }

    #[test]
    fn impl_borrow() {
        let a: SmallVec1<[u8; 4]> = smallvec1![12, 23];
        let _: &[u8] = a.borrow();
        let _: &SmallVec<[u8; 4]> = a.borrow();
    }

    #[test]
    fn impl_borrow_mut() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![12, 23];
        let _: &mut [u8] = a.borrow_mut();
    }

    #[test]
    fn impl_extend() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![12, 23];
        a.extend(vec![1u8,2,3].into_iter());
        assert_eq!(a.as_slice(), &[12u8, 23, 1, 2, 3] as &[u8]);
    }

    #[test]
    fn index() {
        let a: SmallVec1<[u8; 4]> = smallvec1![12, 23];
        assert_eq!(a[0], 12);
    }

    #[test]
    fn index_mut() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![12, 23];
        a[0] = 33;
        assert_eq!(a[0], 33);
    }

    #[test]
    fn impl_try_from_slice_by_from_trait() {
        let a = SmallVec1::<[String; 4]>::try_from(&["hy".to_owned()] as &[String]).unwrap();
        assert_eq!(a[0], "hy");

        SmallVec1::<[String; 4]>::try_from(&[] as &[String]).unwrap_err();
    }

    #[test]
    fn into_iterator_ref() {
        let a: SmallVec1<[u8; 4]> = smallvec1![12, 23];
        let a = (&a).into_iter().collect::<Vec<_>>();
        assert_eq!(a, vec![&12u8, &23]);
    }

    #[test]
    fn into_iterator_ref_mut() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![12, 23];
        let a = (&mut a).into_iter().collect::<Vec<_>>();
        assert_eq!(a, vec![&mut 12u8, &mut 23]);
    }

    #[test]
    fn new() {
        let a = SmallVec1::<[u8; 4]>::new(12);
        let b: SmallVec1<[u8; 4]> = smallvec1![12];
        assert_eq!(a, b);
    }

    #[test]
    fn with_capacity() {
        let a = SmallVec1::<[u8;4]>::with_capacity(32, 21);
        assert_eq!(a.is_empty(), false);
        assert_eq!(a.capacity(), 21);

        let a = SmallVec1::<[u8;4]>::with_capacity(32, 1);
        assert_eq!(a.is_empty(), false);
        assert_eq!(a.capacity(), 4/*yes 4!*/);
    }

    #[test]
    fn try_from_vec() {
        let a = SmallVec1::<[u8; 4]>::try_from_vec(vec![1,2,3]);
        assert_eq!(a, Ok(smallvec1![1,2,3]));

        let b = SmallVec1::<[u8; 4]>::try_from_vec(vec![]);
        assert_eq!(b, Err(Size0Error));
    }

    #[test]
    fn try_from_smallvec() {
        let a = SmallVec1::<[u8; 4]>::try_from_smallvec(smallvec![32,2,3]);
        assert_eq!(a, Ok(smallvec1![32,2,3]));

        let a = SmallVec1::<[u8; 4]>::try_from_smallvec(smallvec![]);
        assert_eq!(a, Err(Size0Error));
    }

    #[test]
    fn try_from_buf() {
        let a = SmallVec1::try_from_buf([1u8, 2, 3, 4]);
        assert_eq!(a, Ok(smallvec1![1,2,3,4]));

        let a = SmallVec1::try_from_buf([] as [u8; 0]);
        assert_eq!(a, Err(Size0Error));
    }

    #[test]
    fn try_from_buf_and_len() {
        let a = SmallVec1::try_from_buf_and_len([1u8, 2, 3, 4, 0, 0, 0, 0], 4);
        assert_eq!(a, Ok(smallvec1![1,2,3,4]));

        let a = SmallVec1::try_from_buf_and_len([1u8, 2, 3], 0);
        assert_eq!(a, Err(Size0Error));
    }

    #[should_panic]
    #[test]
    fn try_from_buf_and_len_panic_if_len_gt_size() {
        let _ = SmallVec1::try_from_buf_and_len([] as [u8; 0], 3);
    }

    #[test]
    fn impl_try_from_traits() {
        let _ = SmallVec1::<[u8; 4]>::try_from(vec![1,2,3]).unwrap();
        let _ = SmallVec1::<[u8; 4]>::try_from(vec![]).unwrap_err();
        let _ = SmallVec1::<[u8; 4]>::try_from(smallvec![1,2,3]).unwrap();
        let _ = SmallVec1::<[u8; 4]>::try_from(smallvec![]).unwrap_err();
        let _ = SmallVec1::<[u8; 4]>::try_from([1u8,2,3,4]).unwrap();
        let _ = SmallVec1::<[u8; 0]>::try_from([] as [u8; 0]).unwrap_err();
    }

    #[test]
    fn into_smallvec() {
        let a: SmallVec1<[u8; 4]> = smallvec1![1,3,2];
        let a = a.into_smallvec();
        let b: SmallVec<[u8; 4]> = smallvec![1,3,2];
        assert_eq!(a, b);
    }

    #[test]
    fn into_vec() {
        let a: SmallVec1<[u8; 4]> = smallvec1![1,3,2];
        let a: Vec<u8> = a.into_vec();
        assert_eq!(a, vec![1,3,2])
    }

    #[test]
    fn into_inner() {
        let a: SmallVec1<[u8; 4]> = smallvec1![1,3,2,4];
        let a: [u8; 4] = a.into_inner().unwrap();
        assert_eq!(a, [1, 3, 2, 4])
    }

    #[test]
    fn into_boxed_slice() {
        let a: SmallVec1<[u8; 4]> = smallvec1![1,3,2,4];
        let a: Box<[u8]> = a.into_boxed_slice();
        assert_eq!(&*a, &[1u8, 3, 2, 4] as &[u8])
    }


    #[test]
    fn into_traits() {
        let a: SmallVec1<[u8; 4]> = smallvec1![1,3,2,4];
        let _: Vec<u8> = a.into();

        let a: SmallVec1<[u8; 4]> = smallvec1![1,3,2,4];
        let _: SmallVec<[u8; 4]> = a.into();

        let a: SmallVec1<[u8; 4]> = smallvec1![1,3,2,4, 5];
        let _: Box<[u8]> = a.into();

        let a: SmallVec1<[u8; 4]> = smallvec1![1,3,2,4];
        let a: std::result::Result<[u8; 4], _> = a.try_into();
        a.unwrap();

        let a: SmallVec1<[u8; 4]> = smallvec1![1,3,2];
        let a: std::result::Result<[u8; 4],_> = a.try_into();
        a.unwrap_err();
    }

    #[test]
    fn last_first_methods_are_shadowed() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![1, 3, 2, 4];
        assert_eq!(a.last(), &4);
        assert_eq!(a.last_mut(), &mut 4);
        assert_eq!(a.first(), &1);
        assert_eq!(a.first_mut(), &mut 1);
    }

    #[test]
    fn try_truncate() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![1, 3, 2, 4];
        assert_eq!(a.try_truncate(0), Err(Size0Error));
        assert_eq!(a.try_truncate(1), Ok(()));
        assert_eq!(a.len(), 1);
    }

    //TODO try_drain

    #[test]
    fn reserve() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![1, 3, 2, 4];
        a.reserve(4);
        assert!(a.capacity() >= 8);
    }

    #[test]
    fn try_reserve() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![1, 3, 2, 4];
        a.try_reserve(4).unwrap();
        assert!(a.capacity() >= 8);
    }

    #[test]
    fn reserve_exact() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![1, 3, 2, 4];
        a.reserve_exact(4);
        assert_eq!(a.capacity(), 8);
    }

    #[test]
    fn try_reserve_exact() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![1, 3, 2, 4];
        a.try_reserve_exact(4).unwrap();
        assert_eq!(a.capacity(), 8);
    }

    #[test]
    fn shrink_to_fit() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![1, 3, 2, 4, 5];
        a.shrink_to_fit();
        assert_eq!(a.capacity(), 5);
    }

    #[test]
    fn push() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![1, 3];
        a.push(12);
        let b: SmallVec1<[u8; 4]> = smallvec1![1, 3, 12];
        assert_eq!(a,b);
    }

    #[test]
    fn insert() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![1, 3];
        a.insert(0, 12);
        let b: SmallVec1<[u8; 4]> = smallvec1![12, 1, 3];
        assert_eq!(a,b);
    }

    #[test]
    fn len() {
        let a: SmallVec1<[u8; 4]> = smallvec1![1, 3];
        assert_eq!(a.len(), 2);
    }

    #[test]
    fn capacity() {
        let a: SmallVec1<[u8; 4]> = smallvec1![1, 3];
        assert_eq!(a.capacity(), 4);
    }

    #[test]
    fn as_slice() {
        let a: SmallVec1<[u8; 4]> = smallvec1![1, 3];
        assert_eq!(a.as_slice(), &[1u8,3] as &[u8]);
    }

    #[test]
    fn as_mut_slice() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![1, 3];
        a.as_mut_slice()[0] = 10;
        let b: SmallVec1<[u8; 4]> = smallvec1![10, 3];
        assert_eq!(a,b);
    }

    #[test]
    fn inline_size() {
        let a: SmallVec1<[u8; 4]> = smallvec1![1, 3];
        assert_eq!(a.inline_size(), 4);
    }

    #[test]
    fn spilled() {
        let a: SmallVec1<[u8; 4]> = smallvec1![1, 3];
        assert_eq!(a.spilled(), false);

        let a: SmallVec1<[u8; 4]> = smallvec1![1, 3, 6, 9, 2];
        assert_eq!(a.spilled(), true);
    }

    #[test]
    fn try_pop() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![1, 3];
        assert_eq!(a.try_pop(), Ok(3));
        assert_eq!(a.try_pop(), Err(Size0Error));
    }

    #[test]
    fn append() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![1, 3];
        let mut b: SmallVec<[u8; 4]> = smallvec![53, 12];
        a.append(&mut b);
        let c: SmallVec1<[u8; 4]> = smallvec1![1, 3, 53, 12];
        assert_eq!(a, c);
    }

    #[test]
    fn grow() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![1, 3];
        a.grow(32);
        assert_eq!(a.capacity(), 32);
    }

    #[test]
    fn try_grow() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![1, 3];
        a.try_grow(32).unwrap();
        assert_eq!(a.capacity(), 32);
    }

    #[test]
    fn try_swap_remove() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![1, 3];
        assert_eq!(a.try_swap_remove(0), Ok(1));
        assert_eq!(a.try_swap_remove(0), Err(Size0Error));
    }

    #[test]
    fn try_remove() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![1, 3];
        assert_eq!(a.try_remove(0), Ok(1));
        assert_eq!(a.try_remove(0), Err(Size0Error));
    }

    #[test]
    fn insert_many() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![1, 3];
        a.insert_many(1, vec![2, 4, 8]);
        let b: SmallVec1<[u8; 4]> = smallvec1![1, 2, 4, 8, 3];
        assert_eq!(a, b);
    }

    #[test]
    fn dedup() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![1, 1];
        a.dedup();
        assert_eq!(a.as_slice(), &[1u8] as &[u8]);
    }

    #[test]
    fn dedup_by() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![1, 1, 4, 4];
        a.dedup_by(|a,b| a==b);
        assert_eq!(a.as_slice(), &[1u8, 4] as &[u8]);
    }

    #[test]
    fn dedup_by_key() {
        let mut a: SmallVec1<[(u8,u8); 4]> = smallvec1![ (1, 2), (1, 5), (4, 4), (5, 4) ];
        a.dedup_by_key(|a| a.0);
        assert_eq!(a.as_slice(), &[(1u8, 2u8), (4, 4), (5, 4)] as &[(u8, u8)]);
    }

    #[test]
    fn try_resize_with() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![1, 2];
        assert_eq!(a.try_resize_with(0, Default::default), Err(Size0Error));
        assert_eq!(a.try_resize_with(4, Default::default), Ok(()));
    }

    #[test]
    fn as_ptr() {
        let a: SmallVec1<[u8; 4]> = smallvec1![1, 2];
        let pa = a.as_ptr();
        let pb = a.as_slice().as_ptr();
        assert_eq!(pa as usize, pb as usize);
    }

    #[test]
    fn as_mut_ptr() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![1, 2];
        let pa = a.as_mut_ptr();
        let pb = a.as_mut_slice().as_mut_ptr();
        assert_eq!(pa as usize, pb as usize);
    }

    #[test]
    fn try_from_slice() {
        let a = SmallVec1::<[u8; 4]>::try_from_slice(&[1u8, 2, 9]).unwrap();
        assert_eq!(a.as_slice(), &[1u8, 2, 9] as &[u8]);

        SmallVec1::<[u8; 4]>::try_from_slice(&[]).unwrap_err();
    }

    #[test]
    fn insert_from_slice() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![1, 2];
        a.insert_from_slice(1, &[3, 9]);
        assert_eq!(a.as_slice(), &[1u8, 3, 9, 2] as &[u8]);
    }

    #[test]
    fn extend_from_slice() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![1, 2];
        a.extend_from_slice(&[3, 9]);
        assert_eq!(a.as_slice(), &[1u8, 2, 3, 9] as &[u8]);
    }

    #[test]
    fn try_resize() {
        let mut a: SmallVec1<[u8; 4]> = smallvec1![1, 2, 3];
        assert_eq!(a.try_resize(0, 12), Err(Size0Error));
        assert_eq!(a.try_resize(2, 12), Ok(()));
        assert_eq!(a.try_resize(4, 12), Ok(()));
        assert_eq!(a.as_slice(), &[1u8, 2, 12, 12] as &[u8]);
    }

    #[test]
    fn try_from_elem() {
        let a = SmallVec1::<[u8; 4]>::try_from_elem(1u8, 3).unwrap();
        assert_eq!(a.as_slice(), &[1u8, 1, 1] as &[u8]);

        SmallVec1::<[u8; 4]>::try_from_elem(1u8, 0).unwrap_err();
    }


    #[test]
    fn split_off_first() {
        let a: SmallVec1<[u8; 4]> = smallvec1![32];
        assert_eq!((32, SmallVec::<[u8; 4]>::new()), a.split_off_first());

        let a: SmallVec1<[u8; 4]> = smallvec1![32, 43];
        let exp: SmallVec<[u8; 4]> = smallvec![43];
        assert_eq!((32, exp), a.split_off_first());
    }

    #[test]
    fn split_off_last() {
        let a: SmallVec1<[u8; 4]> = smallvec1![32];
        assert_eq!((SmallVec::<[u8; 4]>::new(), 32), a.split_off_last());

        let a: SmallVec1<[u8; 4]> = smallvec1![32, 43];
        let exp: SmallVec<[u8; 4]> = smallvec![32];
        assert_eq!((exp, 43), a.split_off_last());
    }

    #[cfg(feature="serde")]
    #[test]
    fn can_be_serialized_and_deserialized() {
        let a: SmallVec1<[u8; 4]> = smallvec1![32,12,14,18,201];
        let json_str = serde_json::to_string(&a).unwrap();
        let b: SmallVec1<[u8; 4]> = serde_json::from_str(&json_str).unwrap();
        assert_eq!(a, b);
    }

    #[cfg(feature="serde")]
    #[test]
    fn array_size_is_not_serialized() {
        let a: SmallVec1<[u8; 4]> = smallvec1![32,12,14,18,201];
        let json_str = serde_json::to_string(&a).unwrap();
        let b: SmallVec1<[u8; 8]> = serde_json::from_str(&json_str).unwrap();
        assert_eq!(a, b);
    }

    #[cfg(feature="serde")]
    #[test]
    fn does_not_allow_empty_deserialization() {
        let a = Vec::<u8>::new();
        let json_str = serde_json::to_string(&a).unwrap();
        serde_json::from_str::<SmallVec1<[u8;8]>>(&json_str).unwrap_err();
    }


}
