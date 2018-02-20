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

    fn grow(&mut self) {
        let (new_cap, ptr) = if self.cap == 0 {
            (1, self.alloc.alloc_one::<T>())
        } else {
            let elem_size = mem::size_of::<T>();

            let new_cap = self.cap * 2;
            let old_num_bytes = self.cap * elem_size;

            assert!(
                old_num_bytes <= (::std::isize::MAX as usize) / 2,
                "capacity overflow"
            );

            unsafe {
                let ptr = self.alloc
                    .realloc_array::<T>(self.ptr.as_ptr(), self.cap, new_cap);
                (new_cap, ptr)
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
