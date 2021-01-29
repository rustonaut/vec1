use std::{fmt::{self, Debug}, ops::Deref, cmp::{Ord, Ordering, PartialEq, Eq}, hash::{Hash, Hasher}};

use smallvec_v1_ as smallvec;
use smallvec::*;


/// A macro similar to `vec!` to create a `SmallVec1`.
///
/// If it is called with less then 1 element a
/// compiler error is triggered (using `compile_error`
/// to make sure you know what went wrong).
#[macro_export]
macro_rules! smallvec1 {
    () => (
        compile_error!("SmallVec1 needs at least 1 element")
    );
    ($first:expr $(, $item:expr)* , ) => (
        $crate::smallvec1!($first $(, $item)*)
    );
    ($first:expr $(, $item:expr)* ) => ({
        let smallvec = $crate::smallvec_v1_::smallvec!($first $(, $item)*);
        SmallVec1(smallvec)
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
//FIXME #[cfg_attr(feature = "serde", derive(serde::Serialize))]
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

}

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
    type Target = SmallVec<A>;

    fn deref(&self) -> &Self::Target {
        &self.0
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
    fn impl_into_iter() {
        let a: SmallVec1<[u8; 4]> = smallvec1![12, 23];
        let a_ = a.clone();
        let b = a.into_iter().collect::<Vec<_>>();
        assert_eq!(&a_[..], &b[..]);
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
}
