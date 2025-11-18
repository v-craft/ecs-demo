use core::mem::ManuallyDrop;

/// 会在自身 drop 时自动调用内部函数的类型
///
/// 此函数可用于在 `panic` 时清理代码（处于 `unwind` 模式时），
/// 类似 C++  RAII 模式的析构函数负责清理资源。
///
/// # 例
///
/// ```
/// # use vct_utils::OnDrop;
/// # fn test_panic(do_panic: bool, log: impl FnOnce(&str)) {
///
/// let _catch = OnDrop::new(|| log("Oops, a panic occurred and this function didn't complete!"));
///
/// // 一些可能导致 panic 的代码...
/// // ...
/// # if do_panic { panic!() }
///
/// // 函数结尾主动 `forget` ，因为运行到此说明没有发生 panic，无需清理资源
/// core::mem::forget(_catch);
/// }
/// #
/// # test_panic(false, |_| unreachable!());
/// # let mut did_log = false;
/// # std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
/// #   test_panic(true, |_| did_log = true);
/// # }));
/// # assert!(did_log);
/// ```
pub struct OnDrop<F: FnOnce()> {
    callback: ManuallyDrop<F>,
}

impl<F: FnOnce()> OnDrop<F> {
    /// 创建一个新对象，自身 drop 时会调用指定的函数
    pub fn new(callback: F) -> Self {
        Self {
            callback: ManuallyDrop::new(callback),
        }
    }
}

impl<F: FnOnce()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        #![expect(unsafe_code, reason = "ManuallyDrop::take is unsafe.")]
        let callback = unsafe { ManuallyDrop::take(&mut self.callback) };
        callback();
    }
}
