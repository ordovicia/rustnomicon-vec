use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;

use raw_vec::RawVec;
use raw_val_iter::RawValIter;
use into_iter::IntoIter;
use drain::Drain;

pub struct Vec<T> {
    buf: RawVec<T>,
    len: usize,
}

impl<T> Deref for Vec<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe { ::std::slice::from_raw_parts(self.ptr(), self.len) }
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { ::std::slice::from_raw_parts_mut(self.ptr(), self.len) }
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        if mem::needs_drop::<T>() {
            while let Some(_) = self.pop() {}
        }

        // deallocation is handled by RawVec
    }
}

impl<T> Vec<T> {
    /// Create a new `Vec` with no elements.
    pub fn default() -> Self {
        Vec {
            buf: RawVec::default(),
            len: 0,
        }
    }

    /// Returns capacity.
    pub fn capacity(&self) -> usize {
        self.buf.cap
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Appends an element to the last position.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate nomicon_vec;
    ///
    /// let mut v = nomicon_vec::vec::Vec::default();
    ///
    /// v.push(0);
    /// assert_eq!(v.len(), 1);
    ///
    /// v.push(1);
    /// assert_eq!(v.len(), 2);
    /// ```
    pub fn push(&mut self, elem: T) {
        if self.len == self.capacity() {
            self.buf.grow();
        }

        unsafe {
            let ptr_last = self.ptr().offset(self.len as isize);
            ptr::write(ptr_last, elem);
        }

        self.len += 1;
    }

    /// Removes and returns an element from the last position.
    /// Returns `None` if no elements are stored.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate nomicon_vec;
    ///
    /// let mut v = nomicon_vec::vec::Vec::default();
    /// assert!(v.pop().is_none());
    ///
    /// v.push(0);
    /// assert_eq!(v.pop(), Some(0));
    ///
    /// v.push(0);
    /// v.push(1);
    /// assert_eq!(v.pop(), Some(1));
    /// assert_eq!(v.pop(), Some(0));
    /// assert_eq!(v.len(), 0);
    ///
    /// assert!(v.pop().is_none());
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe {
                let ptr_last = self.ptr().offset(self.len as isize);
                Some(ptr::read(ptr_last))
            }
        }
    }

    /// Inserts an element to a target index.
    /// Elements at the target index are shifted to right by one.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate nomicon_vec;
    ///
    /// let mut v = nomicon_vec::vec::Vec::default();
    /// v.push(0);
    /// v.push(1);
    /// v.insert(1, 2);
    ///
    /// assert_eq!(v.len(), 3);
    /// assert_eq!(v.get(0), Some(&0));
    /// assert_eq!(v.get(1), Some(&2));
    /// assert_eq!(v.get(2), Some(&1));
    /// ```
    pub fn insert(&mut self, index: usize, elem: T) {
        assert!(index <= self.len, "index out of bounds");

        if self.len == self.capacity() {
            self.buf.grow();
        }

        unsafe {
            if index < self.len {
                // ptr::copy(src, dest, len): "copy from source to dest len elems"
                ptr::copy(
                    self.ptr().offset(index as isize),
                    self.ptr().offset(index as isize + 1),
                    self.len - index,
                );
            }

            ptr::write(self.ptr().offset(index as isize), elem);
        }

        self.len += 1;
    }

    /// Removes an element at a target index.
    /// Elements at the target index are shifted to left by one.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate nomicon_vec;
    ///
    /// let mut v = nomicon_vec::vec::Vec::default();
    /// v.push(0);
    /// v.push(1);
    /// v.push(2);
    ///
    /// assert_eq!(v.remove(1), 1);
    /// assert_eq!(v.len(), 2);
    /// assert_eq!(v.get(0), Some(&0));
    /// assert_eq!(v.get(1), Some(&2));
    /// ```
    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "index out of bounds");

        self.len -= 1;

        unsafe {
            let result = ptr::read(self.ptr().offset(index as isize));
            ptr::copy(
                self.ptr().offset(index as isize + 1),
                self.ptr().offset(index as isize),
                self.len - index,
            );
            result
        }
    }

    /// Creates an [`IntoIter`] instance from self.
    ///
    /// [`IntoIter`]: ../into_iter/struct.IntoIter.html
    pub fn into_iter(self) -> IntoIter<T> {
        unsafe {
            let iter = RawValIter::new(&self);

            let buf = ptr::read(&self.buf);
            mem::forget(self);

            IntoIter::new(buf, iter)
        }
    }

    pub fn drain(&mut self, start: usize) -> Drain<T> {
        assert!(start < self.len);

        unsafe {
            let iter = RawValIter::new(&self[start..]);
            self.len = start;
            Drain::new(iter)
        }
    }

    fn ptr(&self) -> *mut T {
        self.buf.ptr.as_ptr()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zst() {
        let mut v = Vec::default();
        assert_eq!(v.len(), 0);

        v.push(());
        assert_eq!(v.len(), 1);
        assert_eq!(v.pop(), Some(()));

        v.insert(0, ());
        assert_eq!(v.len(), 1);
        assert_eq!(v.remove(0), ());
    }

    #[test]
    #[should_panic]
    fn insert_panic_0() {
        let mut v: Vec<i32> = Vec::default();
        v.insert(1, 0);
    }

    #[test]
    #[should_panic]
    fn insert_panic_1() {
        let mut v: Vec<i32> = Vec::default();
        v.push(0);
        v.insert(2, 0);
    }

    #[test]
    #[should_panic]
    fn remove_panic_0() {
        let mut v: Vec<i32> = Vec::default();
        v.remove(0);
    }

    #[test]
    #[should_panic]
    fn remove_panic_1() {
        let mut v: Vec<i32> = Vec::default();
        v.push(0);
        v.remove(1);
    }

    #[test]
    fn deref_slice() {
        let mut v: Vec<i32> = Vec::default();
        assert!(v.is_empty());

        v.push(0);
        assert_eq!(v.first(), Some(&0));
    }

    #[test]
    fn deref_mut_slice() {
        let mut v: Vec<i32> = Vec::default();

        v.push(0);
        v.push(1);
        v.reverse();

        assert_eq!(v.pop(), Some(0));
        assert_eq!(v.pop(), Some(1));
    }
}
