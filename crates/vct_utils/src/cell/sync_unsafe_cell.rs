#![expect(unsafe_code, reason = "SyncUnsafeCell requires unsafe code.")]

//! 不稳定 API [`std::sync::SyncUnsafeCell`] 的一种实现
//!
//! [`std::sync::SyncUnsafeCell`]: https://doc.rust-lang.org/nightly/std/cell/struct.SyncUnsafeCell.html

use core::cell::UnsafeCell;
use core::ptr;

/// 参考 [`SyncUnsafeCell`](https://doc.rust-lang.org/nightly/std/cell/struct.SyncUnsafeCell.html)
#[repr(transparent)]
pub struct SyncUnsafeCell<T: ?Sized> {
    value: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Sync> Sync for SyncUnsafeCell<T> {}

impl<T> SyncUnsafeCell<T> {
    /// 从给定值构建新的 `SyncUnsafeCell` 实例
    #[inline]
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
        }
    }

    /// 析构自身，并移动出内部值
    #[inline]
    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }
}

impl<T: ?Sized> SyncUnsafeCell<T> {
    /// 获取可变指针
    ///
    /// 使用时需遵守别名规则
    #[inline]
    pub const fn get(&self) -> *mut T {
        self.value.get()
    }

    /// 获取可变引用
    ///
    /// 使用时需遵守别名规则
    #[inline]
    pub const fn get_mut(&mut self) -> &mut T {
        self.value.get_mut()
    }

    /// 获取内部数据类型的指针
    #[inline]
    pub const fn raw_get(this: *const Self) -> *mut T {
        (this as *const T).cast_mut()
    }

    /// 从可变引用获取 `SyncUnsafeCell` 的可变引用
    #[inline]
    pub const fn from_mut(t: &mut T) -> &mut SyncUnsafeCell<T> {
        let ptr = ptr::from_mut(t) as *mut SyncUnsafeCell<T>;
        unsafe { &mut *ptr }
    }
}

impl<T> SyncUnsafeCell<[T]> {
    /// 从 `&SyncUnsafeCell<[T]>` 返回 `&[SyncUnsafeCell<T>]`
    ///
    /// # 例
    ///
    /// ```
    /// # use vct_utils::cell::SyncUnsafeCell;
    ///
    /// let slice: &mut [i32] = &mut [1, 2, 3];
    /// let cell_slice: &SyncUnsafeCell<[i32]> = SyncUnsafeCell::from_mut(slice);
    /// let slice_cell: &[SyncUnsafeCell<i32>] = cell_slice.as_slice_of_cells();
    ///
    /// assert_eq!(slice_cell.len(), 3);
    /// ```
    pub fn as_slice_of_cells(&self) -> &[SyncUnsafeCell<T>] {
        let self_ptr: *const SyncUnsafeCell<[T]> = ptr::from_ref(self);
        let slice_ptr = self_ptr as *const [SyncUnsafeCell<T>];
        unsafe { &*slice_ptr }
    }
}

impl<T: Default> Default for SyncUnsafeCell<T> {
    /// 以内部元素的默认值构成
    fn default() -> SyncUnsafeCell<T> {
        SyncUnsafeCell::new(Default::default())
    }
}

impl<T> From<T> for SyncUnsafeCell<T> {
    fn from(t: T) -> SyncUnsafeCell<T> {
        SyncUnsafeCell::new(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_unsafe_cell() {
        let mut sc = SyncUnsafeCell::new(5);
        *sc.get_mut() += 10;
        unsafe {
            *sc.get() += 100;
        }
        assert_eq!(sc.into_inner(), 115);

        let sc = SyncUnsafeCell::<i32>::default();
        assert_eq!(sc.into_inner(), 0);

        let sc = SyncUnsafeCell::from(1u8);
        let p = SyncUnsafeCell::raw_get(&raw const sc);
        assert_eq!(unsafe { *p }, 1u8);

        let mut x = 79;
        let sc = SyncUnsafeCell::from_mut(&mut x);
        assert_eq!(*sc.get_mut(), 79);
    }
}
