use std::heap::{Alloc, Heap};
use std::mem;

use owned_ptr::OwnedPtr;

pub(super) struct RawVec<T> {
    pub(super) ptr: OwnedPtr<T>,
    pub(super) cap: usize,
    alloc: Heap,
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        let elem_size = mem::size_of::<T>();

        // don't free zero-sized allocations, as they were never allocated.
        if self.cap == 0 || elem_size == 0 {
            return;
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

impl<T> RawVec<T> {
    pub(super) fn default() -> Self {
        // !0 is usize::MAX. This branch should be stripped at compile time.
        let cap = if mem::size_of::<T>() == 0 { !0 } else { 0 };

        RawVec {
            ptr: OwnedPtr::empty(),
            cap,
            alloc: Heap,
        }
    }

    pub(super) fn grow(&mut self) {
        let elem_size = mem::size_of::<T>();

        // since we set the capacity to usize::MAX when elem_size is
        // 0, getting to here necessarily means the Vec is overfull.
        assert!(elem_size != 0, "capacity overflow");

        let (ptr, new_cap) = if self.cap == 0 {
            (self.alloc.alloc_one::<T>(), 1)
        } else {
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
        let mut v: RawVec<i32> = RawVec::default();
        assert_eq!(v.cap, 0);

        for cap in (0..16).map(|c| (2 as usize).pow(c)) {
            v.grow();
            assert_eq!(v.cap, cap);
        }
    }

    #[test]
    fn drop_zst() {
        let _: RawVec<()> = RawVec::default();
    }
}
