#![allow(non_snake_case)]
#![allow(unused)]

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
        //TODO impl, also for smallvec?
        // let a = vec1![32u8, 12u8];
        // let a: Box<[u8]> = a.into_boxed_slice();
        // assert_eq!(a, &[32u8, 12u8]);
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
    fn drain() {
        // let mut a = vec1![1u8, 2, 4, 12, 88, 3, 2];
        // a.drain(1..);
        //
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
            a.splice(3..2, vec1![32u8]);
        });
        assert!(res.is_err());

        let res = catch_unwind(|| {
            let mut a = vec1![1u8, 2, 3, 4];
            a.splice(..100, vec1![32u8]);
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
        use std::{borrow::{ToOwned, Cow}, convert::TryFrom};
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

    mod Write {
        use std::io::Write;

        #[test]
        fn for_bytes() {
            let mut v = vec1![1u8];
            v.write(&[65, 100, 12]).unwrap();
            assert_eq!(v, &[1u8, 65, 100, 12]);
        }
    }
}

mod Cow {

    mod From {
        use std::borrow::{Cow, ToOwned};
        use crate::*;

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
        use std::borrow::Cow;

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
        use std::{ffi::CString, num::NonZeroU8};

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
        use std::collections::BinaryHeap;

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
        use alloc::collections::VecDeque;

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