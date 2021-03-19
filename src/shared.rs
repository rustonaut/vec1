macro_rules! impl_wrapper {
    (
        base_bounds_macro = $($tb:ident : $trait:ident)?,
        impl <$A:ident> $ty_name:ident<$A_:ident> {
            $(fn $fn_name:ident(&$($m:ident)* $(, $param:ident: $tp:ty)*) -> $rt:ty);*
        }
    ) => (
            impl<$A> $ty_name<$A>
            where
                $($tb : $trait,)?
            {$(
                /// See [`Vec`] for a rough idea how this method works.
                #[inline]
                pub fn $fn_name(self: impl_wrapper!{__PRIV_SELF &$($m)*} $(, $param: $tp)*) -> $rt {
                    (self.0).$fn_name($($param),*)
                }
            )*}
    );
    (__PRIV_SELF &mut self) => (&mut Self);
    (__PRIV_SELF &self) => (&Self);
}

macro_rules! shared_impl {
    //Workaround for limitations of rustdoc
    (@IMPORTS) => (
        use core::{
            borrow::{Borrow, BorrowMut},
            cmp::{Eq, Ord, Ordering, PartialEq},
            convert::TryFrom,
            fmt::{self, Debug},
            hash::{Hash, Hasher},
            ops::{Deref, DerefMut, Index, IndexMut},
            slice::SliceIndex,
        };
        use alloc::{vec::Vec, boxed::Box};
    );
    (
        base_bounds_macro = $($tb:ident : $trait:ident)?,
        item_ty_macro = $item_ty:ty,
        $(#[$attr:meta])*
        $v:vis struct $name:ident<$t:ident>($wrapped:ident<$_t:ident>);
    ) => (
        $(#[$attr])*
        $v struct $name<$t>($wrapped<$t>)
        where
            $($tb : $trait,)?;

        // FIXME: This is currently not possible as this will cause the
        //        methods not to be documented by rustdoc:
        //        https://github.com/rust-lang/rust/issues/83026
        // const _: () = {
            impl<$t> $name<$t>
            where
                $($tb : $trait,)?
            {
                /// Creates a new  instance containing a single element.
                pub fn new(first: $item_ty) -> Self {
                    let mut inner = $wrapped::new();
                    inner.push(first);
                    $name(inner)
                }

                /// Creates a new instance with a given capacity and a given "first" element.
                pub fn with_capacity(first: $item_ty, capacity: usize) -> Self {
                    let mut vec = $wrapped::with_capacity(capacity);
                    vec.push(first);
                    $name(vec)
                }

                /// Tries to create a instance from a normal `Vec<T>`.
                ///
                /// # Errors
                ///
                /// This will fail if the input `Vec<T>` is empty.
                /// The returned error is a `Size0Error` instance, as
                /// such this means the _input vector will be dropped if
                /// it's empty_. But this is normally fine as it only
                /// happens if the `Vec<T>` is empty.
                ///
                pub fn try_from_vec(vec: Vec<$item_ty>) -> Result<Self, Size0Error> {
                    if vec.is_empty() {
                        Err(Size0Error)
                    } else {
                        Ok($name($wrapped::from(vec)))
                    }
                }

                /// Returns a reference to the last element.
                ///
                /// As `$name` always contains at least one element there is always a last element.
                pub fn last(&self) -> &$item_ty {
                    //UNWRAP_SAFE: len is at least 1
                    self.0.last().unwrap()
                }

                /// Returns a mutable reference to the last element.
                ///
                /// As `$name` always contains at least one element there is always a last element.
                pub fn last_mut(&mut self) -> &mut $item_ty {
                    //UNWRAP_SAFE: len is at least 1
                    self.0.last_mut().unwrap()
                }

                /// Returns a reference to the first element.
                ///
                /// As `$name` always contains at least one element there is always a first element.
                pub fn first(&self) -> &$item_ty {
                    //UNWRAP_SAFE: len is at least 1
                    self.0.first().unwrap()
                }

                /// Returns a mutable reference to the first element.
                ///
                /// As `$name` always contains at least one element there is always a first element.
                pub fn first_mut(&mut self) -> &mut $item_ty {
                    //UNWRAP_SAFE: len is at least 1
                    self.0.first_mut().unwrap()
                }


                /// Truncates the `SmalVec1` to given length.
                ///
                /// # Errors
                ///
                /// If len is 0 an error is returned as the
                /// length >= 1 constraint must be uphold.
                ///
                pub fn truncate(&mut self, len: usize) -> Result<(), Size0Error> {
                    if len > 0 {
                        self.0.truncate(len);
                        Ok(())
                    } else {
                        Err(Size0Error)
                    }
                }

                /// Truncates the `SmalVec1` to given length.
                ///
                /// # Errors
                ///
                /// If len is 0 an error is returned as the
                /// length >= 1 constraint must be uphold.
                ///
                #[deprecated(
                    since = "1.8.0",
                    note = "try_ prefix created ambiguity use `truncate`"
                )]
                #[inline(always)]
                pub fn try_truncate(&mut self, len: usize) -> Result<(), Size0Error> {
                    self.truncate(len)
                }

                /// Calls `swap_remove` on the inner smallvec if length >= 2.
                ///
                /// # Errors
                ///
                /// If len is 1 an error is returned as the
                /// length >= 1 constraint must be uphold.
                pub fn swap_remove(&mut self, index: usize) -> Result<$item_ty, Size0Error> {
                    if self.len() > 1 {
                        Ok(self.0.swap_remove(index))
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
                #[deprecated(
                    since = "1.8.0",
                    note = "try_ prefix created ambiguity use `swap_remove`"
                )]
                #[inline(always)]
                pub fn try_swap_remove(&mut self, index: usize) -> Result<$item_ty, Size0Error> {
                    self.swap_remove(index)
                }

                /// Calls `remove` on the inner smallvec if length >= 2.
                ///
                /// # Errors
                ///
                /// If len is 1 an error is returned as the
                /// length >= 1 constraint must be uphold.
                pub fn remove(&mut self, index: usize) -> Result<$item_ty, Size0Error> {
                    if self.len() > 1 {
                        Ok(self.0.remove(index))
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
                ///
                /// # Panics
                ///
                /// If `index` is greater or equal then `len`.
                #[deprecated(
                    since = "1.8.0",
                    note = "try_ prefix created ambiguity use `remove`, also try_remove PANICS on out of bounds"
                )]
                #[inline(always)]
                pub fn try_remove(&mut self, index: usize) -> Result<$item_ty, Size0Error> {
                    self.remove(index)
                }

                /// Removes all elements except the ones which the predicate says need to be retained.
                ///
                /// The moment the last element would be removed this will instead fail, not removing
                /// the element. **All but the last element will have been removed anyway.**
                ///
                /// # Error
                ///
                /// If the last element would be removed instead of removing it a `Size0Error` is
                /// returned.
                ///
                /// # Example
                ///
                /// Is for `Vec1` but similar code works with `SmallVec1`, too.
                ///
                /// ```
                /// # use vec1::vec1;
                ///
                /// let mut vec = vec1![1, 7, 8, 9, 10];
                /// vec.retain(|v| *v % 2 == 1).unwrap();
                /// assert_eq!(vec, vec1![1, 7, 9]);
                /// let Size0Error = vec.retain(|_| false).unwrap_err();
                /// assert_eq!(vec.len(), 1);
                /// assert_eq!(vec.last(), &9);
                /// ```
                pub fn retain<F>(&mut self, mut f: F) -> Result<(), Size0Error>
                where
                    F: FnMut(&$item_ty) -> bool
                {
                    // code is based on the code in the standard library,
                    // given a local instal of rust v1.50.0 source documentation in rustup:
                    // <path-to-rustup-rust-v1.50.0-toolchain-with-source-doc>/share/doc/rust/html/src/alloc/vec.rs.html#1314-1334
                    let len = self.len();
                    let mut del = 0;
                    {
                        let v = &mut **self;

                        for i in 0..len {
                            if !f(&v[i]) {
                                del += 1;
                            } else if del > 0 {
                                v.swap(i - del, i);
                            }
                        }
                    }
                    if del == 0 {
                        Ok(())
                    } else {
                        if del < len {
                            self.0.truncate(len - del);
                            Ok(())
                        } else {
                            // if we would delete all then:
                            // del == len AND no swap was done
                            // so retain only last and return error
                            self.swap(0, len - 1);
                            self.0.truncate(1);
                            Err(Size0Error)
                        }
                    }
                }

                /// Calls `dedup_by_key` on the inner smallvec.
                ///
                /// While this can remove elements it will
                /// never produce a empty vector from an non
                /// empty vector.
                pub fn dedup_by_key<F, K>(&mut self, key: F)
                where
                    F: FnMut(&mut $item_ty) -> K,
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
                    F: FnMut(&mut $item_ty, &mut $item_ty) -> bool,
                {
                    self.0.dedup_by(same_bucket)
                }

                /// Remove the last element from this vector, if there is more than one element in it.
                ///
                /// # Errors
                ///
                /// If len is 1 an error is returned as the
                /// length >= 1 constraint must be uphold.
                pub fn pop(&mut self) -> Result<$item_ty, Size0Error> {
                    if self.len() > 1 {
                        //UNWRAP_SAFE: pop on len > 1 can not be none
                        Ok(self.0.pop().unwrap())
                    } else {
                        Err(Size0Error)
                    }
                }

                /// Remove the last element from this vector, if there is more than one element in it.
                ///
                /// # Errors
                ///
                /// If len is 1 an error is returned as the
                /// length >= 1 constraint must be uphold.
                #[deprecated(
                    since = "1.8.0",
                    note = "try_ prefix created ambiguity use `pop`"
                )]
                #[inline(always)]
                pub fn try_pop(&mut self) -> Result<$item_ty, Size0Error> {
                    self.pop()
                }

                /// See [`Vec::resize_with()`] but fails if it would resize to length 0.
                pub fn resize_with<F>(&mut self, new_len: usize, f: F) -> Result<(), Size0Error>
                where
                    F: FnMut() -> $item_ty
                {
                    if new_len > 0 {
                        self.0.resize_with(new_len, f);
                        Ok(())
                    } else {
                        Err(Size0Error)
                    }
                }

                /// See [`Vec::resize_with()`] but fails if it would resize to length 0.
                #[deprecated(
                    since = "1.8.0",
                    note = "try_ prefix created ambiguity use `resize_with`"
                )]
                #[inline(always)]
                pub fn try_resize_with<F>(&mut self, new_len: usize, f: F) -> Result<(), Size0Error>
                where
                    F: FnMut() -> $item_ty
                {
                    self.resize_with(new_len, f)
                }

                /// Splits off the first element of this vector and returns it together with the rest of the
                /// vector.
                ///
                pub fn split_off_first(self) -> ($item_ty, $wrapped<$t>) {
                    let mut smallvec = self.0;
                    let first = smallvec.remove(0);
                    (first, smallvec)
                }

                /// Splits off the last element of this vector and returns it together with the rest of the
                /// vector.
                pub fn split_off_last(self) -> ($wrapped<$t>, $item_ty) {
                    let mut smallvec = self.0;
                    let last = smallvec.remove(smallvec.len() - 1);
                    (smallvec, last)
                }

                /// Turns this vector into a boxed slice.
                ///
                /// For `Vec1` this is as cheap as for `Vec` but for
                /// `SmallVec1` this will cause an allocation if the
                /// on-stack buffer was not yet spilled.
                pub fn into_boxed_slice(self) -> Box<[$item_ty]> {
                    self.into_vec().into_boxed_slice()
                }

                /// Leaks the allocation to return a mutable slice reference.
                ///
                /// This is equivalent to turning this vector into a boxed
                /// slice and then leaking that slice.
                ///
                /// In case of `SmallVec1` calling leak does entail an allocation
                /// if the stack-buffer had not yet spilled.
                pub fn leak<'a>(self) -> &'a mut [$item_ty]
                where
                    $item_ty: 'a
                {
                    self.into_vec().leak()
                }

            }

            // methods in Vec not in &[] which can be directly exposed
            impl_wrapper! {
                base_bounds_macro = $($tb : $trait)?,
                impl<$t> $name<$t> {
                    fn append(&mut self, other: &mut $wrapped<$t>) -> ();
                    fn reserve(&mut self, additional: usize) -> ();
                    fn reserve_exact(&mut self, additional: usize) -> ();
                    fn shrink_to_fit(&mut self) -> ();
                    fn as_mut_slice(&mut self) -> &mut [$item_ty];
                    fn push(&mut self, value: $item_ty) -> ();
                    fn insert(&mut self, idx: usize, val: $item_ty) -> ();
                    fn len(&self) -> usize;
                    fn capacity(&self) -> usize;
                    fn as_slice(&self) -> &[$item_ty]
                }
            }

            impl<$t> $name<$t>
            where
                $item_ty: PartialEq<$item_ty>,
                $($tb : $trait,)?
            {
                pub fn dedup(&mut self) {
                    self.0.dedup()
                }
            }

            impl<$t> $name<$t>
            where
                $item_ty: Copy,
                $($tb : $trait,)?
            {
                pub fn extend_from_slice(&mut self, slice: &[$item_ty]) {
                    self.0.extend_from_slice(slice)
                }
            }

            impl<$t> $name<$t>
            where
                $item_ty: Clone,
                $($tb : $trait,)?
            {
                /// See [`Vec::resize()`] but fails if it would resize to length 0.
                pub fn resize(&mut self, len: usize, value: $item_ty) -> Result<(), Size0Error> {
                    if len == 0 {
                        Err(Size0Error)
                    } else {
                        self.0.resize(len, value);
                        Ok(())
                    }
                }

                /// See [`Vec::resize()`] but fails if it would resize to length 0.
                #[deprecated(
                    since = "1.8.0",
                    note = "try_ prefix created ambiguity use `resize_with`"
                )]
                #[inline(always)]
                pub fn try_resize(&mut self, len: usize, value: $item_ty) -> Result<(), Size0Error> {
                    self.resize(len, value)
                }
            }

            impl<$t> From<$name<$t>> for $wrapped<$t>
            where
                $($tb : $trait,)?
            {
                fn from(vec: $name<$t>) -> $wrapped<$t> {
                    vec.0
                }
            }


            impl<$t> TryFrom<$wrapped<$t>> for $name<$t>
            where
                $($tb : $trait,)?
            {
                type Error = Size0Error;
                fn try_from(vec: $wrapped<$t>) -> Result<Self, Size0Error> {
                    if vec.is_empty() {
                        Err(Size0Error)
                    } else {
                        Ok(Self(vec))
                    }
                }
            }


            impl<$t> TryFrom<&'_ [$item_ty]> for $name<$t>
            where
                $item_ty: Clone,
                $($tb : $trait,)?
            {
                type Error = Size0Error;
                fn try_from(slice: &'_ [$item_ty]) -> Result<Self, Size0Error> {
                    if slice.is_empty() {
                        Err(Size0Error)
                    } else {
                        Ok($name($wrapped::from(slice)))
                    }
                }
            }

            impl<$t> TryFrom<Box<[$item_ty]>> for $name<$t>
            where
                $($tb : $trait,)?
            {
                type Error = Size0Error;
                fn try_from(slice: Box<[$item_ty]>) -> Result<Self, Size0Error> {
                    if slice.is_empty() {
                        Err(Size0Error)
                    } else {
                        let vec = Vec::from(slice);
                        Self::try_from_vec(vec)
                    }
                }
            }

            impl<$t> Debug for $name<$t>
            where
                $item_ty: Debug,
                $($tb : $trait,)?
            {
                #[inline]
                fn fmt(&self, fter: &mut fmt::Formatter) -> fmt::Result {
                    Debug::fmt(&self.0, fter)
                }
            }

            impl<$t> Clone for $name<$t>
            where
                $item_ty: Clone,
                $($tb : $trait,)?
            {
                #[inline]
                fn clone(&self) -> Self {
                    $name(self.0.clone())
                }
            }

            impl<$t, B> PartialEq<B> for $name<$t>
            where
                B: ?Sized,
                $wrapped<$t>: PartialEq<B>,
                $($tb : $trait,)?
            {
                #[inline]
                fn eq(&self, other: &B) -> bool {
                    self.0.eq(other)
                }
            }

            impl<$t> Eq for $name<$t>
            where
                $item_ty: Eq,
                $($tb : $trait,)?
            {}

            impl<$t> Hash for $name<$t>
            where
                $item_ty: Hash,
                $($tb : $trait,)?
            {
                #[inline]
                fn hash<H: Hasher>(&self, state: &mut H) {
                    self.0.hash(state)
                }
            }

            impl<$t> PartialOrd for $name<$t>
            where
                $item_ty: PartialOrd,
                $($tb : $trait,)?
            {
                #[inline]
                fn partial_cmp(&self, other: &$name<$t>) -> Option<Ordering> {
                    self.0.partial_cmp(&other.0)
                }
            }

            impl<$t> Ord for $name<$t>
            where
                $item_ty: Ord,
                $($tb : $trait,)?
            {
                #[inline]
                fn cmp(&self, other: &$name<$t>) -> Ordering {
                    self.0.cmp(&other.0)
                }
            }

            impl<$t> Deref for $name<$t>
            where
                $($tb : $trait,)?
            {
                type Target = [$item_ty];

                fn deref(&self) -> &Self::Target {
                    &*self.0
                }
            }

            impl<$t> DerefMut for $name<$t>
            where
                $($tb : $trait,)?
            {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut *self.0
                }
            }

            impl<'a, $t> IntoIterator for &'a $name<$t>
            where
                $($tb : $trait,)?
            {
                type Item = &'a $item_ty;
                type IntoIter = core::slice::Iter<'a, $item_ty>;

                fn into_iter(self) -> Self::IntoIter {
                    (&self.0).into_iter()
                }
            }

            impl<'a, $t> IntoIterator for &'a mut $name<$t>
            where
                $($tb : $trait,)?
            {
                type Item = &'a mut $item_ty;
                type IntoIter = core::slice::IterMut<'a, $item_ty>;

                fn into_iter(self) -> Self::IntoIter {
                    (&mut self.0).into_iter()
                }
            }

            impl<$t> Default for $name<$t>
            where
                $item_ty: Default,
                $($tb : $trait,)?
            {
                fn default() -> Self {
                    $name::new(Default::default())
                }
            }

            impl<$t> AsRef<[$item_ty]> for $name<$t>
            where
                $($tb : $trait,)?
            {
                fn as_ref(&self) -> &[$item_ty] {
                    self.0.as_ref()
                }
            }

            impl<$t> AsMut<[$item_ty]> for $name<$t>
            where
                $($tb : $trait,)?
            {
                fn as_mut(&mut self) -> &mut [$item_ty] {
                    self.0.as_mut()
                }
            }

            impl<$t> AsRef<$wrapped<$t>> for $name<$t>
            where
                $($tb : $trait,)?
            {
                fn as_ref(&self) -> &$wrapped<$t>{
                    &self.0
                }
            }

            impl<$t> AsRef<$name<$t>> for $name<$t>
            where
                $($tb : $trait,)?
            {
                fn as_ref(&self) -> &$name<$t> {
                    self
                }
            }

            impl<$t> AsMut<$name<$t>> for $name<$t>
            where
                $($tb : $trait,)?
            {
                fn as_mut(&mut self) -> &mut $name<$t> {
                    self
                }
            }



            impl<$t> Borrow<[$item_ty]> for $name<$t>
            where
                $($tb : $trait,)?
            {
                fn borrow(&self) -> &[$item_ty] {
                    self.0.as_ref()
                }
            }


            impl<$t> Borrow<$wrapped<$t>> for $name<$t>
            where
                $($tb : $trait,)?
            {
                fn borrow(&self) -> &$wrapped<$t>{
                    &self.0
                }
            }

            impl<$t, SI> Index<SI> for $name<$t>
            where
                SI: SliceIndex<[$item_ty]>,
                $($tb : $trait,)?
            {
                type Output = SI::Output;

                fn index(&self, index: SI) -> &SI::Output {
                    self.0.index(index)
                }
            }

            impl<$t, SI> IndexMut<SI> for $name<$t>
            where
                SI: SliceIndex<[$item_ty]>,
                $($tb : $trait,)?
            {
                fn index_mut(&mut self, index: SI) -> &mut SI::Output {
                    self.0.index_mut(index)
                }
            }


            impl<$t> BorrowMut<[$item_ty]> for $name<$t>
            where
                $($tb : $trait,)?
            {
                fn borrow_mut(&mut self) -> &mut [$item_ty] {
                    self.0.as_mut()
                }
            }

            impl<$t> Extend<$item_ty> for $name<$t>
            where
                $($tb : $trait,)?
            {
                fn extend<IT: IntoIterator<Item = $item_ty>>(&mut self, iterable: IT) {
                    self.0.extend(iterable)
                }
            }

            //Note: We can not (simply) have if feature serde and feature smallvec enable
            //      dependency smallvec/serde, but we can mirror the serde implementation.
            #[cfg(feature = "serde")]
            const _: () = {
                use core::marker::PhantomData;
                use serde::{
                    de::{SeqAccess,Deserialize, Visitor, Deserializer, Error as _},
                    ser::{Serialize, Serializer, SerializeSeq}
                };

                impl<$t> Serialize for $name<$t>
                where
                    $item_ty: Serialize,
                    $($tb : $trait,)?
                {
                    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                        let mut seq_ser = serializer.serialize_seq(Some(self.len()))?;
                        for item in self {
                            seq_ser.serialize_element(&item)?;
                        }
                        seq_ser.end()
                    }
                }

                impl<'de, $t> Deserialize<'de> for $name<$t>
                where
                    $item_ty: Deserialize<'de>,
                    $($tb : $trait,)?
                {
                    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                        deserializer.deserialize_seq(SmallVec1Visitor {
                            _type_carry: PhantomData,
                        })
                    }
                }
                struct SmallVec1Visitor<$t> {
                    _type_carry: PhantomData<$t>,
                }

                impl<'de, $t> Visitor<'de> for SmallVec1Visitor<$t>
                where
                    $item_ty: Deserialize<'de>,
                    $($tb : $trait,)?
                {
                    type Value = $name<$t>;

                    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                        formatter.write_str("a sequence")
                    }

                    fn visit_seq<B>(self, mut seq: B) -> Result<Self::Value, B::Error>
                    where
                        B: SeqAccess<'de>,
                    {
                        let len = seq.size_hint().unwrap_or(0);
                        let mut vec = $wrapped::new();
                        //FIXME use try_reserve
                        vec.reserve(len);

                        while let Some(value) = seq.next_element()? {
                            vec.push(value);
                        }

                        $name::try_from(vec).map_err(B::Error::custom)
                    }
                }
            };
        // };
    );
}
