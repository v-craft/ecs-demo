use vct_os::sync::Arc;

use crate::{
    Reflect,
    info::{
        CustomAttributes, Generics, Type, TypeInfo, TypePath, Typed,
        attributes::{impl_custom_attributes_fn, impl_with_custom_attributes},
        docs_macro::impl_docs_fn,
        generics::impl_generic_fn,
        type_struct::impl_type_fn,
    },
    ops::Map,
};

/// Container for storing compile-time map-like information.
#[derive(Clone, Debug)]
pub struct MapInfo {
    ty: Type,
    generics: Generics,
    key_ty: Type,
    value_ty: Type,
    // `TypeInfo` is created on the first visit, use function pointers to delay it.
    key_info: fn() -> &'static TypeInfo,
    value_info: fn() -> &'static TypeInfo,
    // Use `Option` to reduce unnecessary heap requests (when empty content).
    custom_attributes: Option<Arc<CustomAttributes>>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl MapInfo {
    impl_docs_fn!(docs);
    impl_type_fn!(ty);
    impl_generic_fn!(generics);
    impl_custom_attributes_fn!(custom_attributes);
    impl_with_custom_attributes!(custom_attributes);

    /// Creates a new [`MapInfo`].
    pub fn new<TMap: Map + TypePath, TKey: Reflect + Typed, TValue: Reflect + Typed>() -> Self {
        Self {
            ty: Type::of::<TMap>(),
            generics: Generics::new(),
            key_ty: Type::of::<TKey>(),
            value_ty: Type::of::<TValue>(),
            key_info: TKey::type_info,
            value_info: TValue::type_info,
            custom_attributes: None,
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }

    /// Returns the [`TypeInfo`] of the key.
    #[inline]
    pub fn key_info(&self) -> &'static TypeInfo {
        (self.key_info)()
    }

    /// Returns the [`Type`] of the key.
    #[inline]
    pub fn key_ty(&self) -> Type {
        self.key_ty
    }

    /// Returns the [`TypeInfo`] of the value.
    #[inline]
    pub fn value_info(&self) -> &'static TypeInfo {
        (self.value_info)()
    }

    /// Returns the [`Type`] of the value.
    #[inline]
    pub fn value_ty(&self) -> Type {
        self.value_ty
    }
}
