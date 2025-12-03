use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;
use crate::{ImplSourceKind, derive_data::ReflectDerive};

pub(crate) fn match_reflect_impls(ast: DeriveInput, source: ImplSourceKind) -> TokenStream {
    let reflect_derive = match ReflectDerive::from_input(&ast, source) {
        Ok(val) => val,
        Err(err) => return err.into_compile_error().into(),
    };

    let reflect_impls: proc_macro2::TokenStream = match reflect_derive {
        ReflectDerive::Struct(_) => todo!(),
        ReflectDerive::TupleStruct(_) => todo!(),
        ReflectDerive::Enum(_) => todo!(),
        ReflectDerive::UnitStruct(_) => todo!(),
        ReflectDerive::Opaque(meta) => crate::impls::impl_opaque(&meta),
    };

    quote! {
        const _: () = {
            #reflect_impls
        }
    }.into()
}

