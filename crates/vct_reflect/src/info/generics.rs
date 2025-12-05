use alloc::{borrow::Cow, boxed::Box};
use core::ops::Deref;

use crate::{
    Reflect,
    info::{ConstParamData, Type, TypePath, type_struct::impl_type_fn},
};

/// Container for storing generic type parameter information.
#[derive(Clone, Debug)]
pub struct TypeParamInfo {
    ty: Type,
    name: Cow<'static, str>,
    default: Option<Type>,
}

impl TypeParamInfo {
    impl_type_fn!(ty);

    /// Creates a new [`TypeParamInfo`].
    #[inline]
    pub fn new<T: TypePath + ?Sized>(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            ty: Type::of::<T>(),
            name: name.into(),
            default: None,
        }
    }

    /// Returns the generic parameter name.
    #[inline]
    pub fn name(&self) -> &Cow<'static, str> {
        &self.name
    }

    /// Returns the default type, if any.
    #[inline]
    pub fn default(&self) -> Option<&Type> {
        self.default.as_ref()
    }

    /// Sets the default type.
    #[inline]
    pub fn with_default<T: TypePath + ?Sized>(mut self) -> Self {
        self.default = Some(Type::of::<T>());
        self
    }
}


/// Container for storing generic const parameter information.
#[derive(Clone, Debug)]
pub struct ConstParamInfo {
    ty: Type,
    name: Cow<'static, str>,
    default: Option<ConstParamData>,
}

impl ConstParamInfo {
    impl_type_fn!(ty);

    /// Creates a new [`ConstParamInfo`].
    #[inline]
    pub fn new<T: TypePath + Into<ConstParamData>>(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            ty: Type::of::<T>(),
            name: name.into(),
            default: None,
        }
    }

    /// Returns the generic parameter name.
    #[inline]
    pub fn name(&self) -> &Cow<'static, str> {
        &self.name
    }

    /// Returns the default const value, if any.
    #[inline]
    pub fn default(&self) -> Option<ConstParamData> {
        self.default
    }

    /// Sets the default const value.
    /// 
    /// # Panic
    /// - incorrect type
    pub fn with_default<T: Reflect + Into<ConstParamData>>(mut self, default: T) -> Self {
        #[cfg(debug_assertions)]
        if !self.is::<T>() {
            panic!("The default value type in the const generic parameter is incorrect.");
        }
        self.default = Some(default.into());
        self
    }
}

/// An enum representing a single generic parameter.
#[derive(Clone, Debug)]
pub enum GenericInfo {
    Type(TypeParamInfo),
    Const(ConstParamInfo),
}

impl From<TypeParamInfo> for GenericInfo {
    #[inline]
    fn from(value: TypeParamInfo) -> Self {
        Self::Type(value)
    }
}

impl From<ConstParamInfo> for GenericInfo {
    #[inline]
    fn from(value: ConstParamInfo) -> Self {
        Self::Const(value)
    }
}

impl GenericInfo {
    impl_type_fn!(self => match self {
        Self::Type(info) => info.ty(),
        Self::Const(info) => info.ty(),
    });

    /// Returns the parameter name.
    #[inline]
    pub fn name(&self) -> &Cow<'static, str> {
        match self {
            Self::Type(info) => info.name(),
            Self::Const(info) => info.name(),
        }
    }

    /// Returns `true` if this is a const parameter.
    #[inline]
    pub fn is_const(&self) -> bool {
        match self {
            Self::Type(_) => false,
            Self::Const(_) => true,
        }
    }
}

/// Container for storing a list of generic parameters.
#[derive(Clone, Default, Debug)]
pub struct Generics(Box<[GenericInfo]>);

impl Generics {
    /// Creates a new empty container.
    #[inline]
    pub fn new() -> Self {
        Self(Box::new([]))
    }

    /// Returns the `GenericInfo` for the given parameter name, if any.
    ///
    /// Complexity: O(N)
    #[inline]
    pub fn get(&self, name: &str) -> Option<&GenericInfo> {
        self.0.iter().find(|info| info.name() == name)
    }

    /// Appends a parameter.
    ///
    /// Complexity: O(N)
    #[inline]
    pub fn with(mut self, info: impl Into<GenericInfo>) -> Self {
        self.0 = IntoIterator::into_iter(self.0)
            .chain(core::iter::once(info.into()))
            .collect();
        self
    }
}

impl<T: Into<GenericInfo>> FromIterator<T> for Generics {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().map(Into::into).collect())
    }
}

impl Deref for Generics {
    type Target = [GenericInfo];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// impl `with_generics` and `generics`
macro_rules! impl_generic_fn {
    ($field:ident) => {
        $crate::info::generics::impl_generic_fn!(self => &self.$field);

        /// Replace its own generic information
        #[inline]
        pub fn with_generics(
            mut self,
            generics: $crate::info::Generics
        ) -> Self {
            self.$field = generics;
            self
        }
    };
    ($self:ident => $expr:expr) => {
        /// Get generics from self based on expressions
        #[inline]
        pub fn generics($self: &Self) -> &$crate::info::Generics {
            $expr
        }
    };
}

pub(crate) use impl_generic_fn;
