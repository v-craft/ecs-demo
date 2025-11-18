use core::{cell::RefCell, ops::DerefMut};
use alloc::vec::Vec;
use thread_local::ThreadLocal;

/// 给定类型的线程局部版本
/// 
/// - 可以使用 [`Parallel::scope`] 访问当前线程中的值。
/// - 可以使用 [`Parallel::iter_mut`] 访问所有线程中的值。
pub struct Parallel<T: Send> {
    locals: ThreadLocal<RefCell<T>>,
}

// ThreadLocal 内部使用 MaybeUninit<T>，因此任何 T 类型都支持 default
impl<T: Send> Default for Parallel<T> {
    fn default() -> Self {
        Self {
            locals: ThreadLocal::default(),
        }
    }
}

impl<T: Send> Parallel<T> {
    /// 获取所有线程值的可变引用的迭代器
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &'_ mut T> {
        self.locals.iter_mut().map(RefCell::get_mut)
    }

    /// 清理所有的线程句柄存储值
    pub fn clear(&mut self) {
        self.locals.clear();
    }

    /// 尝试获取当前线程的值并调用 f 函数
    /// 
    /// 如果值不存在，则先使用 create 函数进行初始化
    pub fn scope_or<R>(&self, create: impl FnOnce() -> T, f: impl FnOnce(&mut T) -> R) -> R {
        f(&mut self.borrow_local_mut_or(create))
    }

    /// 尝试获取当前线程的值的可变借用
    /// 
    /// 如果值不存在，则先使用 create 函数进行初始化
    pub fn borrow_local_mut_or(
        &self,
        create: impl FnOnce() -> T,
    ) -> impl DerefMut<Target = T> + '_ {
        self.locals.get_or(|| RefCell::new(create())).borrow_mut()
    }
}

impl<T: Default + Send> Parallel<T> {
    /// 尝试获取当前线程的值并调用 f 函数
    /// 
    /// 如果值不存在，则先使用默认初始化
    pub fn scope<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        self.scope_or(Default::default, f)
    }

    /// 尝试获取当前线程的值的可变借用
    /// 
    /// 如果值不存在，则先使用默认初始化
    pub fn borrow_local_mut(&self) -> impl DerefMut<Target = T> + '_ {
        self.borrow_local_mut_or(Default::default)
    }
}

impl<T, I> Parallel<I>
where
    I: IntoIterator<Item = T> + Default + Send + 'static,
{
    /// 从所有线程中清空所有已排队的项，并返回一个迭代器。
    /// 
    /// 和 [`Vec::drain`] 不同，这将逐段删除存储的数据块。
    /// 迭代中途终止时，已处理的元素将被丢弃，未处理的被保留。
    pub fn drain(&mut self) -> impl Iterator<Item = T> + '_ {
        self.locals.iter_mut().flat_map(|item| item.take())
    }
}

impl<T: Send> Parallel<Vec<T>> {
    /// 从所有线程中清空所有已排队的项，并插入目标容器的末尾
    /// 
    /// 会为容器预先 reserve 恰好的空间，在 out 为空容器时无需多次扩容
    pub fn drain_into(&mut self, out: &mut Vec<T>) {
        let size: usize = self
            .locals
            .iter_mut()
            .map(|queue| queue.get_mut().len())
            .sum();
        out.reserve(size /* + out.len() */);
        for queue in self.locals.iter_mut() {
            out.append(queue.get_mut());
        }
    }
}

