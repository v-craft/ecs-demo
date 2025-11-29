use syn::{
    Field, Member, Path, Type,
};
use quote::{quote, ToTokens};
use crate::attributes::FieldAttributes;

/// Represents a field on a struct or tuple struct.
#[derive(Clone)]
pub(crate) struct StructField<'a> {
    /// The raw field.
    pub data: &'a Field,
    /// The reflection-based attributes on the field.
    pub attrs: FieldAttributes,
    /// The index of this field within the struct.
    pub declaration_index: usize,
    /// The index of this field as seen by the reflection API.
    ///
    /// This index accounts for the removal of [ignored] fields.
    /// It will only be `Some(index)` when the field is not ignored.
    ///
    /// [ignored]: crate::field_attributes::ReflectIgnoreBehavior::IgnoreAlways
    pub reflection_index: Option<usize>,
    /// The documentation for this field, if any
    #[cfg(feature = "reflect_docs")]
    pub doc: crate::reflect_docs::ReflectDocs,
}

impl<'a> StructField<'a> {
    /// Generates a `TokenStream` for `NamedField` or `UnnamedField` construction.
    pub fn to_info_tokens(&self, vct_reflect_path: &Path) -> proc_macro2::TokenStream {
        let name = match &self.data.ident {
            Some(ident) => ident.to_string().to_token_stream(),
            None => self.reflection_index.to_token_stream(),
        };

        let field_info_path = if self.data.ident.is_some() {
            crate::path::named_field_(vct_reflect_path)
        } else {
            crate::path::unnamed_field_(vct_reflect_path)
        };

        let ty = self.reflected_type();

        let mut info = quote! {
            #field_info_path::new::<#ty>(#name)
        };

        let custom_attributes = &self.attrs.custom_attributes;
        if !custom_attributes.is_empty() {
            let custom_attributes = custom_attributes.to_tokens(vct_reflect_path);
            info.extend(quote! {
                .with_custom_attributes(#custom_attributes)
            });
        }

        #[cfg(feature = "reflect_docs")]
        {
            let docs = &self.doc;
            if !docs.is_empty() {
                info.extend(quote! {
                    .with_docs(#docs)
                });
            }
        }

        info
    }

    /// Returns the reflected type of this field.
    ///
    /// Normally this is just the field's defined type.
    /// However, this can be adjusted to use a different type, like for representing remote types.
    /// In those cases, the returned value is the remote wrapper type.
    pub fn reflected_type(&self) -> &Type {
        self.attrs.alias.as_ref().unwrap_or(&self.data.ty)
    }

    pub fn attrs(&self) -> &FieldAttributes {
        &self.attrs
    }

    /// Generates a [`Member`] based on this field.
    ///
    /// If the field is unnamed, the declaration index is used.
    /// This allows this member to be used for both active and ignored fields.
    pub fn to_member(&self) -> Member {
        match &self.data.ident {
            Some(ident) => Member::Named(ident.clone()),
            None => Member::Unnamed(self.declaration_index.into()),
        }
    }

    /// Returns a token stream for generating a `FieldId` for this field.
    pub fn field_id(&self, vct_reflect_path: &Path) -> proc_macro2::TokenStream {
        let field_id_path = crate::path::field_id_(vct_reflect_path);
        match &self.data.ident {
            Some(ident) => {
                let alloc_path = crate::path::alloc_utils_(vct_reflect_path);
                let name = ident.to_string();
                quote!(#field_id_path::Named(#alloc_path::Cow::Borrowed(#name)))
            }
            None => {
                let index = self.declaration_index;
                quote!(#field_id_path::Unnamed(#index))
            }
        }
    }
}


