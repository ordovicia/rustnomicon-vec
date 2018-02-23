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
    fn next() {
        let mut v = Vec::default();
        v.push(0);
        v.push(1);

        let mut drain = v.drain();
        assert_eq!(drain.next(), Some(0));
        assert_eq!(drain.next(), Some(1));
        assert_eq!(drain.next(), None);
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
            assert_eq!(v.drain().size_hint(), (1, Some(1)));
        }

        {
            let mut v: Vec<i32> = Vec::default();
            v.push(0);
            v.push(1);
            assert_eq!(v.drain().size_hint(), (2, Some(2)));
        }
    }

    #[test]
    fn next_back() {
        let mut v = Vec::default();
        v.push(0);
        v.push(1);

        let mut drain = v.drain();
        assert_eq!(drain.next_back(), Some(1));
        assert_eq!(drain.next_back(), Some(0));
        assert_eq!(drain.next_back(), None);
    }

    #[test]
    fn next_next_back() {
        let mut v = Vec::default();
        v.push(0);
        v.push(1);
        v.push(2);

        let mut drain = v.drain();
        assert_eq!(drain.next(), Some(0));
        assert_eq!(drain.next_back(), Some(2));
        assert_eq!(drain.next(), Some(1));
        assert_eq!(drain.next_back(), None);
        assert_eq!(drain.next(), None);
    }
}
