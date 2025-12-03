use proc_macro2::TokenStream;

use crate::derive_data::{ReflectMeta, ReflectEnum};



pub(crate) fn impl_trait_enum(info: &ReflectEnum) -> TokenStream {
    if !info.meta.attrs().trait_flags.impl_reflect {
        return crate::utils::empty();
    }
    
    todo!()
}
