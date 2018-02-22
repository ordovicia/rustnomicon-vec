use std::ptr;
use std::ops::{Deref, DerefMut};
use std::mem;
use std::heap::{Alloc, Heap};

use owned_ptr::OwnedPtr;
use into_iter::IntoIter;

pub struct Vec<T> {
    ptr: OwnedPtr<T>,
    cap: usize,
    len: usize,
    alloc: Heap,
}

impl<T> Deref for Vec<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe { ::std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { ::std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        if self.cap == 0 {
            return;
        }

        if mem::needs_drop::<T>() {
            while let Some(_) = self.pop() {}
        }

        unsafe {
            if self.cap == 1 {
                self.alloc.dealloc_one(self.ptr.as_non_null());
            } else {
                let e = self.alloc.dealloc_array(self.ptr.as_non_null(), self.cap);
                if let Err(e) = e {
                    self.alloc.oom(e);
                }
            }
        }
    }
}

impl<T> Vec<T> {
    pub fn new() -> Self {
        assert!(mem::size_of::<T>() != 0, "We're not ready to handle ZSTs");

        Vec {
            ptr: OwnedPtr::empty(),
            cap: 0,
            len: 0,
            alloc: Heap,
        }
    }

    /// Stores an element to the last position.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate nomicon_vec;
    ///
    /// let mut v = nomicon_vec::vec::Vec::new();
    ///
    /// v.push(0);
    /// assert_eq!(v.len(), 1);
    ///
    /// v.push(1);
    /// assert_eq!(v.len(), 2);
    /// ```
    pub fn push(&mut self, elem: T) {
        if self.len == self.cap {
            self.grow();
        }

        unsafe {
            let ptr_last = self.ptr.as_ptr().offset(self.len as isize);
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
    /// let mut v = nomicon_vec::vec::Vec::new();
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
                let ptr_last = self.ptr.as_ptr().offset(self.len as isize);
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
    /// let mut v = nomicon_vec::vec::Vec::new();
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

        if self.len == self.cap {
            self.grow();
        }

        unsafe {
            if index < self.len {
                // ptr::copy(src, dest, len): "copy from source to dest len elems"
                ptr::copy(
                    self.ptr.as_ptr().offset(index as isize),
                    self.ptr.as_ptr().offset(index as isize + 1),
                    self.len - index,
                );
            }

            ptr::write(self.ptr.as_ptr().offset(index as isize), elem);
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
    /// let mut v = nomicon_vec::vec::Vec::new();
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
            let result = ptr::read(self.ptr.as_ptr().offset(index as isize));
            ptr::copy(
                self.ptr.as_ptr().offset(index as isize + 1),
                self.ptr.as_ptr().offset(index as isize),
                self.len - index,
            );
            result
        }
    }

    /// Creates an [`IntoIter`] instance from self.
    ///
    /// [`IntoIter`]: ../into_iter/struct.IntoIter.html
    pub fn into_iter(self) -> IntoIter<T> {
        let Vec {
            ptr: buf,
            cap,
            len,
            alloc,
        } = self;

        // Make sure not to drop Vec since that will free the buffer
        mem::forget(self);

        IntoIter {
            buf,
            cap,
            start: buf.as_ptr(),
            end: if cap == 0 {
                // can't offset off this pointer, it's not allocated!
                buf.as_ptr()
            } else {
                unsafe { buf.as_ptr().offset(len as isize) }
            },
            alloc,
        }
    }

    fn grow(&mut self) {
        let (ptr, new_cap) = if self.cap == 0 {
            (self.alloc.alloc_one::<T>(), 1)
        } else {
            let elem_size = mem::size_of::<T>();
            let old_num_bytes = self.cap * elem_size;

            assert!(
                old_num_bytes <= (::std::isize::MAX as usize) / 2,
                "capacity overflow"
            );

            unsafe {
                let new_cap = self.cap * 2;
                let ptr = self.alloc
                    .realloc_array::<T>(self.ptr.as_non_null(), self.cap, new_cap);
                (ptr, new_cap)
            }
        };

        if let Err(e) = ptr {
            self.alloc.oom(e);
        }

        self.ptr = OwnedPtr::with_non_null(ptr.unwrap());
        self.cap = new_cap;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn zst_panic() {
        let _: Vec<()> = Vec::new();
    }

    #[test]
    fn grow() {
        let mut v: Vec<i32> = Vec::new();
        assert_eq!(v.cap, 0);

        for cap in (0..16).map(|c| (2 as usize).pow(c)) {
            v.grow();
            assert_eq!(v.cap, cap);
        }
    }

    #[test]
    fn push_pop() {
        let mut v = Vec::new();

        const ELEM_NUM: usize = 32;
        let elems = 0..ELEM_NUM;

        for (i, e) in elems.clone().enumerate() {
            v.push(e);
            assert_eq!(v.len(), i + 1);
        }

        for (i, e) in elems.rev().enumerate() {
            let p = v.pop();
            assert_eq!(p, Some(e));
            assert_eq!(v.len(), ELEM_NUM - 1 - i);
        }

        assert!(v.pop().is_none());
    }

    #[test]
    fn deref_slice() {
        let mut v: Vec<i32> = Vec::new();
        assert!(v.is_empty());

        v.push(0);
        assert_eq!(v.len(), 1);
        assert_eq!(v.first(), Some(&0));
    }

    #[test]
    fn deref_mut_slice() {
        let mut v: Vec<i32> = Vec::new();

        v.push(0);
        v.push(1);
        v.reverse();

        assert_eq!(v.pop(), Some(0));
        assert_eq!(v.pop(), Some(1));
    }
}
