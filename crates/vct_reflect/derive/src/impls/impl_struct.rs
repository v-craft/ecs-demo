use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

use crate::{derive_data::{FieldAccessors, ReflectMeta, ReflectStruct}, impls::{impl_struct_from_reflect, impl_trait_get_type_traits, impl_trait_reflect, impl_trait_type_path, impl_trait_typed}};



pub(crate) fn impl_struct(info: &ReflectStruct) -> TokenStream {
    let meta = info.meta();

    // trait: TypePath
    let type_path_trait_tokens = if meta.attrs().impl_switchs.impl_type_path {
        impl_trait_type_path(meta)
    } else {
        crate::utils::empty()
    };
    
    // trait: Typed
    let typed_trait_tokens = if meta.attrs().impl_switchs.impl_typed {
        impl_trait_typed(meta, info.to_info_tokens(false))
    } else {
        crate::utils::empty()
    };

    // trait: struct
    let struct_trait_tokens = if meta.attrs().impl_switchs.impl_struct {
        impl_trait_struct(info)
    } else {
        crate::utils::empty()
    };

    // trait: Reflect
    let reflect_trait_tokens = if meta.attrs().impl_switchs.impl_reflect {
        let try_apply_tokens = get_struct_try_apply_impl(meta);
        let to_dynamic_tokens = get_struct_to_dynamic_impl(meta);
        let reflect_clone_tokens = get_struct_clone_impl(info);
        let reflect_partial_eq_tokens = get_struct_partial_eq_impl(meta);
        let reflect_hash_tokens = get_struct_hash_impl(meta);
        let reflect_debug_tokens = get_struct_debug_impl(meta);

        impl_trait_reflect(
            meta, 
            quote!(Struct),
            try_apply_tokens,
            to_dynamic_tokens,
            reflect_clone_tokens,
            reflect_partial_eq_tokens,
            reflect_hash_tokens,
            reflect_debug_tokens,
        )
    } else {
        crate::utils::empty()
    };

    // trait: GetTypeTraits
    let get_type_traits_tokens = if meta.attrs().impl_switchs.impl_get_type_traits {
        impl_trait_get_type_traits(meta, crate::utils::empty())
    } else {
        crate::utils::empty()
    };

    let get_from_reflect_tokens = if meta.attrs().impl_switchs.impl_from_reflect {
        impl_struct_from_reflect(info, false)
    } else {
        crate::utils::empty()
    };

    quote! {
        #type_path_trait_tokens

        #typed_trait_tokens

        #struct_trait_tokens

        #reflect_trait_tokens

        #get_type_traits_tokens

        #get_from_reflect_tokens
    }
}

pub fn impl_trait_struct(info: &ReflectStruct) -> TokenStream {
    use crate::path::fp::OptionFP;
    let meta = info.meta();
    
    let vct_reflect_path = meta.vct_reflect_path();
    let struct_ = crate::path::struct_(vct_reflect_path);
    let reflect_ = crate::path::reflect_(vct_reflect_path);
    let struct_field_iter_ = crate::path::struct_field_iter_(vct_reflect_path);
    let dynamic_struct_ = crate::path::dynamic_struct_(vct_reflect_path);
    let option_ = OptionFP.to_token_stream();

    let field_names = info
        .active_fields()
        .map(|field| {
            field
                .data
                .ident
                .as_ref()
                .map(ToString::to_string)
                .expect("Struct should not have unnamed fields.")
        })
        .collect::<Vec<String>>();

    let FieldAccessors {
        fields_ref,
        fields_mut,
        field_indices,
        field_count,
    } = FieldAccessors::new(info);

    let parser = meta.type_path_parser();
    let real_ident = parser.real_ident();
    let (impl_generics, ty_generics, where_clause) = parser.generics().split_for_impl();

    quote! {
        impl #impl_generics #struct_ for #real_ident #ty_generics #where_clause {
            fn field(&self, name: &str) -> #OptionFP<&dyn #reflect_> {
                match name {
                    #(#field_names => #option_::Some(#fields_ref),)*
                    _ => #OptionFP::None,
                }
            }

            fn field_mut(&mut self, name: &str) -> #OptionFP<&mut dyn #reflect_> {
                match name {
                    #(#field_names => #option_::Some(#fields_mut),)*
                    _ => #OptionFP::None,
                }
            }

            fn field_at(&self, index: usize) -> #OptionFP<&dyn #reflect_> {
                match index {
                    #(#field_indices => #option_::Some(#fields_ref),)*
                    _ => #OptionFP::None,
                }
            }

            fn field_at_mut(&mut self, index: usize) -> #OptionFP<&mut dyn #reflect_> {
                match index {
                    #(#field_indices => #option_::Some(#fields_mut),)*
                    _ => #OptionFP::None,
                }
            }

            fn name_at(&self, index: usize) -> #OptionFP<&str> {
                match index {
                    #(#field_indices => #option_::Some(#field_names),)*
                    _ => #OptionFP::None,
                }
            }

            fn field_len(&self) -> usize {
                #field_count
            }

            fn iter_fields(&self) -> #struct_field_iter_ {
                #struct_field_iter_::new(self)
            }

            fn to_dynamic_struct(&self) -> #dynamic_struct_ {
                let mut dynamic = #dynamic_struct_::with_capacity(#struct_::field_len(self));
                dynamic.set_type_info(#reflect_::represented_type_info(self));
                #(dynamic.insert_boxed(#field_names, #reflect_::to_dynamic(#fields_ref));)*
                dynamic
            }
        }
    }
}

