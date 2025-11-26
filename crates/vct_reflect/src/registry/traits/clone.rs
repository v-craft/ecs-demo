use crate::{Reflect, info::TypePath, registry::FromType};
use alloc::boxed::Box;

/// See [`Clone`]
#[derive(Clone)]
pub struct TypeTraitClone {
    clone: fn(&dyn Reflect) -> Box<dyn Reflect>,
}

impl TypeTraitClone {
    /// # Panic
    /// - Mismatched Type
    #[inline(always)]
    pub fn clone(&self, param_1: &dyn Reflect) -> Box<dyn Reflect> {
        (self.clone)(param_1)
    }
}

impl<T: Clone + Reflect + TypePath> FromType<T> for TypeTraitClone {
    fn from_type() -> Self {
        Self {
            clone: |param_1| {
                if let Some(val) = param_1.downcast_ref::<T>() {
                    Box::new(Clone::clone(val)) as Box<dyn Reflect>
                } else {
                    panic!(
                        "`TypeTraitClone::clone` Type mismatched: self: `{}`, clone to: `{}`.",
                        param_1.reflect_type_path(),
                        T::type_path(),
                    );
                }
            },
        }
    }
}
