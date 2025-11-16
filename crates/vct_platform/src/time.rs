crate::cfg::switch! {
    crate::cfg::web => {
        use web_time as time;
    }
    crate::cfg::std => {
        use std::time;
    }
    _ => {
        compile_error!("This platform is not supported");
    }
}

pub use time::Instant;
