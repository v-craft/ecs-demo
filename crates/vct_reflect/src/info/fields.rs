use alloc::borrow::Cow;
use core::fmt;
use vct_os::sync::Arc;

use crate::info::{
    CustomAttributes, Type, TypeInfo, Typed,
    attributes::{impl_custom_attributes_fn, impl_with_custom_attributes},
    docs_macro::impl_docs_fn,
    type_struct::impl_type_fn,
};

/// A named (struct) field.
#[derive(Clone, Debug)]
pub struct NamedField {
    ty: Type,
    name: &'static str,
    // `TypeInfo` is created on the first visit, use function pointers to delay it.
    type_info: fn() -> &'static TypeInfo,
    // Use `Option` to reduce unnecessary heap requests (when empty content).
    custom_attributes: Option<Arc<CustomAttributes>>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl NamedField {
    impl_docs_fn!(docs);
    impl_type_fn!(ty);
    impl_custom_attributes_fn!(custom_attributes);
    impl_with_custom_attributes!(custom_attributes);

    /// Creates a new [`NamedField`].
    #[inline]
    pub fn new<T: Typed>(name: &'static str) -> Self {
        Self {
            name,
            type_info: T::type_info,
            ty: Type::of::<T>(),
            custom_attributes: None,
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }

    /// Returns the field name.
    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the field's [`TypeInfo`].
    #[inline]
    pub fn type_info(&self) -> &'static TypeInfo {
        (self.type_info)()
    }
}

/// An unnamed (tuple) field.
#[derive(Clone, Debug)]
pub struct UnnamedField {
    ty: Type,
    index: usize,
    // `TypeInfo` is created on the first visit, use function pointers to delay it.
    type_info: fn() -> &'static TypeInfo,
    // Use `Option` to reduce unnecessary heap requests (when empty content).
    custom_attributes: Option<Arc<CustomAttributes>>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl UnnamedField {
    impl_docs_fn!(docs);
    impl_type_fn!(ty);
    impl_custom_attributes_fn!(custom_attributes);
    impl_with_custom_attributes!(custom_attributes);

    /// Creates a new [`UnnamedField`].
    #[inline]
    pub fn new<T: Typed>(index: usize) -> Self {
        Self {
            index,
            type_info: T::type_info,
            ty: Type::of::<T>(),
            custom_attributes: None,
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }

    /// Returns the field index.
    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns the field's [`TypeInfo`].
    #[inline]
    pub fn type_info(&self) -> &'static TypeInfo {
        (self.type_info)()
    }
}

/// A container for representing field identifiers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldId {
    Named(Cow<'static, str>),
    Unnamed(usize),
}

impl fmt::Display for FieldId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Named(name) => fmt::Display::fmt(name, f),
            Self::Unnamed(name) => fmt::Display::fmt(name, f),
        }
    }
}
