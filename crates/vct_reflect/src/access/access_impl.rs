use core::fmt;

use alloc::{borrow::Cow, vec::Vec};

use crate::{
    PartialReflect, Reflect,
    access::{ParseError, parse::PathParser},
    info::{ReflectKind, VariantKind},
    ops::{ReflectMut, ReflectRef},
};

/// A **singular** element access within a path.
///
/// Provide for [`Struct`], [`TupleStruct`], [`Tuple`], [`Array`], [`List`], [`Enum`].
///
/// [`Map`], [`Set`] and `Opaque` are not supported.
///
/// [`Struct`]: crate::ops::Struct
/// [`TupleStruct`]: crate::ops::TupleStruct
/// [`Tuple`]: crate::ops::Tuple
/// [`Array`]: crate::ops::Array
/// [`List`]: crate::ops::List
/// [`Map`]: crate::ops::Map
/// [`Set`]: crate::ops::Set
/// [`Enum`]: crate::ops::Enum
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Accessor<'a> {
    /// A name-based field access on a struct or Enum's struct.
    ///
    /// Example:  the `id` of `.id`
    FieldName(Cow<'a, str>),
    /// A index-based field access on Tuple, Tuple struct or Enum's tuple.
    ///
    /// Example:  the `5` of `.5`
    TupleIndex(usize),
    /// An index-based access on List and Array.  
    ///
    /// Example: the `5` of `[5]`
    ListIndex(usize),
    /// A index-based field access on Struct or Enum's struct.
    ///
    /// Can only be used to access Struct(excluding Tuple struct).
    ///
    /// Example: the `5` of `"#5"`
    FieldIndex(usize),
}

/// The kind of [`AccessError`], along with some kind-specific information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessErrorKind {
    MissingField(ReflectKind),
    IncompatibleKinds {
        expected: ReflectKind,
        actual: ReflectKind,
    },
    IncompatibleVariantKinds {
        expected: VariantKind,
        actual: VariantKind,
    },
}

/// An error originating from an [`Access`] of an element within a type.
///
/// Use the `Display` impl of this type to get information on the error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessError<'a> {
    kind: AccessErrorKind,
    accessor: Accessor<'a>,
    offset: Option<usize>,
}

impl fmt::Display for Accessor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Accessor::FieldName(field) => write!(f, ".{field}"),
            Accessor::FieldIndex(index) => write!(f, "#{index}"),
            Accessor::TupleIndex(index) => write!(f, ".{index}"),
            Accessor::ListIndex(index) => write!(f, "[{index}]"),
        }
    }
}

macro_rules! invalid_kind {
    ($expected:path, $actual:expr) => {
        AccessErrorKind::IncompatibleKinds {
            expected: $expected,
            actual: $actual,
        }
    };
}

macro_rules! match_variant {
    ($name:ident, $kind:path => $expr:expr) => {
        match $name.variant_kind() {
            $kind => Ok($expr),
            actual => Err(AccessErrorKind::IncompatibleVariantKinds {
                expected: $kind,
                actual,
            }),
        }
    };
}

