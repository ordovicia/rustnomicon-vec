use std::ptr;
use std::mem;
use std::heap::{Alloc, Heap};

use owned_ptr::OwnedPtr;

pub struct Vec<T> {
    ptr: OwnedPtr<T>,
    cap: usize,
    len: usize,
    alloc: Heap,
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
    /// v.push(0);
    /// ```
    pub fn push(&mut self, elem: T) {
        if self.len == self.cap {
            self.grow();
        }

        unsafe {
            ptr::write(self.ptr.as_ptr(), elem);
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
    /// assert_eq!(v.pop().unwrap(), 0);
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(ptr::read(self.ptr.as_ptr())) }
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
    fn grow() {
        let mut v: Vec<i32> = Vec::new();
        assert_eq!(v.cap, 0);

        for cap in (0..16).map(|c| (2 as usize).pow(c)) {
            v.grow();
            assert_eq!(v.cap, cap);
        }
    }

    #[test]
    #[should_panic]
    fn zst_panic() {
        let _: Vec<()> = Vec::new();
    }
}
