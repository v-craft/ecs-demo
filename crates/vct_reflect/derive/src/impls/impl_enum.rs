use proc_macro2::TokenStream;

use crate::{derive_data::ReflectEnum, impls::{impl_trait_type_path, impl_trait_typed}};



pub(crate) fn impl_enum(info: &ReflectEnum) -> TokenStream {
    let meta = info.meta();

    // trait: TypePath
    let type_path_trait_tokens = if meta.attrs().impl_switchs.impl_type_path {
        impl_trait_type_path(meta)
    } else {
        crate::utils::empty()
    };
    
    // trait: Typed
    let typed_trait_tokens = if meta.attrs().impl_switchs.impl_typed {
        impl_trait_typed(meta, info.to_info_tokens())
    } else {
        crate::utils::empty()
    };

    todo!()
}
