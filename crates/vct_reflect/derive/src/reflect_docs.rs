//! Contains code related to documentation reflection (requires the `reflect_docs` feature).

use crate::utils::fp::OptionFP;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Attribute, Expr, ExprLit, Lit, Meta};

#[derive(Default, Clone)]
pub(crate) struct ReflectDocs {
    docs: Vec<String>,
}

impl ToTokens for ReflectDocs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(doc) = self.doc_string() {
            quote!(#OptionFP::Some(#doc)).to_tokens(tokens);
        } else {
            quote!(#OptionFP::None).to_tokens(tokens);
        }
    }
}

impl ReflectDocs {
    pub fn doc_string(&self) -> Option<String> {
        if self.docs.is_empty() {
            return None;
        }
        let size: usize = self.docs.iter().map(String::len).sum();
        let mut s = String::with_capacity(size + self.docs.len() - 1);

        let mut it = self.docs.iter();
        s.push_str(it.next().unwrap());
        for item in it {
            s.push('\n');
            s.push_str(item);
        }
        Some(s)
    }

    /// Is the collection empty?
    pub fn is_empty(&self) -> bool {
        self.docs.is_empty()
    }

    /// Push a new docstring to the collection
    pub fn push(&mut self, doc: String) {
        self.docs.push(doc);
    }

    /// Create a new [`ReflectDocs`] from a type's attributes.
    ///
    /// This will collect all `#[doc = "..."]` attributes, 
    /// including the ones generated via `///` and `//!`.
    pub fn from_attributes<'a>(attributes: impl IntoIterator<Item = &'a Attribute>) -> Self {
        let docs = attributes
            .into_iter()
            .filter_map(|attr| match &attr.meta {
                Meta::NameValue(pair) if pair.path.is_ident("doc")  => {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(lit), ..
                    }) = &pair.value {
                        Some(lit.value())
                    } else {
                        None
                    }
                },
                _ => None,
            })
            .collect();
        Self { docs }
    }
}
