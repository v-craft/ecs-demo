
pub(crate) static REFLECT_ATTRIBUTE_NAME: &str = "reflect";
pub(crate) static TYPE_PATH_ATTRIBUTE_NAME: &str = "type_path";
pub(crate) static TYPE_NAME_ATTRIBUTE_NAME: &str = "type_name";

mod path;

mod utils;

#[cfg(feature = "reflect_docs")]
mod reflect_docs;
mod reflect_trait;

mod attributes;
mod type_path;

mod derive_data;






