#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]

pub mod cfg {
    pub use vct_os::cfg::{alloc, std};

    vct_os::cfg::define_alias! {
        #[cfg(feature = "parallel")] => parallel
    }
}

cfg::std! {
    extern crate std;
}

cfg::alloc! {
    extern crate alloc;
    // 容器仅在 alloc 启用时生效
    pub mod collections;
    // 额外的 map 容器也仅在 alloc 启用时生效
    mod maps;
    pub use maps::*;
}

cfg::parallel! {
    // parallel 特性包含 std
    mod parallel_queue;
    pub use parallel_queue::*;
}

pub mod cell;
pub mod debug_info;
mod default;
pub mod hash;
mod on_drop;
mod once_flag;

pub use default::default;
pub use on_drop::OnDrop;
pub use once_flag::OnceFlag;

pub mod prelude {
    crate::cfg::alloc! {
        pub use alloc::{
            borrow::ToOwned, boxed::Box, format, string::String, string::ToString, vec, vec::Vec,
        };
    } // 忽略 `std::prelude` 的内容

    pub use crate::debug_info::DebugName;
    pub use crate::default;
    pub use disqualified::ShortName;
}
