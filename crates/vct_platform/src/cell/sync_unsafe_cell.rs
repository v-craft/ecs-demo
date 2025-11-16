#![expect(unsafe_code, reason = "SyncUnsafeCell requires unsafe code.")]

pub use core::cell::UnsafeCell;
use core::ptr;

#[repr(transparent)]
pub struct SyncUnsafeCell<T: ?Sized> {
    value: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Sync> Sync for SyncUnsafeCell<T> {}

impl<T> SyncUnsafeCell<T> {
    #[inline]
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
        }
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }
}

impl<T: ?Sized> SyncUnsafeCell<T> {
    #[inline]
    pub const fn get(&self) -> *mut T {
        self.value.get()
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.value.get_mut()
    }


    #[inline]
    pub const fn raw_get(this: *const Self) -> *mut T {
        (this as *const T).cast_mut()
    }

    #[inline]
    pub fn from_mut(t: &mut T) -> &mut SyncUnsafeCell<T> {
        let ptr = ptr::from_mut(t) as *mut SyncUnsafeCell<T>;
        unsafe { &mut *ptr }
    }
}

impl<T> SyncUnsafeCell<[T]> {
    pub fn as_slice_of_cells(&self) -> &[SyncUnsafeCell<T>] {
        let self_ptr: *const SyncUnsafeCell<[T]> = ptr::from_ref(self);
        let slice_ptr = self_ptr as *const [SyncUnsafeCell<T>];
        unsafe { &*slice_ptr }
    }
}

impl<T: Default> Default for SyncUnsafeCell<T> {
    /// Creates a new `SyncUnsafeCell` with the `Default` value for T.
    fn default() -> SyncUnsafeCell<T> {
        SyncUnsafeCell::new(Default::default())
    }
}

impl<T> From<T> for SyncUnsafeCell<T> {
    /// Creates a new `SyncUnsafeCell<T>` containing the given value.
    fn from(t: T) -> SyncUnsafeCell<T> {
        SyncUnsafeCell::new(t)
    }
}
