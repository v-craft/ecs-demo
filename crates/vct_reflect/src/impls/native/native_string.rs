use alloc::string::String;
use vct_reflect_derive::impl_full_reflect;

impl_full_reflect!{
    #[reflect(opaque, clone, default, hash, partial_eq, debug, serde)]
    #[reflect(type_path = "alloc::string::String")]
    struct String;
}

