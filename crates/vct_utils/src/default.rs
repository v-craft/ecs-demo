use core::default::Default;

/// 此函数用于简化 `default()` 的使用
///
/// ```
/// use vct_utils::default;
///
/// #[derive(Default)]
/// struct Foo {
///   a: usize,
///   b: usize,
///   c: usize,
/// }
///
/// // 以前需要调用 `Default::default()` 或 `Foo::default()`
/// let foo = Foo {
///   a: 10,
///   ..Default::default()
/// };
/// # let foo = Foo {
/// #   a: 10,
/// #   ..Foo::default()
/// # };
///
/// // 现在可以简化
/// let foo = Foo {
///   a: 10,
///   ..default()
/// };
/// ```
#[inline(always)]
pub fn default<T: Default>() -> T {
    Default::default()
}
