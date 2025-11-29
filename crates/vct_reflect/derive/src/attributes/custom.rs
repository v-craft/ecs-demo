use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::ParseStream, Expr, Path, Token};

#[derive(Default, Clone)]
pub(crate) struct CustomAttributes {
    attributes: Vec<Expr>,
}

impl CustomAttributes {
    /// Generates a `TokenStream` for `CustomAttributes` construction.
    pub fn to_tokens(&self, vct_reflect_path: &Path) -> TokenStream {
        let attributes = self.attributes.iter().map(|value| {
            quote! {
                .with_attribute(#value)
            }
            // `with_attribute`: See vct_reflect::info::attribute.rs
        });
        let custom_attributes_path = crate::path::custom_attributes_(vct_reflect_path);
        
        quote! {
            #custom_attributes_path::default()
                #(#attributes)*
        }
    }

    /// Inserts a custom attribute into the list.
    pub fn push(&mut self, value: Expr) -> syn::Result<()> {
        self.attributes.push(value);
        Ok(())
    }

    /// Is the collection empty?
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }

    /// Parse `@` (custom attribute) attribute.
    ///
    /// Examples:
    /// - `#[reflect(@Foo))]`
    /// - `#[reflect(@Bar::baz("qux"))]`
    /// - `#[reflect(@0..256u8)]`
    pub fn parse_custom_attribute(&mut self, input: ParseStream) -> syn::Result<()> {
        input.parse::<Token![@]>()?;
        self.push(input.parse()?)
    }
}




