use serde::Serialize;
use alloc::boxed::Box;
use crate::{
    Reflect, FromReflect,
    registry::FromType,
    info::TypePath,
};

enum DynamicSerialize<'a> {
    /// An owned serializable value.
    /// 
    /// Used to store the results of [`FromReflect`]
    Owned(Box<dyn erased_serde::Serialize + 'a>),
    /// An immutable reference to a serializable value.
    Borrowed(&'a dyn erased_serde::Serialize),
}

impl DynamicSerialize<'_> {
    #[inline]
    fn as_ref(&self) -> &dyn erased_serde::Serialize {
        match self {
            Self::Borrowed(serialize) => serialize,
            Self::Owned(serialize) => serialize,
        }
    }
}

/// A struct used to serialize reflected instances of a type.
/// 
/// This is a loose type serialization, as long as the type is correct or supports [`FromReflect`].
/// When neither is satisfied, it will panic.
/// 
/// Compared to [`TypeTraitSerialize`], this has extremely small additional costs.
/// 
/// - If the types match, this is consistent with [`TypeTraitSerialize`].
/// - If the types do not match，this is eq to [`TypeTraitFromReflect`] + [`TypeTraitSerialize`].
/// 
/// [`TypeTraitSerialize`]: crate::registry::traits::TypeTraitSerialize
/// [`TypeTraitFromReflect`]: crate::registry::traits::TypeTraitFromReflect
#[derive(Clone)]
pub struct TypeTraitSerializeFrom {
    get_dynamic: fn(value: &dyn Reflect) -> DynamicSerialize,
}

impl TypeTraitSerializeFrom {
    /// Call T's [`Serialize`]
    /// 
    /// [`TypeTraitSerializeFrom`] does not have a type flag, 
    /// but the functions used internally are type specific.
    /// 
    /// - If the types match, this is consistent with [`TypeTraitSerialize`].
    /// - If the types do not match, try using FromReflect to convert the types.
    ///     - If the conversion is successful, serialize it.
    ///     - If the conversion fails, `Panic`.
    /// 
    /// # Panic
    /// - Type Mismatched and FromReflect fail.
    /// 
    /// [`TypeTraitSerialize`]: crate::registry::traits::TypeTraitSerialize
    #[inline(always)]
    pub fn serialize<S: serde::Serializer>(
        &self, value: 
        &dyn Reflect, serializer: S
    ) -> Result<S::Ok, S::Error> {
        (self.get_dynamic)(value).as_ref().serialize(serializer)
    }
}

impl<T: Reflect + TypePath + FromReflect + erased_serde::Serialize> FromType<T> for TypeTraitSerializeFrom {
    fn from_type() -> Self {
        Self {
            get_dynamic: |value| {
                match value.downcast_ref::<T>() {
                    Some(val) => DynamicSerialize::Borrowed(val),
                    None => match T::from_reflect(value) {
                        Some(val) => DynamicSerialize::Owned(Box::new(val)),
                        None => {
                            panic!(
                                "Serial type mismatch, and `FromReflect` failed on type `{}` with value: {value:?}",
                                T::type_path(),
                            );
                        },
                    }
                }
            },
        }
    }
}
