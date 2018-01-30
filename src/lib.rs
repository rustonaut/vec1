//! This crate provides a `Vec` wrapper (`Vec1`) which guarantees to have at last 1 element.
//!
//! This can be usefull if you have a API which accepts one ore more ofe a kind. Instead
//! of accepting a `Vec` and returning an error if it's empty a `Vec1` can be used assuring
//! there is at last 1 element and through this reducing the number of possible error causes.
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
//!     // vec1![] makes sure there is at last one element
//!     // at compiler time
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
use std::{fmt, vec, slice};
use std::ops::{ Deref, DerefMut, Index, IndexMut};
use std::result::{ Result as StdResult };
use std::error::{ Error as StdError };
use std::iter::{IntoIterator, Extend};
use std::borrow::{Borrow, BorrowMut};

/// a macro similar to `vec!` to create a `Vec1`
///
/// If it is called with less then 1 element a
/// compiler error is triggered (using `compile_error`
/// to make sure you know what went wrong)
#[macro_export]
macro_rules! vec1 {
    ( ) => (
        compile_error!("Vec1 needs at last 1 element")
    );
    ( $first:expr) => (
         $crate::Vec1::new( $first )
    );
    ( $first:expr,) => (
         $crate::Vec1::new( $first )
    );
    ( $first:expr, $($item:expr),* ) => ({
        let mut tmp = $crate::Vec1::new( $first );
        $( tmp.push( $item ); )*
        tmp
    });
}


/// Error returned by operations which would cause Vec1 to have a len of 0
#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct Size0Error;

impl fmt::Display for Size0Error {
    fn fmt( &self, fter: &mut fmt::Formatter ) -> fmt::Result {
        write!( fter, "{:?}", self )
    }
}
impl StdError for Size0Error {
    fn description(&self) -> &str {
        "failing function call would have reduced the size of a Vec1 to 0, which is not allowed"
    }
}

type Vec1Result<T> = StdResult<T, Size0Error>;

/// `std::vec::Vec` wrapper which guarantees to have at last 1 element
///
/// `Vec1<T>` dereferences to `&[T]` and `&mut [T]` as functionality
/// exposed through this can not change the length.
///
/// Methods of `Vec` which can be called without reducing the length
/// (e.g. `capacity()`, `reserve()`) are exposed through wrappers
/// with the same signature.
///
/// Methods of `Vec` which could reduce the length to 0 if exposed
/// are implemented with a `try_` prefix returning a `Result`.
/// (e.g. `try_pop(&self)`, `try_truncate()`, etc.).
///
/// Methods with returned `Option<T>` with `None` if the length was 0
/// (and do not reduce the length now) now return T. (e.g. `first`,
/// `last`, `first_mut`, etc.).
///
/// All stable traits and methods implemented on `Vec<T>` are _should_ also
/// be implemented on `Vec1<T>` (except if they make no sense to implement
/// due to the len 1 gurantee). Note that some small thinks are still missing
/// e.g. `Vec1` does not implement drain currently as drains generic argument
/// is `R: RangeArgument<usize>` and `RangeArgument` is not stable.
#[derive( Debug, Clone, Eq, Hash, PartialOrd, Ord )]
pub struct Vec1<T>(Vec<T>);

impl<T> IntoIterator for Vec1<T> {
    type Item = T;
    type IntoIter = vec::IntoIter<T>;

    fn into_iter( self ) -> Self::IntoIter {
        self.0.into_iter()
    }

}

impl<T> Vec1<T> {


    pub fn new( first: T  ) -> Self {
        Vec1( vec![ first ] )
    }

    pub fn from_vec( vec: Vec<T> ) -> StdResult<Self, Vec<T>> {
        if vec.len() > 0 {
            Ok( Vec1( vec ) )
        } else {
            Err( vec )
        }
    }

    pub fn with_capacity( first: T, capacity: usize ) -> Self {
        let mut vec = Vec::with_capacity( capacity );
        vec.push( first );
        Vec1( vec )
    }

