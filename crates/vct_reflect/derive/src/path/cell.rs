use proc_macro2::TokenStream;
use quote::quote;

#[inline(always)]
pub(crate) fn non_generic_type_info_cell_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::cell::NonGenericTypeInfoCell
    }
}

// #[inline(always)]
// pub(crate) fn non_generic_type_path_cell_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::cell::NonGenericTypePathCell
//     }
// }

#[inline(always)]
pub(crate) fn generic_type_info_cell_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::cell::GenericTypeInfoCell
    }
}

#[inline(always)]
pub(crate) fn generic_type_path_cell_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::cell::GenericTypePathCell
    }
}
