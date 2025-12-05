use syn::{Path, Token, punctuated::Punctuated};
use quote::quote;
use crate::derive_data::{
    TypeAttributes, TypePathParser,
};

pub(crate) struct ReflectMeta<'a> {
    /// The registered traits for this type.
    attrs: TypeAttributes,
    /// The path to this type.
    type_path_parser: TypePathParser<'a>,
    /// A cached instance of the path to the `vct_reflect` crate.
    vct_reflect_path: Path,
}

impl<'a> ReflectMeta<'a> {
    pub fn new(attrs: TypeAttributes, type_path_parser: TypePathParser<'a>) -> Self {
        Self {
            attrs,
            type_path_parser,
            vct_reflect_path: crate::path::vct_reflect(),
        }
    }

    pub fn vct_reflect_path(&self) -> &Path {
        &self.vct_reflect_path
    }

    pub fn type_path_parser(&self) -> &TypePathParser<'a> {
        &self.type_path_parser
    }

    pub fn attrs(&self) -> &TypeAttributes {
        &self.attrs
    }

    pub fn with_docs_expression(&self) -> proc_macro2::TokenStream {
        self.attrs.docs.get_expression_with()
    }

    pub fn with_custom_attributes_expression(&self) -> proc_macro2::TokenStream {
        self.attrs.custom_attributes.get_expression_with(&self.vct_reflect_path)
    }

    pub fn with_generics_expression(&self) -> proc_macro2::TokenStream{
        let vct_reflect_path = &self.vct_reflect_path;
        let generics_ = crate::path::generics_(vct_reflect_path);
        let generic_info_ = crate::path::generic_info_(vct_reflect_path);
        let type_param_info_ = crate::path::type_param_info_(vct_reflect_path);
        let const_param_info_ = crate::path::const_param_info_(vct_reflect_path);
        let alloc_utils_ = crate::path::alloc_utils_(vct_reflect_path);

        let generics = self.type_path_parser.generics().params.iter().filter_map(|param| {
            match param {
                syn::GenericParam::Lifetime(_) => None,
                syn::GenericParam::Type(type_param) => {
                    let ident = &type_param.ident;
                    let name = ident.to_string();
                    let with_default = type_param.default.as_ref()
                        .map(|default_ty|quote!(.with_default::<#default_ty>()));

                    Some(quote! {
                        #generic_info_::Type(
                            #type_param_info_::new::<#ident>( 
                                #alloc_utils_::Cow::Borrowed( #name ) 
                            ) 
                            #with_default
                        )
                    })
                },
                syn::GenericParam::Const(const_param) => {
                    let ty = &const_param.ty;
                    let name = const_param.ident.to_string();
                    let with_default = const_param.default.as_ref()
                        .map(|default| quote!(.with_default(#default as #ty)) ); 
                    // use `as` to ensure type correction.

                    Some(quote! {
                        #[allow(clippy::unnecessary_cast, reason = "Const generics require explicit type hint.")]
                        #generic_info_::Const(
                            #const_param_info_::new::<#ty>(
                                #alloc_utils_::Cow::Borrowed(#name),
                            )
                            #with_default
                        )
                    })
                },
            }
        }).collect::<Punctuated<_, Token![,]>>();

        if generics.is_empty() {
            return crate::utils::empty();
        }

        quote!{
            .with_generics(
                #generics_::from_iter([ #generics ])
            )
        }
    }

    /// For Opaque Type
    pub fn to_info_tokens(&self) -> proc_macro2::TokenStream {
        let vct_reflect_path = &self.vct_reflect_path;

        let opaque_info_ = crate::path::opaque_info_(vct_reflect_path);
        let type_info_ = crate::path::type_info_(vct_reflect_path);
        let with_custom_attributes = self.with_custom_attributes_expression();
        let with_docs = self.with_docs_expression();
        let with_generics = self.with_generics_expression();

        quote! {
            #type_info_::Opaque(
                #opaque_info_::new::<Self>()
                    #with_custom_attributes
                    #with_generics
                    #with_docs
            )
        }
    }
}

