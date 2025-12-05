use core::any::TypeId;
use std::format;
use alloc::{boxed::Box, borrow::Cow};

use crate::{
    Reflect,
    cell::NonGenericTypeInfoCell,
    info::{OpaqueInfo, ReflectKind, TypeInfo, TypePath, Typed},
    ops::{ApplyError, ReflectCloneError, ReflectMut, ReflectOwned, ReflectRef},
    reflect::impl_cast_reflect_fn, registry::{TypeRegistry, TypeTraitDefault},
};


/// A custom attribute use to skip serialization and deserialization of fields.
pub enum SkipSerde {
    /// Skip directly
    None,       
    /// Use default values, The type needs to register `TypeTraitDefault` 
    Default,    
    /// Clone a value, need to support reflect_clone and have the correct type.
    Clone(Box<dyn Reflect>),    
}

impl SkipSerde {
    #[inline]
    pub fn clone<T: Reflect>(x: T) -> Self {
        #[cfg(debug_assertions)]
        if let Err(err) = x.reflect_clone() {
            panic!("As the value of `SkipSerde::Clone`, it must support `reflect_clone`: {err} .");
        };

        Self::Clone(Box::new(x).into_reflect())
    }

    pub fn get<E: serde::de::Error>(
        &self,
        id: TypeId,
        registry: &TypeRegistry,
    ) -> Result<Option<Box<dyn Reflect>>, E> {
        match self {
            SkipSerde::None => Ok(None),
            SkipSerde::Default => {
                if let Some(generator) = registry.get_type_trait::<TypeTraitDefault>(id) {
                    Ok(Some(generator.default()))
                } else {
                    Err(E::custom("`SkipSerde::Default` is used, but `TypeTraitDefault` was not found in the registry."))
                }
            },
            SkipSerde::Clone(reflect) => {
                if reflect.type_id() != id {
                    return Err(E::custom("`SkipSerde::Clone` is used, but type mismatched."));
                }
                match reflect.reflect_clone() {
                    Ok(val) => Ok(Some(val)),
                    Err(err) => Err(E::custom(format!(
                        "As the value of `SkipSerde::Clone`, it must support `reflect_clone`: {err} ."
                    ))),
                }
            },
        }
    }
}

impl TypePath for SkipSerde {
    #[inline]
    fn type_path() -> &'static str {
        "vct_reflect::serde::SkipSerde"
    }

    #[inline]
    fn type_name() -> &'static str {
        "SkipSerde"
    }

    #[inline]
    fn type_ident() -> &'static str {
        "SkipSerde"
    }

    #[inline]
    fn crate_name() -> Option<&'static str> {
        Some("vct_reflect")
    }

    #[inline]
    fn module_path() -> Option<&'static str> {
        Some("vct_reflect::serde")
    }
}

impl Typed for SkipSerde {
    fn type_info() -> &'static TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_init(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
    }
}

impl Reflect for SkipSerde {
    impl_cast_reflect_fn!();

    fn reflect_kind(&self) -> ReflectKind {
        ReflectKind::Opaque
    }

    fn reflect_ref(&self) -> ReflectRef<'_> {
        ReflectRef::Opaque(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        ReflectMut::Opaque(self)
    }

    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::Opaque(self)
    }

    fn try_apply(&mut self, _value: &dyn Reflect) -> Result<(), ApplyError> {
        Err(ApplyError::NotSupport { type_path: Cow::Borrowed(Self::type_path()) })
    }

    fn reflect_clone(&self) -> Result<Box<dyn Reflect>, ReflectCloneError> {
        match self {
            SkipSerde::None => Ok(Box::new(Self::None).into_reflect()),
            SkipSerde::Default =>  Ok(Box::new(Self::Default).into_reflect()),
            SkipSerde::Clone(val) => Ok(
                Box::new(Self::Clone(val.reflect_clone().unwrap_or_else(|err|
                    panic!("As the value of `SkipSerde::Clone`, it must support `reflect_clone`: {err} .")
                ))).into_reflect()
            ),
        }
    }

    fn to_dynamic(&self) -> Box<dyn Reflect> {
        match self {
            SkipSerde::None => Box::new(Self::None),
            SkipSerde::Default =>  Box::new(Self::Default),
            SkipSerde::Clone(val) => Box::new(Self::Clone(val.to_dynamic())),
        }
    }

    fn reflect_debug(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SkipSerde::None => f.write_str("SkipSerde::None"),
            SkipSerde::Default =>  f.write_str("SkipSerde::Default"),
            SkipSerde::Clone(val) => {
                f.write_str("SkipSerde::Clone(")?;
                val.reflect_debug(f)?;
                f.write_str(")")
            },
        }
    }
}
