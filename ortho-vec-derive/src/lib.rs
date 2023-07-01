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
    where
        T: std::fmt::Debug,
    {
        a: i32,
        b: &'a f32,
        c: T,
    }

    #[test]
    fn test_use_weird_struct() {
        let mut v_ws = vec![
            WeirdStruct {
                a: 3,
                b: &4.2,
                c: "wow",
            },
            WeirdStruct {
                a: 6,
                b: &24.2,
                c: "hello",
            },
        ]
        .into_ortho();
        assert_eq!(v_ws.len(), 2);

        let last_element = v_ws.pop().expect("There must be a last element");
        assert_eq!(v_ws.len(), 1);

        assert_eq!(last_element.a, 6);
        assert!((*last_element.b - 24.2).abs() < f32::EPSILON);
        assert_eq!(last_element.c, "hello");

        v_ws.push(WeirdStruct {
            a: 7,
            b: &8.123,
            c: "push",
        });
        assert_eq!(v_ws.len(), 2);

        v_ws.reverse();

        for x in v_ws.iter() {
            println!("a is {:?}", x.a);
            println!("b is {:?}", x.b);
            println!("c is {:?}", x.c);
        }

        assert_eq!(v_ws.len(), 2);
        v_ws.clear();
        assert_eq!(v_ws.len(), 0);
    }
}
