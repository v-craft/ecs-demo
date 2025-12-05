use proc_macro2::TokenStream;

use crate::{derive_data::ReflectStruct, impls::{impl_trait_type_path, impl_trait_typed}};



pub(crate) fn impl_tuple_struct(info: &ReflectStruct) -> TokenStream {
    let meta = info.meta();

    // trait: TypePath
    let type_path_trait_tokens = if meta.attrs().impl_switchs.impl_type_path {
        impl_trait_type_path(meta)
    } else {
        crate::utils::empty()
    };
    
    // trait: Typed
    let typed_trait_tokens = if meta.attrs().impl_switchs.impl_typed {
        impl_trait_typed(meta, info.to_info_tokens(true))
    } else {
        crate::utils::empty()
    };

    todo!()
}
