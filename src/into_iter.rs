use std::mem;

use raw_vec::RawVec;
use raw_val_iter::RawValIter;

pub struct IntoIter<T> {
    _buf: RawVec<T>, // we don't actually care abount this. Just need it to live.
    iter: RawValIter<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.iter.next_back()
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        if mem::needs_drop::<T>() {
            for _ in &mut *self {}
        }

        // deallocation is handled by RawVec
    }
}

impl<T> IntoIter<T> {
    pub(super) fn new(buf: RawVec<T>, iter: RawValIter<T>) -> Self {
        IntoIter { _buf: buf, iter }
    }
}

#[cfg(test)]
mod tests {
    use vec::Vec;

    #[test]
    fn next() {
        let mut v = Vec::default();
        v.push(0);
        v.push(1);

        let mut iter = v.into_iter();
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn size_hint() {
        {
            let v: Vec<i32> = Vec::default();
            assert_eq!(v.into_iter().size_hint(), (0, Some(0)));
        }

        {
            let mut v: Vec<i32> = Vec::default();
            v.push(0);
            assert_eq!(v.into_iter().size_hint(), (1, Some(1)));
        }

        {
            let mut v: Vec<i32> = Vec::default();
            v.push(0);
            v.push(1);
            assert_eq!(v.into_iter().size_hint(), (2, Some(2)));
        }
    }

    #[test]
    fn next_back() {
        let mut v = Vec::default();
        v.push(0);
        v.push(1);

        let mut iter = v.into_iter();
        assert_eq!(iter.next_back(), Some(1));
        assert_eq!(iter.next_back(), Some(0));
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn next_next_back() {
        let mut v = Vec::default();
        v.push(0);
        v.push(1);
        v.push(2);

        let mut iter = v.into_iter();
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next_back(), Some(2));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next(), None);
    }
}
