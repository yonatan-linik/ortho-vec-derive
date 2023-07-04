/// Helps you create ortho-`Vec`s to better utilize the cache of the CPU
///
/// # Examples
///
/// ```rust
/// use ortho_vec_derive::prelude::*;
///
/// #[derive(OrthoVec)]
/// struct ExampleStruct {
///     a: i32,
///     b: f32,
/// }
///
/// # fn main() {
/// let ortho_vec = vec![ExampleStruct {a: 7, b: 9.3},
///                      ExampleStruct {a: 3, b: 3.14}]
///                 .into_ortho();
///
/// for es in ortho_vec.iter() {
///     assert_eq!(es.a % 2, 1);
/// }
/// # }
/// ```
pub use crate::OrthoVec;
