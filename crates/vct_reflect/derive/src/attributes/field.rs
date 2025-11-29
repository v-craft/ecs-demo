use crate::{REFLECT_ATTRIBUTE_NAME, attributes::CustomAttributes, utils::terminated_parser};
use quote::ToTokens;
use syn::{parse::ParseStream, Attribute, LitStr, Meta, Token, Type};

mod kw {
    syn::custom_keyword!(skip_serde);
    syn::custom_keyword!(ignore);
    syn::custom_keyword!(clone);
    syn::custom_keyword!(default);
    syn::custom_keyword!(alias);
}

pub(crate) const IGNORE_SERDE_ATTR: &str = "skip_serde";
pub(crate) const IGNORE_ALL_ATTR: &str = "ignore";
pub(crate) const DEFAULT_ATTR: &str = "default";
pub(crate) const CLONE_ATTR: &str = "clone";
pub(crate) const ALIAS_ATTR: &str = "alias";

/// A container for attributes defined on a reflected type's field.
#[derive(Default, Clone)]
pub(crate) struct FieldAttributes {
    /// Determines how this field should be ignored if at all.
    pub ignore: ReflectIgnoreBehavior,
    /// Sets the clone behavior of this field.
    pub clone: CloneBehavior,
    /// Sets the default behavior of this field.
    pub default: DefaultBehavior,
    /// Custom attributes created via `#[reflect(@...)]`.
    pub custom_attributes: CustomAttributes,
    /// For defining the alias wrapper type that should be used in place of the field for reflection logic.
    pub alias: Option<Type>,
}

/// Stores data about if the field should be visible via the Reflect and serialization(deserialization) interfaces
///
/// Note the relationship between serialization and reflection is such that a member must be reflected in order to be serialized.
/// In boolean logic this is described as: `is_serialized -> is_reflected`, 
/// this means we can reflect something without serializing it but not the other way round.
/// The `is_reflected` predicate is provided as `self.is_active()`
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReflectIgnoreBehavior {
    /// Don't ignore, appear to all systems
    #[default]
    None,
    /// Ignore when serializing but not when reflecting
    IgnoreSerde,
    /// Ignore both when serializing and reflecting
    IgnoreAlways,
}


#[derive(Default, Clone)]
pub(crate) enum CloneBehavior {
    #[default]
    Default,
    Trait,
    Func(syn::ExprPath),
}

/// Controls how the default value is determined for a field.
#[derive(Default, Clone)]
pub(crate) enum DefaultBehavior {
    /// Field is required.
    #[default]
    Required,
    /// Field can be defaulted using `Default::default()`.
    Default,
    /// Field can be created using the given function name.
    ///
    /// This assumes the function is in scope, is callable with zero arguments,
    /// and returns the expected type.
    Func(syn::ExprPath),
}

impl FieldAttributes {
    /// Parse `ignore` attribute.
    ///
    /// Examples:
    /// - `#[reflect(ignore)]`
    fn parse_ignore(&mut self, input: ParseStream) -> syn::Result<()> {
        if self.ignore != ReflectIgnoreBehavior::None {
            return Err(input.error(format!(
                "only one of {:?} is allowed",
                [IGNORE_ALL_ATTR, IGNORE_SERDE_ATTR]
            )));
        }

        input.parse::<kw::ignore>()?;
        self.ignore = ReflectIgnoreBehavior::IgnoreAlways;
        Ok(())
    }

    /// Parse `skip_serde` attribute.
    ///
    /// Examples:
    /// - `#[reflect(skip_serde)]`
    fn parse_skip_serde(&mut self, input: ParseStream) -> syn::Result<()> {
        if self.ignore != ReflectIgnoreBehavior::None {
            return Err(input.error(format!(
                "only one of {:?} is allowed",
                [IGNORE_ALL_ATTR, IGNORE_SERDE_ATTR]
            )));
        }

        input.parse::<kw::skip_serde>()?;
        self.ignore = ReflectIgnoreBehavior::IgnoreSerde;
        Ok(())
    }

    /// Parse `clone` attribute.
    ///
    /// Examples:
    /// - `#[reflect(clone)]`
    /// - `#[reflect(clone = "path::to::func")]`
    fn parse_clone(&mut self, input: ParseStream) -> syn::Result<()> {
        if !matches!(self.clone, CloneBehavior::Default) {
            return Err(input.error(format!("only one of {:?} is allowed", [CLONE_ATTR])));
        }

        input.parse::<kw::clone>()?;

        if input.peek(Token![=]) {
            // #[reflect(clone = "path::to::func")]
            input.parse::<Token![=]>()?;
            let lit = input.parse::<LitStr>()?;
            self.clone = CloneBehavior::Func(lit.parse()?);
        } else {
            // #[reflect(clone)]
            self.clone = CloneBehavior::Trait;
        }

        Ok(())
    }

