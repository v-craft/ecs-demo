#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]

extern crate alloc;

pub mod cfg {
    pub use vct_os::cfg::std;

    vct_os::cfg::define_alias! {
        #[cfg(feature = "parallel")] => parallel
    }
}

cfg::std! {
    extern crate std;
}

cfg::parallel! {
    // parallel 特性包含 std
    mod parallel_queue;
    pub use parallel_queue::*;
}

pub mod cell;
pub mod collections;
pub mod debug_info;
pub mod hash;

mod default;
mod maps;
mod on_drop;
mod once_flag;

pub use default::default;
pub use maps::*;
pub use on_drop::OnDrop;
pub use once_flag::OnceFlag;

pub mod prelude {
    pub use alloc::{
        borrow::ToOwned, boxed::Box, format, string::String, string::ToString, vec, vec::Vec,
    };

    pub use crate::debug_info::DebugName;
    pub use crate::default;
    pub use disqualified::ShortName;
}