pub fn get_struct_try_apply_impl(meta: &ReflectMeta) -> TokenStream {
    use crate::path::fp::{ResultFP, OptionFP, CloneFP};

    let vct_reflect_path = meta.vct_reflect_path();
    let reflect_ = crate::path::reflect_(vct_reflect_path);
    let reflect_ref = crate::path::reflect_ref_(vct_reflect_path);
    let struct_ = crate::path::struct_(vct_reflect_path);
    let reflect_kind_ = crate::path::reflect_kind_(vct_reflect_path);
    let apply_error_ = crate::path::apply_error_(vct_reflect_path);

    if meta.attrs().avail_traits.clone {
        quote! {
            fn try_apply(&mut self, value: &dyn #reflect_) -> #ResultFP<(), #apply_error_> {
                if let #OptionFP::Some(value) = <dyn #reflect_>::downcast_ref::<Self>(value) {
                    *self = #CloneFP::clone(value);
                    return #ResultFP::Ok(());
                }
                if let #reflect_ref::Struct(struct_value) = #reflect_::reflect_ref(value) {
                    for (i, value) in ::core::iter::Iterator::enumerate(#struct_::iter_fields(struct_value)) {
                        let name = #struct_::name_at(struct_value, i).unwrap();
                        if let #OptionFP::Some(v) = #struct_::field_mut(self, name) {
                           #reflect_::try_apply(v, value)?;
                        }
                    }
                } else {
                    return #ResultFP::Err(
                        #apply_error_::MismatchedKinds {
                            from_kind: #reflect_::reflect_kind(value),
                            to_kind: #reflect_kind_::Struct,
                        }
                    );
                }
                #ResultFP::Ok(())
            }
        }
    } else {
        quote! {
            fn try_apply(&mut self, value: &dyn #reflect_) -> #ResultFP<(), #apply_error_> {
                if let #reflect_ref::Struct(struct_value) = #reflect_::reflect_ref(value) {
                    for (i, value) in ::core::iter::Iterator::enumerate(#struct_::iter_fields(struct_value)) {
                        let name = #struct_::name_at(struct_value, i).unwrap();
                        if let #OptionFP::Some(v) = #struct_::field_mut(self, name) {
                           #reflect_::try_apply(v, value)?;
                        }
                    }
                } else {
                    return #ResultFP::Err(
                        #apply_error_::MismatchedKinds {
                            from_kind: #reflect_::reflect_kind(value),
                            to_kind: #reflect_kind_::Struct,
                        }
                    );
                }
                #ResultFP::Ok(())
            }
        }
    }

}

fn get_struct_to_dynamic_impl(meta: &ReflectMeta) -> TokenStream {
    let vct_reflect_path = meta.vct_reflect_path();
    let alloc_utils_ = crate::path::alloc_utils_(vct_reflect_path);
    let reflect_ = crate::path::reflect_(vct_reflect_path);
    let struct_ = crate::path::struct_(vct_reflect_path);

    quote! {
        #[inline]
        fn to_dynamic(&self) -> #alloc_utils_::Box<dyn #reflect_> {
            #alloc_utils_::Box::new( #struct_::to_dynamic_struct(self) )
        }
    }
}