impl<'a> Accessor<'a> {
    /// Converts this into an "owned" value.
    #[inline]
    pub fn into_owned(self) -> Accessor<'static> {
        match self {
            Self::FieldName(value) => Accessor::FieldName(Cow::Owned(value.into_owned())),
            Self::FieldIndex(value) => Accessor::FieldIndex(value),
            Self::TupleIndex(value) => Accessor::TupleIndex(value),
            Self::ListIndex(value) => Accessor::ListIndex(value),
        }
    }

    /// Returns a reference to this [`Access`]'s inner value as a [`&dyn Display`](fmt::Display).
    fn display_value(&self) -> &dyn fmt::Display {
        match self {
            Self::FieldName(value) => value,
            Self::FieldIndex(value) | Self::TupleIndex(value) | Self::ListIndex(value) => value,
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            Self::FieldName(_) => "FieldName",
            Self::FieldIndex(_) => "FieldIndex",
            Self::TupleIndex(_) => "TupleIndex",
            Self::ListIndex(_) => "ListIndex",
        }
    }

    /// Dynamic Access Fields, If successful, return the reference of the field.
    pub fn access<'r>(
        &self,
        base: &'r dyn PartialReflect,
        offset: Option<usize>, // use for error info
    ) -> Result<&'r dyn PartialReflect, AccessError<'a>> {
        use ReflectRef::*;

        let res: Result<Option<&'r dyn PartialReflect>, AccessErrorKind> =
            match (self, base.reflect_ref()) {
                (Self::FieldName(field), Struct(struct_ref)) => {
                    Ok(struct_ref.field(field.as_ref()))
                }
                (Self::FieldName(field), Enum(enum_ref)) => {
                    match_variant!(enum_ref, VariantKind::Struct => enum_ref.field(field.as_ref()))
                }
                (Self::FieldName(_), actual) => {
                    Err(invalid_kind!(ReflectKind::Struct, actual.kind()))
                }
                (&Self::FieldIndex(index), Struct(struct_ref)) => Ok(struct_ref.field_at(index)),
                (&Self::FieldIndex(index), Enum(enum_ref)) => {
                    match_variant!(enum_ref, VariantKind::Struct => enum_ref.field_at(index))
                }
                (Self::FieldIndex(_), actual) => {
                    Err(invalid_kind!(ReflectKind::Struct, actual.kind()))
                }
                (&Self::TupleIndex(index), TupleStruct(tuple)) => Ok(tuple.field(index)),
                (&Self::TupleIndex(index), Tuple(tuple)) => Ok(tuple.field(index)),
                (&Self::TupleIndex(index), Enum(enum_ref)) => {
                    match_variant!(enum_ref, VariantKind::Tuple => enum_ref.field_at(index))
                }
                (Self::TupleIndex(_), actual) => {
                    Err(invalid_kind!(ReflectKind::Tuple, actual.kind()))
                }
                (&Self::ListIndex(index), List(list)) => Ok(list.get(index)),
                (&Self::ListIndex(index), Array(list)) => Ok(list.get(index)),
                (Self::ListIndex(_), actual) => {
                    Err(invalid_kind!(ReflectKind::List, actual.kind()))
                }
            };

        res.and_then(|opt| opt.ok_or(AccessErrorKind::MissingField(base.reflect_kind())))
            .map_err(|kind| AccessError {
                kind,
                accessor: self.clone(),
                offset,
            })
    }

    /// Dynamic Access Fields, If successful, return the mutable reference of the field.
    pub fn access_mut<'r>(
        &self,
        base: &'r mut dyn PartialReflect,
        offset: Option<usize>, // use for error info
    ) -> Result<&'r mut dyn PartialReflect, AccessError<'a>> {
        use ReflectMut::*;

        let base_kind = base.reflect_kind();

        let res: Result<Option<&'r mut dyn PartialReflect>, AccessErrorKind> = match (
            self,
            base.reflect_mut(),
        ) {
            (Self::FieldName(field), Struct(struct_mut)) => {
                Ok(struct_mut.field_mut(field.as_ref()))
            }
            (Self::FieldName(field), Enum(enum_mut)) => {
                match_variant!(enum_mut, VariantKind::Struct => enum_mut.field_mut(field.as_ref()))
            }
            (Self::FieldName(_), actual) => Err(invalid_kind!(ReflectKind::Struct, actual.kind())),
            (&Self::FieldIndex(index), Struct(struct_mut)) => Ok(struct_mut.field_at_mut(index)),
            (&Self::FieldIndex(index), Enum(enum_mut)) => {
                match_variant!(enum_mut, VariantKind::Struct => enum_mut.field_at_mut(index))
            }
            (Self::FieldIndex(_), actual) => Err(invalid_kind!(ReflectKind::Struct, actual.kind())),
            (&Self::TupleIndex(index), TupleStruct(tuple)) => Ok(tuple.field_mut(index)),
            (&Self::TupleIndex(index), Tuple(tuple)) => Ok(tuple.field_mut(index)),
            (&Self::TupleIndex(index), Enum(enum_mut)) => {
                match_variant!(enum_mut, VariantKind::Tuple => enum_mut.field_at_mut(index))
            }
            (Self::TupleIndex(_), actual) => Err(invalid_kind!(ReflectKind::Tuple, actual.kind())),
            (&Self::ListIndex(index), List(list)) => Ok(list.get_mut(index)),
            (&Self::ListIndex(index), Array(list)) => Ok(list.get_mut(index)),
            (Self::ListIndex(_), actual) => Err(invalid_kind!(ReflectKind::List, actual.kind())),
        };

        res.and_then(|opt| opt.ok_or(AccessErrorKind::MissingField(base_kind)))
            .map_err(|kind| AccessError {
                kind,
                accessor: self.clone(),
                offset,
            })
    }
}

impl<'a> AccessError<'a> {
    /// Returns the kind of [`AccessError`].
    #[inline]
    pub fn kind(&self) -> &AccessErrorKind {
        &self.kind
    }

