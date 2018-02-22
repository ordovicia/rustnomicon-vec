use std::ptr;
use std::mem;
use std::heap::{Alloc, Heap};

use owned_ptr::OwnedPtr;

pub struct IntoIter<T> {
    pub(super) buf: OwnedPtr<T>,
    pub(super) cap: usize,
    pub(super) start: *const T,
    pub(super) end: *const T,
    pub(super) alloc: Heap,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let result = ptr::read(self.start);
                self.start = self.start.offset(1);
                Some(result)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.end as usize - self.start as usize) / mem::size_of::<T>();
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                self.end = self.end.offset(-1);
                Some(ptr::read(self.end))
            }
        }
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        if self.cap == 0 {
            return;
        }

        if mem::needs_drop::<T>() {
            for _ in &mut *self {}
        }
        unsafe {
            if self.cap == 1 {
                self.alloc.dealloc_one(self.buf.as_non_null());
            } else {
                let e = self.alloc.dealloc_array(self.buf.as_non_null(), self.cap);
                if let Err(e) = e {
                    self.alloc.oom(e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    use vec::Vec;

    #[test]
    fn iter_next() {
        let mut v = Vec::new();
        v.push(0);
        v.push(1);

        let mut iter = v.into_iter();
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
    }

    #[test]
    fn iter_next_back() {
        let mut v = Vec::new();
        v.push(0);
        v.push(1);

        let mut iter = v.into_iter();
        assert_eq!(iter.next_back(), Some(1));
        assert_eq!(iter.next_back(), Some(0));
    }
}