fn get_struct_clone_impl(info: &ReflectStruct) -> TokenStream {
    use crate::path::fp::{ResultFP, CloneFP, OptionFP};

    let meta = info.meta();
    let vct_reflect_path = meta.vct_reflect_path();
    let alloc_utils_ = crate::path::alloc_utils_(vct_reflect_path);
    let macro_exports_ = crate::path::macro_exports_(vct_reflect_path);
    let reflect_ = crate::path::reflect_(vct_reflect_path);
    let reflect_clone_error_ = crate::path::reflect_clone_error_(vct_reflect_path);
    let type_path_ = crate::path::type_path_(vct_reflect_path);


    if meta.attrs().avail_traits.clone {
        quote! {
            #[inline]
            fn reflect_clone(&self) -> #ResultFP<#alloc_utils_::Box<dyn #reflect_>, #reflect_clone_error_> {
                #ResultFP::Ok(#alloc_utils_::Box::new(<Self as #CloneFP>::clone(self)) as #alloc_utils_::Box<dyn #reflect_>)
            }
        }
    } else {
        for field in info.fields().iter() {
            if field.attrs.ignore {
                let field_id = field.field_id(vct_reflect_path);
                return quote! {
                    #[inline]
                    fn reflect_clone(&self) -> #ResultFP<#alloc_utils_::Box<dyn #reflect_>, #reflect_clone_error_> {
                        #ResultFP::Err(#reflect_clone_error_::FieldNotCloneable {
                            type_path:  #alloc_utils_::Cow::Borrowed(<Self as #type_path_>::type_path())
                            field: #field_id,
                            variant: #OptionFP::None,
                        })
                    }
                };
            }
        }

        let mut tokens = TokenStream::new();

        for field in info.fields().iter() {
            let field_ty = &field.data.ty;
            let member = field.to_member();
            let accessor = info.access_for_field(field, false);

            tokens.extend(quote! {
                #member: #macro_exports_::reflect_clone_field::<#field_ty>(#accessor)?,
            });
        }

        quote! {
            #[inline]
            fn reflect_clone(&self) -> #ResultFP<#alloc_utils_::Box<dyn #reflect_>, #reflect_clone_error_> {
                #ResultFP::Ok(#alloc_utils_::Box::new(
                    Self {
                        #tokens
                    }
                ) as #alloc_utils_::Box<dyn #reflect_>)
            }
        }
    }
}

fn get_struct_partial_eq_impl(meta: &ReflectMeta) -> TokenStream  {
    use crate::path::fp::{OptionFP, PartialEqFP};
    let vct_reflect_path = meta.vct_reflect_path();
    let reflect_ = crate::path::reflect_(vct_reflect_path);

    if meta.attrs().avail_traits.partial_eq {
        quote! {
            #[inline]
            fn reflect_partial_eq(&self, other: &dyn #reflect_) -> #OptionFP<bool> {
                if let #OptionFP::Some(value) = other.downcast_ref::<Self>() {
                    return #OptionFP::Some( #PartialEqFP::eq(self, value) );
                }
                #OptionFP::None
            }
        }
    } else {
        crate::utils::empty()
    }
}

fn get_struct_hash_impl(meta: &ReflectMeta) -> TokenStream {
    use crate::path::fp::{OptionFP, HashFP, HasherFP};

    let vct_reflect_path = meta.vct_reflect_path();
    let reflect_hasher = crate::path::reflect_hasher_(vct_reflect_path);

    if meta.attrs().avail_traits.hash {
        quote! {
            #[inline]
            fn reflect_hash(&self) -> #OptionFP<u64> {
                let mut hasher = #reflect_hasher();
                <Self as #HashFP>::hash(self, &mut hasher);
                #OptionFP::Some(#HasherFP::finish(&hasher))
            }
        }
    } else {
        crate::utils::empty()
    }
}

fn get_struct_debug_impl(meta: &ReflectMeta) -> TokenStream {
    use crate::path::fp::DebugFP;

    if meta.attrs().avail_traits.debug {
        quote! {
            #[inline]
            fn reflect_debug(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                <Self as #DebugFP>::fmt(self, f)
            }
        }
    } else {
        crate::utils::empty()
    }
}


