use proc_macro2::TokenStream;
use quote::quote;
use crate::path::fp::OptionFP;

pub(crate) fn wrap_in_option(tokens: Option<TokenStream>) -> TokenStream {
    match tokens {
        Some(tokens) => quote! {
            #OptionFP::Some(#tokens)
        },
        None => quote! {
            #OptionFP::None
        },
    }
}

