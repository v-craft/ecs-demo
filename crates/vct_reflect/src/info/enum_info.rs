use alloc::{boxed::Box, format, string::String};
use vct_os::sync::Arc;
use vct_utils::collections::HashMap;

use crate::{
    info::{
        CustomAttributes, Generics, Type, TypePath, VariantInfo,
        attributes::{impl_custom_attributes_fn, impl_with_custom_attributes},
        docs_macro::impl_docs_fn,
        generics::impl_generic_fn,
        type_struct::impl_type_fn,
    },
    ops::Enum,
};

/// Container for storing compile-time enum information.
#[derive(Clone, Debug)]
pub struct EnumInfo {
    ty: Type,
    generics: Generics,
    variants: Box<[VariantInfo]>,
    variant_names: Box<[&'static str]>,
    variant_indices: HashMap<&'static str, usize>,
    // Use `Option` to reduce unnecessary heap requests (when empty content).
    custom_attributes: Option<Arc<CustomAttributes>>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl EnumInfo {
    impl_type_fn!(ty);
    impl_docs_fn!(docs);
    impl_generic_fn!(generics);
    impl_custom_attributes_fn!(custom_attributes);
    impl_with_custom_attributes!(custom_attributes);

    /// Creates a new [`EnumInfo`].
    pub fn new<TEnum: Enum + TypePath>(variants: &[VariantInfo]) -> Self {
        // Not Inline: Reduce compilation time.
        let variant_indices = variants
            .iter()
            .enumerate()
            .map(|(index, variant)| (variant.name(), index))
            .collect();

        let variant_names = variants.iter().map(VariantInfo::name).collect();

        Self {
            ty: Type::of::<TEnum>(),
            generics: Generics::new(),
            variants: variants.to_vec().into_boxed_slice(),
            variant_names,
            variant_indices,
            custom_attributes: None,
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }

    /// Returns the list of variant names.
    #[inline]
    pub fn variant_names(&self) -> &[&'static str] {
        &self.variant_names
    }

    /// Returns the [`VariantInfo`] for the given name, if it exists.
    #[inline]
    pub fn variant(&self, name: &str) -> Option<&VariantInfo> {
        self.variant_indices
            .get(name)
            .map(|index| &self.variants[*index])
    }

    /// Returns the [`VariantInfo`] at the given index, if it exists.
    #[inline]
    pub fn variant_at(&self, index: usize) -> Option<&VariantInfo> {
        self.variants.get(index)
    }

    /// Returns the index for the given variant name, if it exists.
    #[inline]
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.variant_indices.get(name).copied()
    }

    /// Returns the full type path for the given variant name.
    #[inline]
    pub fn variant_path(&self, name: &str) -> String {
        format!("{}::{name}", self.type_path())
    }

    /// Returns `true` if a variant with the given name exists.
    #[inline]
    pub fn contains_variant(&self, name: &str) -> bool {
        self.variant_indices.contains_key(name)
    }

    /// Returns an iterator over the contained variants.
    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, VariantInfo> {
        self.variants.iter()
    }

    /// Returns the number of contained variants.
    #[inline]
    pub fn variant_len(&self) -> usize {
        self.variants.len()
    }
}
