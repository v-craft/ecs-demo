use crate::info::TypeInfo;
use alloc::vec::Vec;
use core::fmt;

/// Helper struct for managing a stack of [`TypeInfo`] instances.
///
/// This is useful for tracking the type hierarchy when serializing and deserializing types.
#[derive(Default, Clone)]
pub(crate) struct TypeInfoStack(Vec<&'static TypeInfo>);

impl TypeInfoStack {
    #[inline]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    pub fn push(&mut self, info: &'static TypeInfo) {
        self.0.push(info);
    }

    #[inline]
    pub fn pop(&mut self) {
        self.0.pop();
    }

    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, &'static TypeInfo> {
        self.0.iter()
    }
}

impl fmt::Debug for TypeInfoStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.iter();

        if let Some(first) = iter.next() {
            write!(f, "`{}`", first.type_path())?;
        }

        for info in iter {
            write!(f, " -> `{}`", info.type_path())?;
        }
        Ok(())
    }
}
