// Temporarily replacing the unstable Concat API.
// Fast and memory efficient string concat.
// TODO: Replace to alloc::slice::Concat
mod concat;
pub use concat::concat;
