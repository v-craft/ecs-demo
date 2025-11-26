use crate::{FromReflect, PartialReflect, Reflect, registry::FromType};
use alloc::boxed::Box;

/// See [`FromReflect`]
#[derive(Clone)]
pub struct TypeTraitFromReflect {
    from_reflect: fn(&dyn PartialReflect) -> Option<Box<dyn Reflect>>,
    take_from_reflect:
        fn(Box<dyn PartialReflect>) -> Result<Box<dyn Reflect>, Box<dyn PartialReflect>>,
}

impl TypeTraitFromReflect {
    #[inline(always)]
    pub fn from_reflect(&self, param_1: &dyn PartialReflect) -> Option<Box<dyn Reflect>> {
        (self.from_reflect)(param_1)
    }

    #[inline(always)]
    pub fn take_from_reflect(
        &self,
        param_1: Box<dyn PartialReflect>,
    ) -> Result<Box<dyn Reflect>, Box<dyn PartialReflect>> {
        (self.take_from_reflect)(param_1)
    }
}

impl<T: FromReflect> FromType<T> for TypeTraitFromReflect {
    fn from_type() -> Self {
        Self {
            from_reflect: |param_1| {
                T::from_reflect(param_1).map(|val| Box::new(val) as Box<dyn Reflect>)
            },
            take_from_reflect: |param_1| {
                T::take_from_reflect(param_1).map(|val| Box::new(val) as Box<dyn Reflect>)
            },
        }
    }
}
