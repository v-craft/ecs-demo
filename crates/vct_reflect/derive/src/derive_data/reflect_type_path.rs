use quote::{ToTokens, quote};
use syn::{Ident, Path, Generics, punctuated::Punctuated, GenericParam, spanned::Spanned, LitStr, TypeParam};
use crate::utils::StringExpr;


pub(crate) enum TypePathParser<'a> {
    /// Types without a crate/module that can be named from any scope (e.g. `bool`).
    Primitive(&'a Ident),
    /// The type must be able to be reached with just its ident.
    /// 
    /// For local types, can use [`module_path!()`](module_path) to get the module path.
    Local {
        ident: &'a Ident,
        custom_path: Option<Path>,
        generics: &'a Generics,
    },
    /// For foreign, [`module_path!()`](module_path) can not be used.
    /// So the user needs to provide the complete path, using `::my_crate::foo::Bar` syntax.
    Foreign {
        path: &'a Path,
        custom_path: Option<Path>,
        generics: &'a Generics,
    },
}

impl<'a> TypePathParser<'a> {
    // /// return `true` if self is `Local/Foreign` ans has custom path
    // pub fn has_custom_path(&self) -> bool {
    //     match self {
    //         Self::Local{ custom_path, .. } | Self::Foreign{ custom_path, .. } => {
    //             custom_path.is_some()
    //         }
    //         _ => false,
    //     }
    // }

    pub fn generics(&self) -> &'a Generics {
        // Use a constant because we need to return a reference of at least 'a.
        const EMPTY_GENERICS: &Generics = &Generics {
            gt_token: None,
            lt_token: None,
            where_clause: None,
            params: Punctuated::new(),
        };

