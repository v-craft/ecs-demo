use proc_macro2::TokenStream;
use quote::quote;
use crate::derive_data::ReflectMeta;



pub(crate) fn impl_trait_get_type_traits<'a>(meta: &ReflectMeta, register_deps_tokens: TokenStream) -> TokenStream {
    debug_assert!(meta.attrs().impl_switchs.impl_get_type_traits);

    let vct_reflect_path = meta.vct_reflect_path();

    let get_type_traits_ = crate::path::get_type_traits_(vct_reflect_path);
    let type_traits_ = crate::path::type_traits_(vct_reflect_path);
    let from_type_ = crate::path::from_type_(vct_reflect_path);
    let type_trait_from_ptr = crate::path::type_trait_from_ptr_(vct_reflect_path);
    let type_trait_from_reflect = crate::path::type_trait_from_reflect_(vct_reflect_path);
    
    let insert_default = if meta.attrs().avail_traits.default {
        let type_trait_default_ = crate::path::type_trait_default_(vct_reflect_path);
        quote! {
            #type_traits_::insert::<#type_trait_default_>(&mut type_traits, #from_type_::<Self>::from_type());
        }
    } else {
        crate::utils::empty()
    };

    let insert_serialize = if meta.attrs().avail_traits.serialize {
        let type_trait_serialize_ = crate::path::type_trait_serialize_(vct_reflect_path);
        quote! {
            #type_traits_::insert::<#type_trait_serialize_>(&mut type_traits, #from_type_::<Self>::from_type());
        }
    } else {
        crate::utils::empty()
    };

    let insert_deserialize = if meta.attrs().avail_traits.deserialize {
        let type_trait_deserialize_ = crate::path::type_trait_deserialize_(vct_reflect_path);
        quote! {
            #type_traits_::insert::<#type_trait_deserialize_>(&mut type_traits, #from_type_::<Self>::from_type());
        }
    } else {
        crate::utils::empty()
    };

    let parser = meta.type_path_parser();
    let real_ident = parser.real_ident();
    let (impl_generics, ty_generics, where_clause) = parser.generics().split_for_impl();

    quote! {
        impl #impl_generics #get_type_traits_ for #real_ident #ty_generics #where_clause {
            fn get_type_traits() -> #type_traits_ {
                let mut type_traits = #type_traits_::of::<Self>();
                #type_traits_::insert::<#type_trait_from_ptr>(&mut type_traits, #from_type_::<Self>::from_type());
                #type_traits_::insert::<#type_trait_from_reflect>(&mut type_traits, #from_type_::<Self>::from_type());
                #insert_default
                #insert_serialize
                #insert_deserialize
                type_traits
            }

            #register_deps_tokens
        }
    }
}
