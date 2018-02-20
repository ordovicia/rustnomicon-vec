use std::marker::PhantomData;
use std::ptr::NonNull;

pub(crate) struct OwnedPtr<T> {
    ptr: NonNull<T>,
    _marker: PhantomData<T>,
}

unsafe impl<T: Send> Send for OwnedPtr<T> {}
unsafe impl<T: Sync> Sync for OwnedPtr<T> {}

impl<T> OwnedPtr<T> {
    pub(crate) fn empty() -> Self {
        OwnedPtr {
            ptr: NonNull::dangling(),
            _marker: PhantomData,
        }
    }

    // pub(crate) unsafe fn new(ptr: *mut T) -> Self {
    //     OwnedPtr {
    //         ptr: NonNull::new_unchecked(ptr),
    //         _marker: PhantomData,
    //     }
    // }

    pub(crate) fn with_non_null(ptr: NonNull<T>) -> Self {
        OwnedPtr {
            ptr,
            _marker: PhantomData,
        }
    }

    pub(crate) fn as_ptr(&self) -> NonNull<T> {
        self.ptr
    }
}
