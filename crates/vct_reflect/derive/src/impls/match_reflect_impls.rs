use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;
use crate::{ImplSourceKind, derive_data::ReflectDerive};

pub(crate) fn match_reflect_impls(ast: DeriveInput, source: ImplSourceKind) -> TokenStream {
    let reflect_derive = match ReflectDerive::from_input(&ast, source) {
        Ok(val) => val,
        Err(err) => return err.into_compile_error().into(),
    };

    let real_ident = reflect_derive.meta().type_path_parser().real_ident();

    let assert_ident = quote! {
        mod __reflect_assert_ident {
            use super::*;
            type AssertIdentValidity = #real_ident;
        }
    };

    let reflect_impls: proc_macro2::TokenStream = match reflect_derive {
        ReflectDerive::Struct(info) => crate::impls::impl_struct(&info),
        ReflectDerive::TupleStruct(info) => crate::impls::impl_tuple_struct(&info),
        ReflectDerive::Enum(info) => crate::impls::impl_enum(&info),
        ReflectDerive::UnitStruct(meta) => crate::impls::impl_unit(&meta),
        ReflectDerive::Opaque(meta) => crate::impls::impl_opaque(&meta),
    };

    let res = quote! {
        const _: () = {
            #assert_ident

            #reflect_impls
        };
    };


    // eprintln!("{}", res);

    res.into()
}