    /// Parse `default` attribute.
    ///
    /// Examples:
    /// - `#[reflect(default)]`
    /// - `#[reflect(default = "path::to::func")]`
    fn parse_default(&mut self, input: ParseStream) -> syn::Result<()> {
        if !matches!(self.default, DefaultBehavior::Required) {
            return Err(input.error(format!("only one of {:?} is allowed", [DEFAULT_ATTR])));
        }

        input.parse::<kw::default>()?;

        if input.peek(Token![=]) {
            // #[reflect(default = "path::to::func")]
            input.parse::<Token![=]>()?;
            let lit = input.parse::<LitStr>()?;
            self.default = DefaultBehavior::Func(lit.parse()?);
        } else {
            // #[reflect(default)]
            self.default = DefaultBehavior::Default;
        }

        Ok(())
    }

    /// Parse `@` (custom attribute) attribute.
    ///
    /// Examples:
    /// - `#[reflect(@Foo))]`
    /// - `#[reflect(@Bar::baz("qux"))]`
    /// - `#[reflect(@0..256u8)]`
    fn parse_custom_attribute(&mut self, input: ParseStream) -> syn::Result<()> {
        self.custom_attributes.parse_custom_attribute(input)
    }

    /// Parse `alias` attribute.
    ///
    /// Examples:
    /// - `#[reflect(alias = path::to::SameType)]`
    fn parse_alias(&mut self, input: ParseStream) -> syn::Result<()> {
        if let Some(alias) = self.alias.as_ref() {
            return Err(input.error(format!(
                "alias type already specified as {}",
                alias.to_token_stream()
            )));
        }

        input.parse::<kw::alias>()?;
        input.parse::<Token![=]>()?;

        self.alias = Some(input.parse()?);

        Ok(())
    }

    fn parse_field_attribute(&mut self, input: ParseStream) -> syn::Result<()> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![@]) {
            self.parse_custom_attribute(input)
        } else if lookahead.peek(kw::ignore) {
            self.parse_ignore(input)
        } else if lookahead.peek(kw::skip_serde) {
            self.parse_skip_serde(input)
        } else if lookahead.peek(kw::clone) {
            self.parse_clone(input)
        } else if lookahead.peek(kw::default) {
            self.parse_default(input)
        } else if lookahead.peek(kw::alias) {
            self.parse_alias(input)
        } else {
            Err(lookahead.error())
        }
    }

    /// Parse **all** field attributes marked "reflect" (such as `#[reflect(ignore)]`).
    pub fn parse_attributes(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut args = FieldAttributes::default();

        attrs
            .iter()
            .filter_map(|attr| {
                if !attr.path().is_ident(REFLECT_ATTRIBUTE_NAME) {
                    // Is not a reflect attribute, skip
                    return None;
                }

                let Meta::List(meta) = &attr.meta else {
                    return Some(syn::Error::new_spanned(attr, "reflect attribute expected meta list"));
                };

                let parser = terminated_parser(Token![,], |stream| {
                    args.parse_field_attribute(stream)
                });

                meta.parse_args_with(parser).err()
            })
            .reduce(|mut acc, err| {
                acc.combine(err);
                acc
            })
            .map_or(Ok(args), Err)
    }

    /// Returns `Some(true)` if the field has a generic alias type.
    ///
    /// If the alias type is not generic, returns `Some(false)`.
    ///
    /// If the field does not have a alias type, returns `None`.
    pub fn alias_has_generic(&self) -> Option<bool> {
        if let Type::Path(type_path) = self.alias.as_ref()? {
            type_path
                .path
                .segments
                .last()
                .map(|segment| !segment.arguments.is_empty())
        } else {
            Some(false)
        }
    }

}

impl ReflectIgnoreBehavior {
    /// Returns `true` if the ignoring behavior implies member is included in the reflection API, and false otherwise.
    pub fn is_active(self) -> bool {
        match self {
            ReflectIgnoreBehavior::None | ReflectIgnoreBehavior::IgnoreSerde => true,
            ReflectIgnoreBehavior::IgnoreAlways => false,
        }
    }

    /// The exact logical opposite of `self.is_active()` returns true iff this member is not part of the reflection API whatsoever (neither serialized nor reflected)
    pub fn is_ignored(self) -> bool {
        !self.is_active()
    }
}

