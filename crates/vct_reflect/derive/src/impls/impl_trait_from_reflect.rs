use crate::derive_data::ReflectStruct;
use quote::{ToTokens, quote};
use syn::Ident;
use proc_macro2::Span;


pub(crate) fn impl_struct_from_reflect(info: &ReflectStruct, is_tuple: bool) -> proc_macro2::TokenStream {
    use crate::path::fp::{OptionFP, CloneFP};

    let meta = info.meta();
    let vct_reflect_path = meta.vct_reflect_path();
    let from_reflect_ = crate::path::from_reflect_(vct_reflect_path);
    let reflect_ = crate::path::reflect_(vct_reflect_path);
    let reflect_ref_ = crate::path::reflect_ref_(vct_reflect_path);
    // let option_ = OptionFP.to_token_stream();

    let parser = meta.type_path_parser();
    let real_ident = parser.real_ident();
    let (impl_generics, ty_generics, where_clause) = parser.generics().split_for_impl();

    let clone_tokens = if meta.attrs().avail_traits.clone {
        quote! {
            if let Some(val) = <dyn #reflect_>::downcast_ref::<Self>(value) {
                return #OptionFP::Some(#CloneFP::clone(val));
            }
        }
    } else {
        crate::utils::empty()
    };

    let ref_struct = Ident::new("__ref_struct", Span::call_site());
    let ref_struct_type = if is_tuple {
        Ident::new("TupleStruct", Span::call_site())
    } else {
        Ident::new("Struct", Span::call_site())
    };

    // TODO
    let constructor= quote! ( #OptionFP::None );

    quote! {
        impl #impl_generics #from_reflect_ for #real_ident #ty_generics #where_clause  {
            fn from_reflect(value: &dyn #reflect_) -> #OptionFP<Self> {

                #clone_tokens

                if <dyn #reflect_>::is::<Self>(value) {
                    if let Ok(cloned) = #reflect_::reflect_clone(value)
                        && Ok(val) = <dyn #reflect_>::take::<Self>(cloned)
                    {
                        return #OptionFP::Some(*val);
                    }
                }

                if let #reflect_ref_::#ref_struct_type(#ref_struct) = #reflect_::reflect_ref(reflect) {
                    #constructor
                } else {
                    #OptionFP::None
                }
            }
        }
    }
}







