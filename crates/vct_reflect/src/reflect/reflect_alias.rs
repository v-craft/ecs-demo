use crate::Reflect;

/// This Trait allows types from external libraries to be included in reflection.
///
/// Here 'Alias' means the same memory layout, which requires the following:
/// - `Self` is a single-field tuple/struct (newtype) containing the remote type.
/// - `Self` is `#[repr(transparent)]` over the alias type.
pub trait ReflectAlias: Reflect {
    /// The remote type this type represents via reflection.
    type Alias;

    /// Converts a reference of this wrapper to a reference of its remote type.
    fn as_alias(&self) -> &Self::Alias;
    /// Converts a mutable reference of this wrapper to a mutable reference of its remote type.
    fn as_alias_mut(&mut self) -> &mut Self::Alias;
    /// Converts this wrapper into its remote type.
    fn into_alias(self) -> Self::Alias;

    /// Converts a reference of the remote type to a reference of this wrapper.
    fn as_self(remote: &Self::Alias) -> &Self;
    /// Converts a mutable reference of the remote type to a mutable reference of this wrapper.
    fn as_self_mut(remote: &mut Self::Alias) -> &mut Self;
    /// Converts the remote type into this wrapper.
    fn into_self(remote: Self::Alias) -> Self;
}
