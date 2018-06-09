#![feature(test)]
extern crate test;

use std::slice::from_raw_parts;
use std::marker::PhantomData;

// Thank you Yorick !
pub fn group_by_equality<T: Eq>(slice: &[T]) -> impl Iterator<Item=&[T]> {
    GroupBy::new(slice, PartialEq::eq)
}

pub struct GroupBy<'a, T: 'a, P> {
    ptr: *const T,
    len: usize,
    predicate: P,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T: 'a, P> GroupBy<'a, T, P>
where P: FnMut(&T, &T) -> bool,
{
    pub fn new(slice: &'a [T], predicate: P) -> Self {
        Self {
            ptr: slice.as_ptr(),
            len: slice.len(),
            predicate: predicate,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: 'a, P> Iterator for GroupBy<'a, T, P>
where P: FnMut(&T, &T) -> bool,
{
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.len == 0 { return None }

            for i in 0..self.len.saturating_sub(1) {
                let a = &*self.ptr.add(i);
                let b = &*self.ptr.add(i + 1);

                if !(self.predicate)(a, b) {
                    let slice = from_raw_parts(self.ptr, i + 1);

                    self.ptr = self.ptr.add(i + 1);
                    self.len = self.len - (i + 1);

                    return Some(slice)
                }
            }

            let slice = from_raw_parts(self.ptr, self.len);
            self.len = 0;
            Some(slice)
        }
    }

    fn last(mut self) -> Option<Self::Item> {
        unsafe {
            if self.len == 0 { return None }

            for i in (0..self.len.saturating_sub(1)).rev() {
                let a = &*self.ptr.add(i);
                let b = &*self.ptr.add(i + 1);

                if !(self.predicate)(a, b) {
                    let len = self.len - (i + 1);
                    let slice = from_raw_parts(self.ptr.add(i + 1), len);
                    return Some(slice)
                }
            }

            let slice = from_raw_parts(self.ptr, self.len);
            Some(slice)
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;
    use super::*;

    #[derive(Debug, Eq)]
    enum Guard {
        Valid,
        Invalid,
    }

    impl PartialEq for Guard {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Guard::Valid, Guard::Valid) => true,
                _ => panic!("denied read on Guard::Invalid variant")
            }
        }
    }

    #[test]
    fn empty_slice() {
        let slice: &[i32] = &[];

        let mut iter = group_by_equality(slice);

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn one_little_group() {
        let slice = &[1];

        let mut iter = group_by_equality(slice);

        assert_eq!(iter.next(), Some(&[1][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn one_big_group() {
        let slice = &[1, 1, 1, 1];

        let mut iter = GroupBy::new(slice, |a, b| a == b);

        assert_eq!(iter.next(), Some(&[1, 1, 1, 1][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn two_equal_groups() {
        let slice = &[1, 1, 1, 1, 2, 2, 2, 2];

        let mut iter = GroupBy::new(slice, |a, b| a == b);

        assert_eq!(iter.next(), Some(&[1, 1, 1, 1][..]));
        assert_eq!(iter.next(), Some(&[2, 2, 2, 2][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn two_little_equal_groups() {
        let slice = &[1, 2];

        let mut iter = GroupBy::new(slice, |a, b| a == b);

        assert_eq!(iter.next(), Some(&[1][..]));
        assert_eq!(iter.next(), Some(&[2][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn three_groups() {
        let slice = &[1, 1, 1, 3, 3, 2, 2, 2];

        let mut iter = GroupBy::new(slice, |a, b| a == b);

        assert_eq!(iter.next(), Some(&[1, 1, 1][..]));
        assert_eq!(iter.next(), Some(&[3, 3][..]));
        assert_eq!(iter.next(), Some(&[2, 2, 2][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn three_little_groups() {
        let slice = &[1, 3, 2];

        let mut iter = GroupBy::new(slice, |a, b| a == b);

        assert_eq!(iter.next(), Some(&[1][..]));
        assert_eq!(iter.next(), Some(&[3][..]));
        assert_eq!(iter.next(), Some(&[2][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn overflow() {
        let slice = &[Guard::Valid, Guard::Valid, Guard::Invalid];

        let mut iter = GroupBy::new(&slice[0..2], |a, b| a == b);

        assert_eq!(iter.next(), Some(&[Guard::Valid, Guard::Valid][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn last_three_little_groups() {
        let slice = &[1, 3, 2];

        let iter = GroupBy::new(slice, |a, b| a == b);

        assert_eq!(iter.last(), Some(&[2][..]));
    }

    #[test]
    fn last_three_groups() {
        let slice = &[1, 1, 1, 3, 3, 2, 2, 2];

        let iter = GroupBy::new(slice, |a, b| a == b);

        assert_eq!(iter.last(), Some(&[2, 2, 2][..]));
    }

    #[test]
    fn last_overflow() {
        let slice = &[Guard::Valid, Guard::Valid, Guard::Invalid];

        let iter = GroupBy::new(&slice[0..2], |a, b| a == b);

        assert_eq!(iter.last(), Some(&[Guard::Valid, Guard::Valid][..]));
    }

    #[bench]
    fn vector_16_000(b: &mut test::Bencher) {
        use self::rand::{Rng, SeedableRng};
        use self::rand::rngs::StdRng;
        use self::rand::distributions::Alphanumeric;

        let mut rng = StdRng::from_seed([42; 32]);

        let len = 16_000;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(rng.sample(Alphanumeric));
        }

        b.iter(|| {
            let group_by = GroupBy::new(vec.as_slice(), |a, b| a == b);
            test::black_box(group_by.for_each(drop))
        })
    }

}
