use core::any::Any;

use crate::registry::{TypeRegistry, TypeTraits};

pub trait GetTypeTraits: Any {
    /// Returns the **default** [`TypeTraits`] for this type.
    fn get_type_traits() -> TypeTraits;

    /// Registers other types needed by this type.
    /// **Allow** not to register oneself.
    fn register_dependencies(_registry: &mut TypeRegistry) {}
}
