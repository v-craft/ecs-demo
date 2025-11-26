use crate::{PartialReflect, Reflect};
use alloc::boxed::Box;

/// A trait that enables types to be dynamically constructed from reflected data.
pub trait FromReflect: Reflect + Sized {
    /// Constructs a concrete instance of `Self` from a reflected value.
    fn from_reflect(reflect: &dyn PartialReflect) -> Option<Self>;

    /// Attempts to downcast the given value to `Self` using,
    /// constructing the value using [`from_reflect`] if that fails.
    fn take_from_reflect(
        reflect: Box<dyn PartialReflect>,
    ) -> Result<Self, Box<dyn PartialReflect>> {
        match reflect.try_take::<Self>() {
            Ok(value) => Ok(value),
            Err(value) => match Self::from_reflect(value.as_ref()) {
                None => Err(value),
                Some(success) => Ok(success),
            },
        }
    }
}
