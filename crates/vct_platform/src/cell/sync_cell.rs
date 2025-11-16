#![expect(unsafe_code, reason = "SyncCell requires unsafe code.")]


use core::ptr;

#[repr(transparent)]
pub struct SyncCell<T: ?Sized> {
    inner: T,
}

impl<T: Sized> SyncCell<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn into_inner(Self { inner }: Self) -> T {
        inner
    }
}

impl<T: ?Sized> SyncCell<T> {
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn as_ref(&self) -> &T
    where
        T: Sync,
    {
        &self.inner
    }

    pub fn from_mut(r: &mut T) -> &mut SyncCell<T> {
        unsafe { &mut *(ptr::from_mut(r) as *mut SyncCell<T>) }
    }
}

unsafe impl<T: ?Sized> Sync for SyncCell<T> {}