        match self {
            Self::Local { generics, .. } | Self::Foreign { generics, .. } => generics,
            _ => EMPTY_GENERICS,
        }
    }

    /// Whether an implementation of `Typed` or `TypePath` should be generic.
    pub fn impl_with_generic(&self) -> bool {
        // exist non-lifecycle generic parameters 
        !self
            .generics()
            .params
            .iter()
            .all(|param| matches!(param, GenericParam::Lifetime(_)))
    }

    /// This name is used in `impl ... for #real_ident {...}`.
    pub fn real_ident(&self) -> proc_macro2::TokenStream {
        match self {
            Self::Local { ident, .. } | Self::Primitive(ident) => ident.to_token_stream(),
            Self::Foreign { path, .. } => path.to_token_stream(),
        }
    }

    /// Try to get (custom) ident
    pub fn get_ident(&self) -> &Ident {
        match self {
            Self::Primitive(ident) => ident,
            Self::Local{ ident, custom_path, .. } => {
                custom_path
                    .as_ref()
                    .map(|path| &path.segments.last().unwrap().ident)
                    .unwrap_or(ident)
            },
            Self::Foreign{ path, custom_path, .. } => {
                &custom_path
                    .as_ref()
                    .unwrap_or(path)
                    .segments
                    .last()
                    .unwrap()
                    .ident
            },
        }
    }

    /// Try to get full (custom) path.
    pub fn get_path(&self) -> Option<&Path> {
        match self {
            Self::Local{ custom_path, .. } => custom_path.as_ref(),
            Self::Foreign{ path, custom_path, .. } => Some(
                custom_path.as_ref().unwrap_or(path)
            ),
            _ => None,
        }
    }

    pub fn crate_name(&self) -> Option<StringExpr> {
        if let Some(path) = self.get_path() {
            let crate_name = &path.segments.first().unwrap_or_else(||{
                panic!("If Path/CustomPath is exist, can not be empty.");
            }).ident;
            return Some(StringExpr::from(crate_name));
        }

        match self {
            Self::Local { .. } => Some(StringExpr::Borrowed(quote! {
                ::core::module_path!().split(':').next().unwrap()
            })),
            _ => None,
        }
    }

    pub fn module_path(&self) -> Option<StringExpr> {
        if let Some(path) = self.get_path() {
            let path_string = path
                .segments
                .iter()
                .take(path.segments.len() - 1)
                .map(|segment| segment.ident.to_string())
                .reduce(|path, ident| path + "::" + &ident)
                .unwrap_or_else(||panic!("If Path/CustomPath is exist, can not be empty."));

            let path_lit = LitStr::new(&path_string, path.span());
            return Some(StringExpr::from_lit(&path_lit));
        }

        match self {
            Self::Local { .. } => Some(StringExpr::Const(quote! {
                ::core::module_path!()
            })),
            _ => None,
        }
    }

    pub fn type_ident(&self) -> StringExpr {
        StringExpr::from(self.get_ident())
    }

    /// Combines type generics and const generics into one [`StringExpr`].
    ///
    /// This string can be used with a `GenericTypePathCell` in a `TypePath` implementation.
    ///
    /// The `ty_generic_fn` param maps [`TypeParam`]s to [`StringExpr`]s.
    fn reduce_generics(
        generics: &Generics,
        mut ty_generic_fn: impl FnMut(&TypeParam) -> StringExpr,
        vct_reflect_path: &Path,
    ) -> StringExpr {
        let alloc_utils_path = crate::path::alloc_utils_(vct_reflect_path);

        let mut params = generics.params.iter().filter_map(|param| match param {
            GenericParam::Type(type_param) => Some(ty_generic_fn(type_param)),
            GenericParam::Const(const_param) => {
                let ident = &const_param.ident;
                let ty = &const_param.ty;

                Some(StringExpr::Owned(quote! {
                    <#ty as #alloc_utils_path::ToString>::to_string(&#ident)
                }))
            }
            GenericParam::Lifetime(_) => None,
        });

        let first = params.next().into_iter();

        StringExpr::from_iter(
            first.chain(params.flat_map(|x| [StringExpr::from_str(", "), x])), 
            vct_reflect_path
        )
    }

    /// Get type_name, see `TypePath::type_name`
    /// For `core::option::Option<core::marker::PhantomData>`, this is `"Option<PhantomData>"`.
    pub fn type_name(&self, vct_reflect_path: &Path) -> StringExpr {
        match self {
            Self::Primitive(ident) => StringExpr::from(ident),
            Self::Local{ generics, .. } | Self::Foreign{ generics, .. } => {
                let ident = self.type_ident();

                if self.impl_with_generic() {
                    let type_path_ = crate::path::type_path_(vct_reflect_path);

                    let generics = TypePathParser::reduce_generics(
                        generics,
                        |TypeParam { ident, .. }| {
                            StringExpr::Borrowed(quote! {
                                <#ident as #type_path_>::type_name()
                            })
                        },
                        vct_reflect_path,
                    );

                    StringExpr::from_iter([
                        ident,
                        StringExpr::from_str("<"),
                        generics,
                        StringExpr::from_str(">"),
                    ], vct_reflect_path)
                } else {
                    ident
                }
            }
        }
    }

    /// Returns a [`StringExpr`] representing the "type path" of the type.
    ///
    /// For `Option<PhantomData>`, this is `"core::option::Option<core::marker::PhantomData>"`.
    pub fn type_path(&self, vct_reflect_path: &Path) -> StringExpr {
        match self {
            Self::Primitive(ident) => StringExpr::from(ident),
            Self::Local{ generics, .. } | Self::Foreign{ generics, .. } => {
                let ident = self.type_ident();
                let module_path = self.module_path().unwrap_or_else(||{
                    panic!("Non-Primitive type, try to parse type_path but get module_path fail.");
                });

                if self.impl_with_generic() {
                    let type_path = crate::path::type_path_(vct_reflect_path);

                    let generics = TypePathParser::reduce_generics(
                        generics,
                        |TypeParam { ident, .. }| {
                            StringExpr::Borrowed(quote! {
                                <#ident as #type_path>::type_path()
                            })
                        },
                        vct_reflect_path,
                    );

                    StringExpr::from_iter([
                        module_path,
                        StringExpr::from_str("::"),
                        ident,
                        StringExpr::from_str("<"),
                        generics,
                        StringExpr::from_str(">"),
                    ], vct_reflect_path)
                } else {
                    StringExpr::from_iter([module_path, StringExpr::from_str("::"), ident], vct_reflect_path)
                }
            }
        }
    }

    pub fn type_path_into_owned(&self, vct_reflect_path: &Path) -> proc_macro2::TokenStream {
        self.type_path(vct_reflect_path).into_owned(vct_reflect_path)
    }

    pub fn type_name_into_owned(&self, vct_reflect_path: &Path) -> proc_macro2::TokenStream {
        self.type_name(vct_reflect_path).into_owned(vct_reflect_path)
    }
}

