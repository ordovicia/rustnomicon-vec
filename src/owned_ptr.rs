use std::marker::PhantomData;
use std::ptr::NonNull;

pub(super) struct OwnedPtr<T: ?Sized> {
    ptr: NonNull<T>,
    _marker: PhantomData<T>,
}

impl<T: ?Sized> Clone for OwnedPtr<T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized> Copy for OwnedPtr<T> {}

unsafe impl<T: ?Sized + Send> Send for OwnedPtr<T> {}
unsafe impl<T: ?Sized + Sync> Sync for OwnedPtr<T> {}

impl<T: ?Sized> OwnedPtr<T> {
    pub(super) fn with_non_null(ptr: NonNull<T>) -> Self {
        OwnedPtr {
            ptr,
            _marker: PhantomData,
        }
    }

    pub(super) fn as_non_null(&self) -> NonNull<T> {
        self.ptr
    }

    pub(super) fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }
}

impl<T> OwnedPtr<T> {
    pub(super) fn empty() -> Self {
        OwnedPtr {
            ptr: NonNull::dangling(),
            _marker: PhantomData,
        }
    }
}
