pub use time_impl::Instant;

crate::cfg::switch! {
    crate::cfg::web => {
        use web_time as time_impl;
    }
    crate::cfg::std => {
        use std::time as time_impl;
    }
    _ => {
        compile_error!("This platform is not supported");
    }
}
