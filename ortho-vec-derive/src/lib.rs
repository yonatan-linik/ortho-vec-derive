#![doc = include_str!("../README.md")]
#![warn(
    clippy::all,
    clippy::missing_const_for_fn,
    clippy::trivially_copy_pass_by_ref,
    clippy::map_unwrap_or,
    clippy::explicit_into_iter_loop,
    clippy::unused_self,
    clippy::needless_pass_by_value
)]

pub mod prelude;

pub use ortho_vec_derive_impl::*;
pub use ortho_vec_derive_macro::*;

#[cfg(doctest)]
mod test_readme {
    macro_rules! external_doc_test {
        ($x:expr) => {
            #[doc = $x]
            extern "C" {}
        };
    }

    external_doc_test!(include_str!("../../README.md"));
}

#[cfg(test)]
mod tests {
    use ortho_vec_derive_macro::OrthoVec;

    #[derive(OrthoVec)]
    struct WeirdStruct<'a, T: Send>
    where T: std::fmt::Debug {
        a: i32,
        b: &'a f32,
        c: T,
    }

    #[test]
    fn test_use_weird_struct() {
        let v_ws = vec![WeirdStruct {a: 3, b: &4.2, c: "wow"}, WeirdStruct {a: 6, b: &24.2, c: "hello"}].into_ortho();
        for x in v_ws {
            println!("a is {:?}", x.a);
            println!("b is {:?}", x.b);
            println!("c is {:?}", x.c);
        }
    }

}