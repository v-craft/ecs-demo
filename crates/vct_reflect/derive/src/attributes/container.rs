use crate::{
    utils::{
        fp::{AnyFP, CloneFP, OptionFP, ResultFP},
        terminated_parser,
    },
    attributes::CustomAttributes,
};

use proc_macro2::{Ident, Span};
use quote::quote_spanned;
use syn::{
    ext::IdentExt, parenthesized, parse::ParseStream, spanned::Spanned, token, Expr, LitBool,
    MetaList, MetaNameValue, Path, Token, WhereClause,
};

mod kw {
    syn::custom_keyword!(from_reflect);
    syn::custom_keyword!(type_path);
    syn::custom_keyword!(Debug);
    syn::custom_keyword!(PartialEq);
    syn::custom_keyword!(Hash);
    syn::custom_keyword!(Clone);
    syn::custom_keyword!(no_field_bounds);
    syn::custom_keyword!(no_auto_register);
    syn::custom_keyword!(opaque);
}

// The "special" trait idents that are used internally for reflection.
// Received via attributes like `#[reflect(PartialEq, Hash, ...)]`
const DEBUG_ATTR: &str = "Debug";
const PARTIAL_EQ_ATTR: &str = "PartialEq";
const HASH_ATTR: &str = "Hash";

// The traits listed below are not considered "special" (i.e. they use the `ReflectMyTrait` syntax)
// but useful to know exist nonetheless
pub(crate) const REFLECT_DEFAULT: &str = "ReflectDefault";

// Attributes for `FromReflect` implementation
const FROM_REFLECT_ATTR: &str = "from_reflect";

// Attributes for `TypePath` implementation
const TYPE_PATH_ATTR: &str = "type_path";

// The error message to show when a trait/type is specified multiple times
const CONFLICTING_TYPE_DATA_MESSAGE: &str = "conflicting type data registration";

/// A marker for trait implementations registered via the `Reflect` derive macro.
#[derive(Clone, Default)]
pub(crate) enum TraitImpl {
    /// The trait is not registered as implemented.
    #[default]
    NotImplemented,

    /// The trait is registered as implemented.
    Implemented(Span),

    /// The trait is registered with a custom function rather than an actual implementation.
    Custom(Path, Span),
}

impl TraitImpl {
    /// Merges this [`TraitImpl`] with another.
    ///
    /// Update `self` with whichever value is not [`TraitImpl::NotImplemented`].
    /// If `other` is [`TraitImpl::NotImplemented`], then `self` is not modified.
    /// An error is returned if neither value is [`TraitImpl::NotImplemented`].
    pub fn merge(&mut self, other: TraitImpl) -> Result<(), syn::Error> {
        match (&self, other) {
            (TraitImpl::NotImplemented, value) => {
                *self = value;
                Ok(())
            }
            (_, TraitImpl::NotImplemented) => Ok(()),
            (_, TraitImpl::Implemented(span) | TraitImpl::Custom(_, span)) => {
                Err(syn::Error::new(span, CONFLICTING_TYPE_DATA_MESSAGE))
            }
        }
    }
}

/// A collection of attributes used for deriving `FromReflect`.
#[derive(Clone, Default)]
pub(crate) struct FromReflectAttrs {
    auto_derive: Option<LitBool>,
}

impl FromReflectAttrs {
    /// Returns true if `FromReflect` should be automatically derived as part of the `Reflect` derive.
    pub fn should_auto_derive(&self) -> bool {
        self.auto_derive.as_ref().is_none_or(LitBool::value)
    }
}

/// A collection of attributes used for deriving `TypePath` via the `Reflect` derive.
///
/// Note that this differs from the attributes used by the `TypePath` derive itself,
/// which look like `[type_path = "my_crate::foo"]`.
/// The attributes used by reflection take the form `#[reflect(type_path = false)]`.
///
/// These attributes should only be used for `TypePath` configuration specific to
/// deriving `Reflect`.
#[derive(Clone, Default)]
pub(crate) struct TypePathAttrs {
    auto_derive: Option<LitBool>,
}

impl TypePathAttrs {
    /// Returns true if `TypePath` should be automatically derived as part of the `Reflect` derive.
    pub fn should_auto_derive(&self) -> bool {
        self.auto_derive.as_ref().is_none_or(LitBool::value)
    }
}

#[derive(Default, Clone)]
pub(crate) struct ContainerAttributes {
    clone: TraitImpl,
    debug: TraitImpl,
    hash: TraitImpl,
    partial_eq: TraitImpl,
    from_reflect_attrs: FromReflectAttrs,
    type_path_attrs: TypePathAttrs,
    custom_where: Option<WhereClause>,
    no_field_bounds: bool,
    no_auto_register: bool,
    custom_attributes: CustomAttributes,
    is_opaque: bool,
    idents: Vec<Ident>,
}