    pub fn into_vec( self ) -> Vec<T> {
        self.0
    }

    /// create a new Vec1 by consuming this vec1 and mapping each element
    ///
    /// This is usefull as it keeps the knowledge that the length is >= 1,
    /// even through the old Vec1 is consumed and turned into an iterator.
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
    /// //without mapped
    /// let data = Vec1::from_vec(data.into_iter().map(|x|x*2).collect::<Vec<_>>()).unwrap();
    /// assert_eq!(data, vec![4,8,12]);
    /// # }
    /// ```
    pub fn mapped<F, N>(self, map_fn: F) -> Vec1<N>
        where F: FnMut(T) -> N
    {
        Vec1(self.into_iter().map(map_fn).collect::<Vec<_>>())
    }

    /// create a new Vec1 by consuming this vec1 and mapping each element
    ///
    /// This is usefull as it keeps the knowledge that the length is >= 1,
    /// even through the old Vec1 is consumed and turned into an iterator.
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
        where F: FnMut(T) -> Result<N, E>
    {
        let mut map_fn = map_fn;
        // ::collect<Result<Vec<_>>>() is uses the iterators size hint's lower bound
        // for with_capacity, which is 0 as it might fail at the first element
        let mut out = Vec::with_capacity(self.len());
        for element in self.into_iter() {
            out.push(map_fn(element)?);
        }
        Ok(Vec1(out))
    }


    /// returns a reference to the last element
    /// as Vec1 contains always at last one element
    /// there is always a last element
    pub fn last( &self ) -> &T {
        //UNWRAP_SAFE: len is at last 1
        self.0.last().unwrap()
    }

    pub fn last_mut( &mut self ) -> &mut T {
        //UNWRAP_SAFE: len is at last 1
        self.0.last_mut().unwrap()
    }

    /// returns a reference to the first element
    /// as Vec1 contains always at last one element
    /// there is always a first element
    pub fn first( &self ) -> &T {
        //UNWRAP_SAFE: len is at last 1
        self.0.first().unwrap()
    }

    pub fn first_mut( &mut self ) -> &mut T {
        //UNWRAP_SAFE: len is at last 1
        self.0.first_mut().unwrap()
    }

    pub fn try_truncate(&mut self, len: usize) -> Vec1Result<()> {
        if len > 0 {
            self.0.truncate( len );
            Ok( () )
        } else {
            Err( Size0Error )
        }
    }

    pub fn try_swap_remove(&mut self, index: usize) -> Vec1Result<T> {
        if self.len() > 1 {
            Ok( self.0.swap_remove( index ) )
        } else {
            Err( Size0Error )
        }
    }

    pub fn try_remove( &mut self, index: usize ) -> Vec1Result<T> {
        if self.len() > 1 {
            Ok( self.0.remove( index ) )
        } else {
            Err( Size0Error )
        }
    }

    pub fn try_split_off(&mut self, at: usize) -> Vec1Result<Vec1<T>> {
        if at == 0 {
            Err(Size0Error)
        } else if at >= self.len() {
            Err(Size0Error)
        } else {
            let out = self.0.split_off(at);
            Ok(Vec1(out))
        }
    }

    pub fn dedup_by_key<F, K>(&mut self, key: F)
        where F: FnMut(&mut T) -> K,
              K: PartialEq<K>
    {
        self.0.dedup_by_key( key )
    }

    pub fn dedup_by<F>(&mut self, same_bucket: F)
        where F: FnMut(&mut T, &mut T) -> bool
    {
        self.0.dedup_by( same_bucket )
    }


    /// pops if there is _more_ than 1 element in the vector
    pub fn try_pop(&mut self) -> Vec1Result<T> {
        if self.len() > 1 {
            //UNWRAP_SAFE: pop on len > 1 can not be none
            Ok(self.0.pop().unwrap())
        } else {
            Err(Size0Error)
        }
    }

    pub fn as_vec(&self) -> &Vec<T> {
        &self.0
    }

}

