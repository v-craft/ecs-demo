#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod cfg;
pub mod thread;
pub mod time;
pub mod hash;
pub mod cell;
pub mod collections;

/// 重导出 web 相关库
#[doc(hidden)]
pub mod exports {
    crate::cfg::web! {
        pub use js_sys;
        pub use wasm_bindgen;
        pub use wasm_bindgen_futures;
    }
}
