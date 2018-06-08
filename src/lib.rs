use std::mem;

pub struct GroupBy<'a, T: 'a, P> {
    slice: &'a [T],
    predicate: P,
}

impl<'a, T: 'a, P> GroupBy<'a, T, P>
where P: FnMut(&T, &T) -> bool,
{
    pub fn new(slice: &'a [T], predicate: P) -> Self {
        Self { slice, predicate }
    }

    pub fn with_eq(slice: &[T]) -> impl Iterator<Item=&[T]>
    where T: Eq,
    {
        GroupBy { slice, predicate: T::eq }
    }
}

impl<'a, T: 'a, P> Iterator for GroupBy<'a, T, P>
where P: FnMut(&T, &T) -> bool,
{
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.is_empty() { return None }

        for (i, s) in self.slice.windows(2).enumerate() {
            if !(self.predicate)(&s[0], &s[1]) {
                let (left, right) = self.slice.split_at(i + 1);
                self.slice = right;
                return Some(left)
            }
        }

        let old = mem::replace(&mut self.slice, &[]);
        Some(old)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_slice() {
        let slice: &[i32] = &[];

        let mut iter = GroupBy::new(slice, |&a, &b| a == b);

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn one_little_group() {
        let slice: &[i32] = &[1];

        let mut iter = GroupBy::new(slice, |&a, &b| a == b);

        assert_eq!(iter.next(), Some(&[1][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn one_big_group() {
        let slice: &[i32] = &[1, 1, 1, 1];

        let mut iter = GroupBy::new(slice, |&a, &b| a == b);

        assert_eq!(iter.next(), Some(&[1, 1, 1, 1][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn two_equal_groups() {
        let slice: &[i32] = &[1, 1, 1, 1, 2, 2, 2, 2];

        let mut iter = GroupBy::new(slice, |&a, &b| a == b);

        assert_eq!(iter.next(), Some(&[1, 1, 1, 1][..]));
        assert_eq!(iter.next(), Some(&[2, 2, 2, 2][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn two_little_equal_groups() {
        let slice: &[i32] = &[1, 2];

        let mut iter = GroupBy::new(slice, |&a, &b| a == b);

        assert_eq!(iter.next(), Some(&[1][..]));
        assert_eq!(iter.next(), Some(&[2][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn three_groups() {
        let slice: &[i32] = &[1, 1, 1, 3, 3, 2, 2, 2];

        let mut iter = GroupBy::new(slice, |&a, &b| a == b);

        assert_eq!(iter.next(), Some(&[1, 1, 1][..]));
        assert_eq!(iter.next(), Some(&[3, 3][..]));
        assert_eq!(iter.next(), Some(&[2, 2, 2][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn three_little_groups() {
        let slice: &[i32] = &[1, 3, 2];

        let mut iter = GroupBy::new(slice, |&a, &b| a == b);

        assert_eq!(iter.next(), Some(&[1][..]));
        assert_eq!(iter.next(), Some(&[3][..]));
        assert_eq!(iter.next(), Some(&[2][..]));
        assert_eq!(iter.next(), None);
    }

}
