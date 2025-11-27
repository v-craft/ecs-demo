use crate::{
    PartialReflect, Reflect,
    access::{PathAccessError, parse::PathParser},
    ops::{Array, Enum, List, Struct, Tuple, TupleStruct},
};

/// Provide timely calculated path access function.
///
/// Each call requires a re-parsing of the path string.
/// The good news is that path parsing is fast (O(N))
/// and does **not** require allocating heap space.
///
/// You should **not** manually implement this trait,
/// as it has already been implemented for all necessary types:
///
/// - Types that implement [`PartialReflect`]
/// - [`dyn Reflect`]
/// - [`dyn PartialReflect`]
/// - [`dyn Struct`]
/// - [`dyn TupleStruct`]
/// - [`dyn Tuple`]
/// - [`dyn List`]
/// - [`dyn Array`]
/// - [`dyn Enum`]
pub trait ReflectPathAccess {
    /// Returns a reference to the value specified by `path`.
    ///
    /// See [`ReflectPathAccess`]
    fn access<'a, 'b>(
        &'a self,
        path: &'b str,
    ) -> Result<&'a dyn PartialReflect, PathAccessError<'b>>;

    /// Returns a mutable reference to the value specified by `path`.
    ///
    /// See [`ReflectPathAccess`]
    fn access_mut<'a, 'b>(
        &'a mut self,
        path: &'b str,
    ) -> Result<&'a mut dyn PartialReflect, PathAccessError<'b>>;

    /// Returns a typed reference to the value specified by `path`.
    ///
    /// See [`ReflectPathAccess`]
    fn access_as<'a, 'b, T: Reflect>(&'a self, path: &'b str)
    -> Result<&'a T, PathAccessError<'b>>;

    /// Returns a mutable typed reference to the value specified by `path`.
    ///
    /// See [`ReflectPathAccess`]
    fn access_mut_as<'a, 'b, T: Reflect>(
        &'a mut self,
        path: &'b str,
    ) -> Result<&'a mut T, PathAccessError<'b>>;
}

impl ReflectPathAccess for dyn PartialReflect {
    #[inline(never)]
    fn access<'a, 'b>(
        &'a self,
        path: &'b str,
    ) -> Result<&'a dyn PartialReflect, PathAccessError<'b>> {
        let mut it: &dyn PartialReflect = self;
        let parser = PathParser::new(path);
        for (res, offset) in parser {
            match res {
                Ok(accessor) => match accessor.access(it, Some(offset)) {
                    Ok(val) => it = val,
                    Err(err) => return Err(PathAccessError::InvalidAccess(err)),
                },
                Err(error) => {
                    return Err(PathAccessError::ParseError {
                        offset,
                        path,
                        error,
                    });
                }
            }
        }
        return Ok(it);
    }

    #[inline(never)]
    fn access_mut<'a, 'b>(
        &'a mut self,
        path: &'b str,
    ) -> Result<&'a mut dyn PartialReflect, PathAccessError<'b>> {
        let mut it: &mut dyn PartialReflect = self;
        let parser = PathParser::new(path);
        for (res, offset) in parser {
            match res {
                Ok(accessor) => match accessor.access_mut(it, Some(offset)) {
                    Ok(val) => it = val,
                    Err(err) => return Err(PathAccessError::InvalidAccess(err)),
                },
                Err(error) => {
                    return Err(PathAccessError::ParseError {
                        offset,
                        path,
                        error,
                    });
                }
            }
        }
        return Ok(it);
    }

    #[inline(never)]
    fn access_as<'a, 'b, T: Reflect>(
        &'a self,
        path: &'b str,
    ) -> Result<&'a T, PathAccessError<'b>> {
        // Not Inline `access`: Reduce compilation time.
        // Now `access` is compiled only once per impl, independent of T.
        let it = ReflectPathAccess::access(self, path)?;
        match it.try_downcast_ref::<T>() {
            Some(it) => Ok(it),
            None => Err(PathAccessError::InvalidDowncast),
        }
    }

