/// Trait used to generate [`TypeTrait`] for trait reflection.
pub trait FromType<T> {
    fn from_type() -> Self;
}
