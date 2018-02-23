use std::marker::PhantomData;

use raw_val_iter::RawValIter;

pub struct Drain<'a, T: 'a> {
    _vec: PhantomData<&'a mut Vec<T>>,
    iter: RawValIter<T>,
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for Drain<'a, T> {
    fn next_back(&mut self) -> Option<T> {
        self.iter.next_back()
    }
}

impl<'a, T> Drop for Drain<'a, T> {
    fn drop(&mut self) {
        for _ in &mut self.iter {}
    }
}

impl<'a, T> Drain<'a, T> {
    pub(super) fn new(iter: RawValIter<T>) -> Self {
        Drain {
            _vec: PhantomData,
            iter,
        }
    }
}

#[cfg(test)]
mod tests {
    use vec::Vec;

    #[test]
    fn start_0() {
        let mut v = Vec::default();
        v.push(0);
        v.push(1);

        {
            let mut drain = v.drain(0);
            assert_eq!(drain.next(), Some(0));
            assert_eq!(drain.next(), Some(1));
            assert_eq!(drain.next(), None);
        }

        assert_eq!(v.len(), 0);
    }

    #[test]
    fn start_0_back() {
        let mut v = Vec::default();
        v.push(0);
        v.push(1);

        {
            let mut drain = v.drain(0);
            assert_eq!(drain.next_back(), Some(1));
            assert_eq!(drain.next_back(), Some(0));
            assert_eq!(drain.next_back(), None);
        }

        assert_eq!(v.len(), 0);
    }

    #[test]
    fn start_1() {
        let mut v = Vec::default();
        v.push(0);
        v.push(1);

        {
            let mut drain = v.drain(1);
            assert_eq!(drain.next(), Some(1));
            assert_eq!(drain.next(), None);
        }

        assert_eq!(v.len(), 1);

        let mut iter = v.into_iter();
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn start_1_back() {
        let mut v = Vec::default();
        v.push(0);
        v.push(1);

        {
            let mut drain = v.drain(1);
            assert_eq!(drain.next_back(), Some(1));
            assert_eq!(drain.next_back(), None);
        }

        assert_eq!(v.len(), 1);

        let mut iter = v.into_iter();
        assert_eq!(iter.next_back(), Some(0));
        assert_eq!(iter.next_back(), None);
    }
}
