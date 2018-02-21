use std::ptr;
use std::ops::{Deref, DerefMut};
use std::mem;
use std::heap::{Alloc, Heap};

use owned_ptr::OwnedPtr;

pub struct Vec<T> {
    ptr: OwnedPtr<T>,
    cap: usize,
    len: usize,
    alloc: Heap,
}

impl<T> Deref for Vec<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe {
            ::std::slice::from_raw_parts(self.ptr.as_ptr(), self.len)
        }
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            ::std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len)
        }
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        match self.cap {
            0 => {}
            1 => {
                if mem::needs_drop::<T>() {
                    self.pop();
                }
                unsafe {
                    self.alloc.dealloc_one(self.ptr.as_non_null());
                }
            }
            n => {
                if mem::needs_drop::<T>() {
                    while let Some(_) = self.pop() {}
                }
                unsafe {
                    if let Err(e) = self.alloc.dealloc_array(self.ptr.as_non_null(), n) {
                        self.alloc.oom(e);
                    }
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
    /// assert_eq!(v.pop().unwrap(), 0);
    ///
    /// v.push(0);
    /// v.push(1);
    /// assert_eq!(v.pop().unwrap(), 1);
    /// assert_eq!(v.pop().unwrap(), 0);
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
    use test;

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

    #[bench]
    fn dealloc_i32(b: &mut test::Bencher) {
        b.iter(|| {
            let mut v: Vec<i32> = test::black_box(Vec::new());
            for i in 0..(1 << 16) {
                v.push(i);
            }
        });
    }

    #[derive(Clone)]
    struct Array {
        x: [i32; 32],
    }

    #[bench]
    fn dealloc_array(b: &mut test::Bencher) {
        let n = Array { x: [0; 32] };

        b.iter(|| {
            let mut v = test::black_box(Vec::new());
            for _ in 0..(1 << 16) {
                v.push(n.clone());
            }
        });
    }
}
