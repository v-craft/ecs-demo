use alloc::{
    format,
    string::{String, ToString},
};
use core::{fmt, slice::Iter};
use serde::{
    Deserialize,
    de::{Error, MapAccess, SeqAccess, Visitor},
};

use crate::{
    info::{NamedField, StructInfo, StructVariantInfo},
    ops::DynamicStruct,
    registry::TypeRegistry,
    serde::SkipSerde,
};

use super::{DeserializerProcessor, InternalDeserializer};

/// A helper trait for accessing type information from struct-like types.
pub(super) trait StructLikeInfo {
    fn field<E: Error>(&self, name: &str) -> Result<&NamedField, E>;
    fn field_at<E: Error>(&self, index: usize) -> Result<&NamedField, E>;
    fn field_len(&self) -> usize;
    fn iter_fields(&self) -> Iter<'_, NamedField>;
}

impl StructLikeInfo for StructInfo {
    fn field<E: Error>(&self, name: &str) -> Result<&NamedField, E> {
        Self::field(self, name).ok_or_else(|| {
            Error::custom(format!(
                "no field named `{}` on struct `{}`",
                name,
                self.type_path(),
            ))
        })
    }

    fn field_at<E: Error>(&self, index: usize) -> Result<&NamedField, E> {
        Self::field_at(self, index).ok_or_else(|| {
            Error::custom(format!(
                "no field at index `{}` on struct `{}`",
                index,
                self.type_path(),
            ))
        })
    }

    #[inline]
    fn field_len(&self) -> usize {
        Self::field_len(self)
    }

    #[inline]
    fn iter_fields(&self) -> Iter<'_, NamedField> {
        self.iter()
    }
}

impl StructLikeInfo for StructVariantInfo {
    fn field<E: Error>(&self, name: &str) -> Result<&NamedField, E> {
        Self::field(self, name).ok_or_else(|| {
            Error::custom(format!(
                "no field named `{}` on variant `{}`",
                name,
                self.name(),
            ))
        })
    }

    fn field_at<E: Error>(&self, index: usize) -> Result<&NamedField, E> {
        Self::field_at(self, index).ok_or_else(|| {
            Error::custom(format!(
                "no field at index `{}` on variant `{}`",
                index,
                self.name(),
            ))
        })
    }

    #[inline]
    fn field_len(&self) -> usize {
        Self::field_len(self)
    }

    #[inline]
    fn iter_fields(&self) -> Iter<'_, NamedField> {
        self.iter()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Ident(pub String);

impl<'de> Deserialize<'de> for Ident {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct IdentVisitor;

        impl<'de> Visitor<'de> for IdentVisitor {
            type Value = Ident;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("identifier")
            }

            #[inline]
            fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(Ident(value.to_string()))
            }

            #[inline]
            fn visit_string<E: Error>(self, value: String) -> Result<Self::Value, E> {
                Ok(Ident(value))
            }
        }

        deserializer.deserialize_identifier(IdentVisitor)
    }
}

/// Deserializes a [struct-like] type from a mapping of fields, returning a [`DynamicStruct`].
///
/// [struct-like]: StructLikeInfo
pub(super) fn visit_struct<'de, T, V, P>(
    map: &mut V,
    info: &'static T,
    registry: &TypeRegistry,
    mut processor: Option<&mut P>,
) -> Result<DynamicStruct, V::Error>
where
    T: StructLikeInfo,
    V: MapAccess<'de>,
    P: DeserializerProcessor,
{
    let mut dynamic_struct = DynamicStruct::with_capacity(info.field_len());

    while let Some(Ident(key)) = map.next_key::<Ident>()? {
        let field_ty = info.field::<V::Error>(&key)?.ty();

        // cannot skip here, we need to call `next_value_seed`.

        let Some(type_traits) = registry.get(field_ty.id()) else {
            return Err(Error::custom(format!(
                "no type_traits found for type `{field_ty:?}`"
            )));
        };

        let value = map.next_value_seed(InternalDeserializer::new_internal(
            type_traits,
            registry,
            processor.as_deref_mut(),
        ))?;
        dynamic_struct.insert_boxed(key, value);
    }

    for field in info.iter_fields() {
        if let Some(skip_serde) = field.get_attribute::<SkipSerde>() {
             if let Some(val) = skip_serde.get(field.type_id(), registry)? {
                dynamic_struct.insert_boxed(field.name(), val);
            }
        }
    }

    Ok(dynamic_struct)
}

/// Deserializes a [struct-like] type from a sequence of fields, returning a [`DynamicStruct`].
///
/// [struct-like]: StructLikeInfo
pub(super) fn visit_struct_seq<'de, T, V, P>(
    seq: &mut V,
    info: &T,
    registry: &TypeRegistry,
    mut processor: Option<&mut P>,
) -> Result<DynamicStruct, V::Error>
where
    T: StructLikeInfo,
    V: SeqAccess<'de>,
    P: DeserializerProcessor,
{

    let len = info.field_len();
    let mut dynamic_struct = DynamicStruct::with_capacity(len);

    for index in 0..len {
        let field = info.field_at::<V::Error>(index)?;

        if field.has_attribute::<SkipSerde>() {
            if let Some(skip_serde) = field.get_attribute::<SkipSerde>() {
                if let Some(value) = skip_serde.get(field.type_id(), registry)? {
                    dynamic_struct.insert_boxed(field.name(), value);
                }
            }
            continue;
        }

        let Some(type_traits) = registry.get(field.type_id()) else {
            return Err(Error::custom(format!(
                "no type_traits found for type `{:?}`",
                field.ty()
            )));
        };

        let value = seq
            .next_element_seed(InternalDeserializer::new_internal(
                type_traits,
                registry,
                processor.as_deref_mut(),
            ))?
            .ok_or_else(|| Error::invalid_length(index, &len.to_string().as_str()))?;

        dynamic_struct.insert_boxed(field.name(), value);
    }

    Ok(dynamic_struct)
}
