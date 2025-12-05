use alloc::boxed::Box;
use core::{any::TypeId, fmt};
use vct_utils::collections::TypeIdMap;

use crate::Reflect;

/// Container for recording custom attributes.
/// 
/// # Note
/// 
/// The choice of internal type for `CustomAttributes` is an interesting question. 
/// Using `dyn Any` could also meet the requirements. 
/// However, since this custom attribute is provided by the reflection system 
/// and most regular types have reflection support implemented by this library, 
/// we still choose to store `dyn Reflect`.
#[derive(Default)]
pub struct CustomAttributes {
    attributes: TypeIdMap<Box<dyn Reflect>>,
}

impl CustomAttributes {
    /// Creates an empty [`CustomAttributes`].
    ///
    /// Equivalent to [`Default`], but available as a `const` function.
    #[inline]
    pub const fn new() -> Self {
        Self {
            attributes: TypeIdMap::new(),
        }
    }

    /// Adds an attribute.
    #[inline]
    pub fn with_attribute<T: Reflect>(mut self, value: T) -> Self {
        self.attributes.insert(TypeId::of::<T>(), Box::new(value));
        self
    }

    /// Returns an iterator over the stored attributes.
    #[inline]
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&TypeId, &dyn Reflect)> {
        self.attributes.iter().map(|(key, val)| (key, &**val))
    }

    /// Returns `true` if it contains the given attribute type.
    #[inline]
    pub fn contains<T: Reflect>(&self) -> bool {
        self.attributes.contains_key(&TypeId::of::<T>())
    }

    /// Returns `true` if it contains the attribute with the given `TypeId`.
    #[inline]
    pub fn contains_by_id(&self, id: TypeId) -> bool {
        self.attributes.contains_key(&id)
    }

    /// Returns the attribute of type `T`, if present.
    #[inline]
    pub fn get<T: Reflect>(&self) -> Option<&T> {
        self.attributes.get(&TypeId::of::<T>())?.downcast_ref::<T>()
    }

    /// Returns the attribute with the given `TypeId`, if present.
    #[inline]
    pub fn get_by_id(&self, id: TypeId) -> Option<&dyn Reflect> {
        Some(self.attributes.get(&id)?.as_ref())
    }

    /// Returns the number of stored attributes.
    #[inline]
    pub fn len(&self) -> usize {
        self.attributes.len()
    }

    /// Returns `true` if no attributes are stored.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }
}

impl fmt::Debug for CustomAttributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Not inline: Debug allows for performance loss
        f.debug_set().entries(self.attributes.values()).finish()
    }
}

/// impl `custom_attributes` `get_attribute` `get_attribute_by_id`
/// `has_attribute` `has_attribute_by_id`
macro_rules! impl_custom_attributes_fn {
    ($field:ident) => {
        $crate::info::attributes::impl_custom_attributes_fn!(self => &self.$field);
    };
    ($self:ident => $expr:expr) => {
        /// Returns its custom attributes, if any.
        #[inline]
        pub fn custom_attributes($self: &Self) -> Option<&$crate::info::CustomAttributes> {
            match $expr {
                Some(arc) => Some(&**arc),
                None => None,
            }
        }

        /// Returns the attribute of type `T`, if present.
        pub fn get_attribute<T: $crate::Reflect>($self: &Self) -> Option<&T> {
            // Not inline: Avoid excessive inline (recursion)
            $self.custom_attributes()?.get::<T>()
        }

        /// Returns the attribute with the given `TypeId`, if present.
        pub fn get_attribute_by_id($self: &Self, __id: ::core::any::TypeId) -> Option<&dyn $crate::Reflect> {
            // Not inline: Avoid excessive inline (recursion)
            $self.custom_attributes()?.get_by_id(__id)
        }

        /// Returns `true` if it contains the given attribute type.
        pub fn has_attribute<T: $crate::Reflect>($self: &Self) -> bool {
            // Not inline: Avoid excessive inline (recursion)
            match $self.custom_attributes() {
                Some(attrs) => attrs.contains::<T>(),
                None => false,
            }
        }

        /// Returns `true` if it contains the attribute with the given `TypeId`.
        pub fn has_attribute_by_id($self: &Self, __id: ::core::any::TypeId) -> bool {
            // Not inline: Avoid excessive inline (recursion)
            match $self.custom_attributes() {
                Some(attrs) => attrs.contains_by_id(__id),
                None => false,
            }
        }
    };
}

macro_rules! impl_with_custom_attributes {
    ($field:ident) => {
        /// Replaces stored attributes (overwrite, do not merge).
        ///
        /// Used by the proc-macro crate.
        pub fn with_custom_attributes(self, attributes: CustomAttributes) -> Self {
            if attributes.is_empty() {
                Self {
                    $field: None,
                    ..self
                }
            } else {
                Self {
                    $field: Some(Arc::new(attributes)),
                    ..self
                }
            }
        }
    };
}

pub(crate) use impl_custom_attributes_fn;
pub(crate) use impl_with_custom_attributes;
