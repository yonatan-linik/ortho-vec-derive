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

    struct TestContext<'a> {
        v_ws: OrthoVecWeirdStruct<'a, &'a str>,
    }

    impl<'a> TestContext<'a> {
        fn setup() -> Self {
            Self {
                v_ws: vec![
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
                .into_ortho(),
            }
        }
    }

    #[test]
    fn test_use_weird_struct() {
        let mut ctx = TestContext::setup();
        assert_eq!(ctx.v_ws.len(), 2);

        let last_element = ctx.v_ws.pop().expect("There must be a last element");
        assert_eq!(ctx.v_ws.len(), 1);

        assert_eq!(last_element.a, 6);
        assert!((*last_element.b - 24.2).abs() < f32::EPSILON);
        assert_eq!(last_element.c, "hello");

        ctx.v_ws.push(WeirdStruct {
            a: 7,
            b: &8.123,
            c: "push",
        });
        assert_eq!(ctx.v_ws.len(), 2);

        ctx.v_ws.reverse();

        for x in ctx.v_ws.iter() {
            println!("a is {:?}", x.a);
            println!("b is {:?}", x.b);
            println!("c is {:?}", x.c);
        }

        assert_eq!(ctx.v_ws.len(), 2);
        ctx.v_ws.clear();
        assert_eq!(ctx.v_ws.len(), 0);
    }

    #[test]
    fn test_insert_remove() {
        let mut ctx = TestContext::setup();

        ctx.v_ws.insert(
            1,
            WeirdStruct {
                a: 7,
                b: &3.0,
                c: "in the middle",
            },
        );
        assert_eq!(ctx.v_ws.len(), 3);
        assert_eq!(ctx.v_ws.pop().unwrap().a, 6);
        assert_eq!(ctx.v_ws.len(), 2);
        assert_eq!(ctx.v_ws.remove(1).a, 7);
        assert_eq!(ctx.v_ws.len(), 1);
        assert_eq!(ctx.v_ws.remove(0).a, 3);
        assert_eq!(ctx.v_ws.len(), 0);
    }

    #[test]
    fn test_swap_remove() {
        let mut ctx = TestContext::setup();

        ctx.v_ws.push(WeirdStruct {
            a: 7,
            b: &3.0,
            c: "Going to the start",
        });

        assert_eq!(ctx.v_ws.len(), 3);
        ctx.v_ws.swap_remove(0);
        assert_eq!(ctx.v_ws.len(), 2);

        assert_eq!(ctx.v_ws.remove(0).c, "Going to the start");
    }
}
