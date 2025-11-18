//! 提供 [`std::sync::Exclusive`] 和 [`std::cell::SyncUnsafeCell`] 的平替

mod sync_cell;
mod sync_unsafe_cell;

pub use sync_cell::SyncCell;
pub use sync_unsafe_cell::SyncUnsafeCell;
