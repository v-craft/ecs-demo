
pub use thread::sleep;

crate::cfg::switch! {
    crate::cfg::std => {
        use std:: thread;
    }
    _ => {
        compile_error!("This platform is not supported");
    }
}