    /// The returns the [`Access`] that this [`AccessError`] occurred in.
    #[inline]
    pub fn accessor(&self) -> &Accessor<'_> {
        &self.accessor
    }

    /// If the [`Access`] was created with a parser or an offset was manually provided,
    /// returns the offset of the [`Access`] in its path string.
    #[inline]
    pub fn offset(&self) -> Option<&usize> {
        self.offset.as_ref()
    }
}

impl<'a> fmt::Display for AccessError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let AccessError {
            kind,
            accessor,
            offset,
        } = self;

        write!(f, "Error accessing element with `{accessor}` accessor")?;
        if let Some(offset) = offset {
            write!(f, "(offset {offset})")?;
        }
        write!(f, ": ")?;

        match kind {
            AccessErrorKind::MissingField(type_accessed) => match accessor {
                Accessor::FieldName(_) => write!(
                    f,
                    "The {type_accessed} accessed doesn't have field name `{}`",
                    accessor.display_value()
                ),
                Accessor::FieldIndex(_) => write!(
                    f,
                    "The {type_accessed} accessed doesn't have field index `{}`",
                    accessor.display_value(),
                ),
                Accessor::TupleIndex(_) | Accessor::ListIndex(_) => write!(
                    f,
                    "The {type_accessed} accessed doesn't have index `{}`",
                    accessor.display_value()
                ),
            },
            AccessErrorKind::IncompatibleKinds { expected, actual } => write!(
                f,
                "Expected {} accessor to access a {expected}, found a {actual} instead.",
                accessor.kind()
            ),
            AccessErrorKind::IncompatibleVariantKinds { expected, actual } => write!(
                f,
                "Expected variant {} accessor to access a {expected} variant, found a {actual} variant instead.",
                accessor.kind()
            ),
        }
    }
}

impl core::error::Error for AccessError<'_> {}

/// An error returned from a failed path string query.
#[derive(Debug, PartialEq, Eq)]
pub enum PathAccessError<'a> {
    /// An error caused by trying to access a path that's not able to be accessed,
    /// see [`AccessError`] for details.
    InvalidAccess(AccessError<'a>),

    /// An error that occurs when a type cannot downcast to a given type.
    InvalidDowncast,

    /// An error caused by an invalid path string that couldn't be parsed.
    ParseError {
        /// Position in `path`.
        offset: usize,
        /// The path that the error occurred in.
        path: &'a str,
        /// The underlying error.
        error: ParseError<'a>,
    },
}

impl fmt::Display for PathAccessError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAccess(err) => fmt::Display::fmt(err, f),
            Self::InvalidDowncast => {
                f.write_str("Can't downcast result of access to the given type")
            }
            Self::ParseError {
                offset,
                path,
                error,
            } => {
                write!(
                    f,
                    "Encountered an error at offset {offset} while parsing `{path}`: {error}"
                )
            }
        }
    }
}

impl core::error::Error for PathAccessError<'_> {}

// impl<'a> From<AccessError<'a>> for PathAccessError<'a> {
//     #[inline]
//     fn from(value: AccessError<'a>) -> Self {
//         Self::InvalidAccess(value)
//     }
// }

/// An [`Access`] combined with an `offset` for more helpful error reporting.
///
/// Offset is only used to display error messages, unrelated to access.
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct OffsetAccessor {
    pub accessor: Accessor<'static>,
    pub offset: Option<usize>,
}

impl From<Accessor<'static>> for OffsetAccessor {
    #[inline]
    fn from(accessor: Accessor<'static>) -> Self {
        Self {
            accessor,
            offset: None,
        }
    }
}

impl OffsetAccessor {
    #[inline]
    pub fn access<'r>(
        &self,
        base: &'r dyn PartialReflect,
    ) -> Result<&'r dyn PartialReflect, AccessError<'static>> {
        self.accessor.access(base, self.offset)
    }

    #[inline]
    pub fn access_mut<'r>(
        &self,
        base: &'r mut dyn PartialReflect,
    ) -> Result<&'r mut dyn PartialReflect, AccessError<'static>> {
        self.accessor.access_mut(base, self.offset)
    }
}

/// Reusable path accessor, wrapper of [`Vec<OffsetAccessor>`] .
///
/// [`OffsetAccessor`] and [`Accessor`] only allow access to a single level,
/// while this type allows for complete path queries.
///
/// Unlike [`ReflectPathAccess`], this container parses the path string only once during initialization.
/// However, for non-static strings, it requires copying for storage.
///
/// [`ReflectPathAccess`]: crate::access::ReflectPathAccess
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathAccessor(Vec<OffsetAccessor>);