    #[inline(never)]
    fn access_mut_as<'a, 'b, T: Reflect>(
        &'a mut self,
        path: &'b str,
    ) -> Result<&'a mut T, PathAccessError<'b>> {
        // Not Inline `access`: Reduce compilation time.
        // Now `access` is compiled only once per impl, independent of T.
        let it = ReflectPathAccess::access_mut(self, path)?;
        match it.try_downcast_mut::<T>() {
            Some(it) => Ok(it),
            None => Err(PathAccessError::InvalidDowncast),
        }
    }
}

impl<P: Sized + PartialReflect> ReflectPathAccess for P {
    #[inline(always)]
    fn access<'a, 'b>(
        &'a self,
        path: &'b str,
    ) -> Result<&'a dyn PartialReflect, PathAccessError<'b>> {
        // Significantly reduce compilation time
        <dyn PartialReflect as ReflectPathAccess>::access(self, path)
    }

    #[inline(always)]
    fn access_mut<'a, 'b>(
        &'a mut self,
        path: &'b str,
    ) -> Result<&'a mut dyn PartialReflect, PathAccessError<'b>> {
        // Significantly reduce compilation time
        <dyn PartialReflect as ReflectPathAccess>::access_mut(self, path)
    }

    #[inline(always)]
    fn access_as<'a, 'b, T: Reflect>(
        &'a self,
        path: &'b str,
    ) -> Result<&'a T, PathAccessError<'b>> {
        // Significantly reduce compilation time
        <dyn PartialReflect as ReflectPathAccess>::access_as::<T>(self, path)
    }

    #[inline(always)]
    fn access_mut_as<'a, 'b, T: Reflect>(
        &'a mut self,
        path: &'b str,
    ) -> Result<&'a mut T, PathAccessError<'b>> {
        // Significantly reduce compilation time
        <dyn PartialReflect as ReflectPathAccess>::access_mut_as::<T>(self, path)
    }
}

macro_rules! impl_reflect_path_access {
    (dyn $name:ident) => {
        impl ReflectPathAccess for dyn $name {
            #[inline(always)]
            fn access<'a, 'b>(
                &'a self,
                path: &'b str,
            ) -> Result<&'a dyn PartialReflect, PathAccessError<'b>> {
                // Significantly reduce compilation time
                <dyn PartialReflect as ReflectPathAccess>::access(self, path)
            }

            #[inline(always)]
            fn access_mut<'a, 'b>(
                &'a mut self,
                path: &'b str,
            ) -> Result<&'a mut dyn PartialReflect, PathAccessError<'b>> {
                // Significantly reduce compilation time
                <dyn PartialReflect as ReflectPathAccess>::access_mut(self, path)
            }

            #[inline(always)]
            fn access_as<'a, 'b, T: Reflect>(
                &'a self,
                path: &'b str,
            ) -> Result<&'a T, PathAccessError<'b>> {
                // Significantly reduce compilation time
                <dyn PartialReflect as ReflectPathAccess>::access_as::<T>(self, path)
            }

            #[inline(always)]
            fn access_mut_as<'a, 'b, T: Reflect>(
                &'a mut self,
                path: &'b str,
            ) -> Result<&'a mut T, PathAccessError<'b>> {
                // Significantly reduce compilation time
                <dyn PartialReflect as ReflectPathAccess>::access_mut_as::<T>(self, path)
            }
        }
    };
}

impl_reflect_path_access!(dyn Reflect);
impl_reflect_path_access!(dyn Struct);
impl_reflect_path_access!(dyn TupleStruct);
impl_reflect_path_access!(dyn Tuple);
impl_reflect_path_access!(dyn List);
impl_reflect_path_access!(dyn Array);
impl_reflect_path_access!(dyn Enum);
