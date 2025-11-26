use alloc::string::String;

/// Temporarily replacing the unstable Concat API.
/// Fast and memory efficient string concat.
/// TODO: Replace to alloc::slice::Concat
///
/// At least twice as efficient as direct `new` + `push_str`, with potential for 10x speedup.
/// Eliminates all unnecessary memory allocations.
#[inline]
pub fn concat(arr: &[&str]) -> String {
    let mut len = 0usize;
    for &item in arr {
        len += item.len();
    }
    let mut res = String::with_capacity(len);
    for &item in arr {
        res.push_str(item);
    }
    res
}