macro_rules! impl_wrapper {
    (pub $T:ident>
        $(fn $name:ident(&$($m:ident)* $(, $param:ident: $tp:ty)*) -> $rt:ty);*) => (
            impl<$T> Vec1<$T> {$(
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
    pub T>
        fn reserve(&mut self, additional: usize) -> ();
        fn reserve_exact(&mut self, additional: usize) -> ();
        fn shrink_to_fit(&mut self) -> ();
        fn as_mut_slice(&mut self) -> &mut [T];
        fn push(&mut self, value: T) -> ();
        fn append(&mut self, other: &mut Vec<T>) -> ();
        fn insert(&mut self, idx: usize, val: T) -> ();
        fn len(&self) -> usize;
        fn capacity(&self) -> usize;
        fn as_slice(&self) -> &[T]
}


impl<T> Vec1<T> where T: Clone {
    pub fn try_resize(&mut self, new_len: usize, value: T) -> Vec1Result<()> {
        if new_len >= 1 {
            Ok( self.0.resize( new_len, value ) )
        } else {
            Err( Size0Error )
        }
    }

    pub fn extend_from_slice(&mut self, other: &[T]) {
        self.0.extend_from_slice( other )
    }
}

impl<T> Vec1<T> where T: PartialEq<T> {
    pub fn dedub(&mut self) {
        self.0.dedup()
    }
}


impl<T> Vec1<T> where T: PartialEq<T> {
    pub fn dedup(&mut self) {
        self.0.dedup()
    }
}


impl<T> Deref for Vec1<T> {
    type Target = [T];

    fn deref( &self ) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Vec1<T> {
    fn deref_mut( &mut self ) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Into<Vec<T>> for Vec1<T> {

    fn into( self ) -> Vec<T> {
        self.0
    }
}

impl<A, B> PartialEq<Vec1<B>> for Vec1<A>
    where A: PartialEq<B>
{
    fn eq(&self, other: &Vec1<B>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<A, B> PartialEq<B> for Vec1<A>
    where Vec<A>: PartialEq<B>
{
    fn eq(&self, other: &B) -> bool {
        self.0.eq(other)
    }
}


impl<T, O, R> Index<R> for Vec1<T>
    where Vec<T>: Index<R, Output=O>,
          O: ?Sized
{
    type Output = O;

    fn index(&self, index: R) -> &O {
        self.0.index(index)
    }
}

impl<T, O, R> IndexMut<R> for Vec1<T>
    where Vec<T>: IndexMut<R, Output=O>,
          O: ?Sized
{
    fn index_mut(&mut self, index: R) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl<T> Borrow<[T]> for Vec1<T> {
    fn borrow(&self) -> &[T] {
        self
    }
}

impl<T> BorrowMut<[T]> for Vec1<T> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T> Borrow<Vec<T>> for Vec1<T> {
    fn borrow(&self) -> &Vec<T> {
        &self.0
    }
}

impl<'a, T> Extend<&'a T> for Vec1<T>
    where T: 'a + Copy
{
    fn extend<I>(&mut self, iter: I)
        where I: IntoIterator<Item = &'a T>
    {
        self.0.extend(iter)
    }
}

impl<T> Extend<T> for Vec1<T> {
    fn extend<I>(&mut self, iter: I)
        where I: IntoIterator<Item = T>
    {
        self.0.extend(iter)
    }
}

impl<T> AsRef<[T]> for Vec1<T> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T> AsMut<[T]> for Vec1<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T> AsRef<Vec<T>> for Vec1<T> {
    fn as_ref(&self) -> &Vec<T> {
        &self.0
    }
}
impl<T> AsRef<Vec1<T>> for Vec1<T> {
    fn as_ref(&self) -> &Vec1<T> {
        self
    }
}

impl<T> AsMut<Vec1<T>> for Vec1<T> {
    fn as_mut(&mut self) -> &mut Vec1<T> {
        self
    }
}

impl<'a, T> IntoIterator for &'a Vec1<T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
impl<'a, T> IntoIterator for &'a mut Vec1<T> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}


#[cfg(test)]
mod test {

    #[macro_export]
    macro_rules! assert_ok {
        ($val:expr) => ({
            match $val {
                Ok( res ) => res,
                Err( err ) => panic!( "expected Ok(..) got Err({:?})", err)
            }
        });
        ($val:expr, $ctx:expr) => ({
            match $val {
                Ok( res ) => res,
                Err( err ) => panic!( "expected Ok(..) got Err({:?}) [ctx: {:?}]", err, $ctx)
            }
        });
    }

    macro_rules! assert_err {
        ($val:expr) => ({
            match $val {
                Ok( val ) => panic!( "expected Err(..) got Ok({:?})", val),
                Err( err ) => err,
            }
        });
        ($val:expr, $ctx:expr) => ({
            match $val {
                Ok( val ) => panic!( "expected Err(..) got Ok({:?}) [ctx: {:?}]", val, $ctx),
                Err( err ) => err,
            }
        });
    }

    mod Size0Error {
        #![allow(non_snake_case)]
        use super::super::*;

        #[test]
        fn implements_std_error() {
            fn comp_check<T: StdError>(){}
            comp_check::<Size0Error>();
        }
    }

    mod Vec1 {
        #![allow(non_snake_case)]
        use super::super::*;

        #[test]
        fn now_warning_on_empty_vec() {
            #![deny(warnings)]

            let _ = vec1![1u8,];
            let _ = vec1![1u8];

        }

        #[test]
        fn deref_slice() {
            let vec = Vec1::new(1u8);
            let _: &[u8] = &*vec;
        }

        #[test]
        fn deref_slice_mut() {
            let mut vec = Vec1::new(1u8);
            let _: &mut [u8] = &mut *vec;
        }

        #[test]
        fn provided_all_ro_functions() {
            let vec = Vec1::new(1u8);
            assert_eq!(vec.len(), 1);
            assert!(vec.capacity() > 0);
            assert_eq!(vec.as_slice(), &*vec);
            // there is obviously no reason we should provide this,
            // as it can't be empty at all, that's the point behind Vec1
            //assert_eq!(vec.is_empty(), true)
        }

        #[test]
        fn provides_some_safe_mut_functions() {
            let mut vec = Vec1::new(1u8);
            vec.reserve(12);
            assert!(vec.capacity() >= 13);
            vec.reserve_exact(31);
            assert!(vec.capacity() >= 31);
            vec.shrink_to_fit();
            let _: &mut [u8] = vec.as_mut_slice();
            vec.insert(1, 31u8);
            vec.insert(1, 2u8);
            assert_eq!(&*vec, &[1, 2, 31]);
            vec.dedup_by_key(|k| *k/3);
            assert_eq!(&*vec, &[1, 31]);
            vec.push(31);
            assert_eq!(&*vec, &[1, 31, 31]);
            vec.dedup_by(|l,r| l == r);
            assert_eq!(&*vec, &[1, 31]);
            vec.extend_from_slice(&[31,2,3]);
            assert_eq!(&*vec, &[1, 31, 31, 2, 3]);
            vec.dedub();
            assert_eq!(&*vec, &[1, 31, 2, 3]);
            // as the passed in vec is emptied this won't work with a vec1 as parameter
            vec.append(&mut vec![1,2,3]);
            assert_eq!(&*vec, &[1, 31, 2, 3, 1, 2, 3])
        }

        #[test]
        fn provides_other_methos_in_failible_form() {
            let mut vec = vec1![1u8,2,3,4];
            assert_ok!(vec.try_truncate(3));
            assert_err!(vec.try_truncate(0));
            assert_eq!(vec, &[1,2,3]);

            assert_ok!(vec.try_swap_remove(0));
            assert_eq!(vec, &[3, 2]);
            assert_ok!(vec.try_remove(0));
            assert_eq!(vec, &[2]);
            assert_err!(vec.try_swap_remove(0));
            assert_err!(vec.try_remove(0));
            vec.push(12);

            assert_eq!(vec.try_pop(), Ok(12));
            assert_eq!(vec.try_pop(), Err(Size0Error));
            assert_eq!(vec, &[2]);

        }

        #[test]
        fn try_split_of() {
            let mut vec = vec1![1,2,3,4];
            assert_err!(vec.try_split_off(0));
            let len = vec.len();
            assert_err!(vec.try_split_off(len));
            let nvec = assert_ok!(vec.try_split_off(len-1));
            assert_eq!(vec, &[1,2,3]);
            assert_eq!(nvec, &[4]);
        }

        #[test]
        fn try_resize() {
            let mut vec = Vec1::new(1u8);
            assert_ok!(vec.try_resize(10,2u8));
            assert_eq!(vec.len(), 10);
            assert_ok!(vec.try_resize(1, 2u8));
            assert_eq!(vec, &[1]);
            assert_err!(vec.try_resize(0, 2u8));
        }


        #[test]
        fn with_capacity() {
            let vec = Vec1::with_capacity(1u8, 16);
            assert!(vec.capacity() >= 16);
        }

        #[test]
        fn impl_index() {
            let vec = vec1![ 1,2,3,3];
            assert_eq!(&vec[..2], &[1,2]);
        }
        #[test]
        fn impl_index_mut() {
            let mut vec = vec1![ 1,2,3,3];
            assert_eq!(&mut vec[..2], &mut [1,2]);
        }

        #[test]
        fn impl_extend() {
            let mut vec = Vec1::new(1u8);
            vec.extend([2,3].iter().cloned());
            assert_eq!(vec, &[1, 2, 3]);
        }

        #[test]
        fn impl_extend_ref_copy() {
            let mut vec = Vec1::new(1u8);
            vec.extend([2,3].iter());
            assert_eq!(vec, &[1, 2, 3]);
        }

        #[test]
        fn impl_borrow_mut_slice() {
            fn chk<E, T: BorrowMut<[E]>>(){};
            chk::<u8, Vec1<u8>>();
        }

        #[test]
        fn impl_borrow_slice() {
            fn chk<E, T: BorrowMut<[E]>>(){};
            chk::<u8, Vec1<u8>>();
        }

        #[test]
        fn impl_as_mut_slice() {
            fn chk<E, T: AsMut<[E]>>(){};
            chk::<u8, Vec1<u8>>();
        }

        #[test]
        fn impl_as_ref() {
            fn chk<E, T: AsRef<[E]>>(){};
            chk::<u8, Vec1<u8>>();
        }
        #[test]
        fn impl_as_mut_slice_self() {
            fn chk<E, T: AsMut<Vec1<E>>>(){};
            chk::<u8, Vec1<u8>>();
        }

        #[test]
        fn impl_as_ref_self() {
            fn chk<E, T: AsRef<Vec1<E>>>(){};
            chk::<u8, Vec1<u8>>();
        }

        #[test]
        fn impl_as_ref_vec() {
            fn chk<E, T: AsRef<Vec<E>>>(){};
            chk::<u8, Vec1<u8>>();
        }

        //into iter self, &, &mut
        #[test]
        fn impl_into_iter() {
            let vec = vec1![ 1, 2, 3];
            assert_eq!(6, vec.into_iter().sum());
        }
        #[test]
        fn impl_into_iter_on_ref() {
            let vec = vec1![ 1, 2, 3];
            assert_eq!(6, (&vec).into_iter().sum());
        }
        #[test]
        fn impl_into_iter_on_ref_mut() {
            let mut vec = vec1![ 1, 2, 3];
            assert_eq!(3, (&mut vec).into_iter().fold(0u8, |x, m| {
                *m = *m + 1;
                x + 1
            }));
            assert_eq!(vec, &[2,3,4]);
        }

        #[test]
        fn non_slice_indexing_works() {
            let mut vec = vec1!["a"];
            assert_eq!(&mut vec[0], &mut "a");
        }


    }


}