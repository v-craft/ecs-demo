use proc_macro2::TokenStream;

use crate::derive_data::{ReflectMeta, ReflectStruct};



pub(crate) fn impl_trait_struct(info: &ReflectStruct) -> TokenStream {
    if !info.meta.attrs().trait_flags.impl_struct {
        return crate::utils::empty();
    }
    
    todo!()
}