impl From<Vec<OffsetAccessor>> for PathAccessor {
    #[inline]
    fn from(value: Vec<OffsetAccessor>) -> Self {
        Self(value)
    }
}

impl PathAccessor {
    /// Parse the path string and create an [`PathAccessor`].
    /// If the path is incorrect, it will return [`ParseError`].
    ///
    /// This function will create a [`String`] for each path segment.
    /// For '&'static str', please consider using ['parse_static'] for better performance.
    ///
    /// [`String`]: alloc::string::String
    /// ['parse_static']: crate::access::PathAccessor::parse_static
    pub fn parse(path: &str) -> Result<Self, ParseError<'_>> {
        let mut vc: Vec<OffsetAccessor> = Vec::new();
        for (res, offset) in PathParser::new(path) {
            let accessor = res?;
            vc.push(OffsetAccessor {
                accessor: accessor.into_owned(),
                offset: Some(offset),
            });
        }
        Ok(Self(vc))
    }

    /// Parse the path string and create an [`PathAccessor`].
    /// If the path is incorrect, it will return [`ParseError`].
    ///
    /// Can only be used for '&'static str', internal storage string references,
    /// no need to create additional [`String`].
    ///
    /// [`String`]: alloc::string::String
    pub fn parse_static(path: &'static str) -> Result<Self, ParseError<'static>> {
        let mut vc: Vec<OffsetAccessor> = Vec::new();
        for (res, offset) in PathParser::new(path) {
            vc.push(OffsetAccessor {
                accessor: res?,
                offset: Some(offset),
            });
        }
        Ok(Self(vc))
    }

    /// Return the length of the internal [`Vec`],
    /// which is the number of [`OffsetAccessor`].
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// The parse function directly uses [`Vec::push`] to add elements,
    /// which may result in redundant memory.
    ///
    /// An [`OffsetAccessor`] is 40 bytes. The std [`Vec`] expands in the pattern of `0->4->8->16`.
    /// Since path queries typically don't exceed a length of 4, these overheads are acceptable.
    ///
    /// But if needed, you can try using this function to
    /// shrinks the capacity as much as possible.
    ///
    /// See: [`Vec::shrink_to_fit`]
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit();
    }

    /// Returns a reference to the value specified by `path`.
    ///
    /// The accessor itself will not change and can be reused.
    pub fn access<'r>(
        &self,
        base: &'r dyn PartialReflect,
    ) -> Result<&'r dyn PartialReflect, PathAccessError<'static>> {
        let mut it = base;
        for accessor in &self.0 {
            it = match accessor.access(it) {
                Ok(val) => val,
                Err(err) => return Err(PathAccessError::InvalidAccess(err)),
            };
        }
        Ok(it)
    }

    /// Returns a mutable reference to the value specified by `path`.
    ///
    /// The accessor itself will not change and can be reused.
    pub fn access_mut<'r>(
        &self,
        base: &'r mut dyn PartialReflect,
    ) -> Result<&'r mut dyn PartialReflect, PathAccessError<'static>> {
        let mut it = base;
        for accessor in &self.0 {
            it = match accessor.access_mut(it) {
                Ok(val) => val,
                Err(err) => return Err(PathAccessError::InvalidAccess(err)),
            };
        }
        Ok(it)
    }

    /// Returns a typed reference to the value specified by `path`.
    ///
    /// The accessor itself will not change and can be reused.
    pub fn access_as<'r, T: Reflect>(
        &self,
        base: &'r dyn PartialReflect,
    ) -> Result<&'r T, PathAccessError<'static>> {
        let res = self.access(base)?;
        match res.try_downcast_ref::<T>() {
            Some(val) => Ok(val),
            None => Err(PathAccessError::InvalidDowncast),
        }
    }

    /// Returns a mutable typed reference to the value specified by `path`.
    ///
    /// The accessor itself will not change and can be reused.
    pub fn access_mut_as<'r, T: Reflect>(
        &self,
        base: &'r mut dyn PartialReflect,
    ) -> Result<&'r mut T, PathAccessError<'static>> {
        let res = self.access_mut(base)?;
        match res.try_downcast_mut::<T>() {
            Some(val) => Ok(val),
            None => Err(PathAccessError::InvalidDowncast),
        }
    }
}

impl fmt::Display for PathAccessor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for it in &self.0 {
            fmt::Display::fmt(&it.accessor, f)?;
        }
        Ok(())
    }
}
