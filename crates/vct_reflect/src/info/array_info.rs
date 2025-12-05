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
    ops::Array,
};

/// Container for storing compile-time array information.
#[derive(Clone, Debug)]
pub struct ArrayInfo {
    ty: Type,
    generics: Generics,
    item_ty: Type,
    // `TypeInfo` is created on the first visit, use function pointers to delay it.
    item_info: fn() -> &'static TypeInfo,
    capacity: usize,
    // Use `Option` to reduce unnecessary heap requests (when empty content).
    custom_attributes: Option<Arc<CustomAttributes>>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl ArrayInfo {
    impl_type_fn!(ty);
    impl_docs_fn!(docs);
    impl_generic_fn!(generics);
    impl_custom_attributes_fn!(custom_attributes);
    impl_with_custom_attributes!(custom_attributes);

    /// Create a new [`ArrayInfo`].
    pub fn new<TArray: Array + TypePath, TItem: Reflect + Typed>(capacity: usize) -> Self {
        // Not Inline: Perhaps it can reduce compilation time.
        Self {
            ty: Type::of::<TArray>(),
            generics: Generics::new(),
            item_ty: Type::of::<TItem>(),
            item_info: TItem::type_info,
            capacity,
            custom_attributes: None,
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }

    /// Returns the fixed array capacity.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the [`TypeInfo`] of array items.
    #[inline]
    pub fn item_info(&self) -> &'static TypeInfo {
        (self.item_info)()
    }

    /// Returns the [`Type`] of an array item.
    #[inline]
    pub fn item_ty(&self) -> Type {
        self.item_ty
    }
}
