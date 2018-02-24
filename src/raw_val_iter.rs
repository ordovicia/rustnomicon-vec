use std::mem;
use std::ptr;

pub(super) struct RawValIter<T> {
    start: *const T,
    end: *const T,
}

impl<T> RawValIter<T> {
    // unsafe to construct because it has no associated lifetimes.
    // This is necessary to store a RawValIter in the same struct as
    // its actual allocation. OK since it's a private implementation
    // detail.
    pub(super) unsafe fn new(slice: &[T]) -> Self {
        let start = slice.as_ptr();

        RawValIter {
            start,
            end: if mem::size_of::<T>() == 0 {
                ((start as usize) + slice.len()) as *const _
            } else if slice.is_empty() {
                start
            } else {
                start.offset(slice.len() as isize)
            },
        }
    }
}

impl<T> Iterator for RawValIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let result = ptr::read(self.start);
                self.start = if mem::size_of::<T>() == 0 {
                    (self.start as usize + 1) as *const _
                } else {
                    self.start.offset(1)
                };

                Some(result)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let elem_size = mem::size_of::<T>();
        let len =
            (self.end as usize - self.start as usize) / if elem_size == 0 { 1 } else { elem_size };
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for RawValIter<T> {
    fn next_back(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                self.end = if mem::size_of::<T>() == 0 {
                    (self.end as usize - 1) as *const _
                } else {
                    self.end.offset(-1)
                };

                Some(ptr::read(self.end))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next() {
        let mut iter = unsafe { RawValIter::new(&[0, 1]) };

        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn size_hint() {
        {
            let iter: RawValIter<i32> = unsafe { RawValIter::new(&[]) };
            assert_eq!(iter.size_hint(), (0, Some(0)));
        }

        {
            let iter: RawValIter<i32> = unsafe { RawValIter::new(&[0, 1]) };
            assert_eq!(iter.size_hint(), (2, Some(2)));
        }
    }

    #[test]
    fn next_back() {
        let mut iter = unsafe { RawValIter::new(&[0, 1]) };

        assert_eq!(iter.next_back(), Some(1));
        assert_eq!(iter.next_back(), Some(0));
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn next_next_back() {
        let mut iter = unsafe { RawValIter::new(&[0, 1, 2, 3]) };

        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next_back(), Some(3));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next_back(), Some(2));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }
}
