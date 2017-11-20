#[macro_use]
extern crate derive_diff;
extern crate struct_diff;

use struct_diff::{Diff, Difference};

#[derive(Debug, PartialEq, Diff)]
struct A {
    key: String,
}

#[cfg(test)]
mod tests {
    use struct_diff::{Diff, Difference};

    #[test]
    fn same() {
        #[derive(Debug, PartialEq, Diff)]
        struct A {
            key: String,
        }
        let a = A { key: "hello".into() };
        assert!(a.diff(&a).is_none());
    }

    #[test]
    fn not_same() {
        #[derive(Debug, PartialEq, Diff)]
        struct A {
            key: String,
        }
        let a = A { key: "hello".into() };
        let b = A { key: "world".into() };
        let diff = a.diff(&b).unwrap();
        assert_eq!(diff[0].field, "key".to_string());
    }

    #[test]
    fn recursive() {
        #[derive(Debug, PartialEq, Diff)]
        struct A {
            b: B
        }

        #[derive(Debug, PartialEq, Diff)]
        struct B {
            val: u32
        }

        let a = A { b: B { val: 5 }};
        let b = A { b: B { val: 6 }};
        let diff = a.diff(&b).unwrap();
        assert_eq!(diff[0].field, "b.val");
    }

    #[test]
    fn arrays() {

        #[derive(Debug, PartialEq, Diff)]
        struct A {
            v: [u8; 3]
        }

        let a = A { v: [1,2,4] };
        let b = A { v: [2,2,3] };

        let diff = &a.diff(&b).unwrap();
        assert_eq!(diff[0].field, "v.[0]");
        assert_eq!(format!("{:?}", diff[0].left).as_str(), "1");
        assert_eq!(format!("{:?}", diff[0].right).as_str(), "2");
    }

    #[test]
    fn arrays_2() {
        let a = [1,2,3];
        let b = [2,2,3];
        let diff = &a.diff(&b).unwrap()[0];
        assert_eq!(diff.field, "[0]");
        assert_eq!(format!("{:?}", diff.left).as_str(), "1");
        assert_eq!(format!("{:?}", diff.right).as_str(), "2");
    }

    #[test]
    fn vecs() {
        #[derive(Debug, PartialEq, Diff)]
        struct A {
            v: Vec<u8>,
        }
        let a = A { v: vec![5, 6, 8] };
        let b = A { v: vec![5, 7, 8] };
        assert_eq!(a.diff(&b).unwrap()[0].field, "v.[1]");
    }

    #[test]
    fn vecs2() {
        #[derive(Debug, PartialEq, Diff)]
        struct A {
            v: Vec<u8>,
        }
        let a = A { v: vec![5, 6, 8] };
        let b = A { v: vec![5, 7, 9] };
        let diff = a.diff(&b).unwrap();
        let first = &diff[0].field;
        let second = &diff[1].field;
        assert_eq!(first, "v.[1]");
        assert_eq!(second, "v.[2]");
        assert_eq!(diff.len(), 2);
    }


//    #[test]
//    fn vecs_len() {
//        #[derive(Debug, PartialEq, Diff)]
//        struct A {
//            v: Vec<u8>,
//        }
//        let a = A { v: vec![5, 7] };
//        let b = A { v: vec![5, 7, 8] };
//        assert_eq!(a.diff(&b).unwrap()[0].field, "v.{length}");
//    }


    #[test]
    fn enums_1() {
        #[derive(Debug, PartialEq, Diff)]
        enum A {
            A(u32),
            B(&'static str)
        }
        let a = A::A(5);
        let b = A::A(6);
        assert_eq!(a.diff(&b).unwrap()[0].field, "A.0");
    }

    #[test]
    fn enums_2() {
        #[derive(Debug, PartialEq, Diff)]
        enum A {
            A(u32),
            B(&'static str)
        }
        let a = A::A(5);
        let b = A::B("kek");
        assert_eq!(a.diff(&b).unwrap()[0].field, "self");
    }

    #[test]
    fn enums_22() {
        #[derive(Debug, PartialEq, Diff)]
        enum A {
            A(u32, u32),
            B(&'static str)
        }
        let a = A::A(5, 6);
        let b = A::B("kek");
        assert_eq!(a.diff(&b).unwrap()[0].field, "self");
    }

    #[test]
    fn enums_3() {
        #[derive(Debug, PartialEq, Diff)]
        struct B {
            a: u32,
            b: u32,
        }
        #[derive(Debug, PartialEq, Diff)]
        enum A {
            A(u32),
            B(B)
        }
        let a = A::B(B { a: 7, b: 6 });
        let b = A::B(B { a: 5, b: 6 });
        assert_eq!(a.diff(&b).unwrap()[0].field, "B.0.a");
    }

    #[test]
    fn enums_4() {
        #[derive(Debug, PartialEq, Diff)]
        pub struct A {
            a: String,
        }
        #[derive(Debug, PartialEq, Diff)]
        pub enum R {
            S { t_vec: Vec<A> },
            C { c_vec: Vec<R> },
        }
        let i = R::S { t_vec: vec![A { a: "test".into()} ]};
        let a = R::C { c_vec: vec![i] };
        let i1 = R::S { t_vec: vec![A { a: "hello".into()} ]};
        let b = R::C { c_vec: vec![i1] };
        let c = R::S { t_vec: vec![A { a: "test".into()} ]};
        assert_eq!(a.diff(&b).unwrap()[0].field, "C.c_vec.[0].S.t_vec.[0].a");
        assert_eq!(a.diff(&c).unwrap()[0].field, "self");
    }

    #[test]
    fn inner_structs() {
        #[derive(Debug, PartialEq, Diff)]
        pub struct A {
            a: B,
        }
        #[derive(Debug, PartialEq, Diff)]
        pub struct B {
            a: Vec<C>,
        }
        #[derive(Debug, PartialEq, Diff)]
        pub struct C {
            a: String,
        }

        let a = A { a: B { a: vec![C { a: "the end".to_owned() }]}};
        let b = A { a: B { a: vec![C { a: "20th century fox".to_owned() }]}};
        let diffs = a.diff(&b).unwrap();
        assert_eq!(diffs[0].field, "a.a.[0].a".to_owned());
        assert_eq!(format!("{:?}", diffs[0].left), format!("{:?}", "the end".to_owned()));
        assert_eq!(format!("{:?}", diffs[0].right), format!("{:?}", "20th century fox".to_owned()));
    }

    #[test]
    fn inner_vector_structs_and_enums() {
        #[derive(Debug, PartialEq, Diff)]
        pub struct A {
            a: B,
        }
        #[derive(Debug, PartialEq, Diff)]
        pub enum B {
            First(Vec<C>),
            Second(String),
        }
        #[derive(Debug, PartialEq, Diff)]
        pub struct C {
            a: D,
            b: String,
        }

        #[derive(Debug, PartialEq, Diff)]
        pub struct D {
            a: Vec<E>,
        }

        #[derive(Debug, PartialEq, Diff)]
        pub struct E {
            a: String,
        }


        let a = A { a: B::First(vec![C { a: D { a: vec![E{ a: "the end".to_owned() }]}, b: "c struct b field 1".to_owned() }])};
        let b = A { a: B::First(vec![C { a: D { a: vec![E{ a: "20th century fox".to_owned() }]}, b: "c struct b field 2".to_owned()}])};
        let diffs = a.diff(&b).unwrap();
        assert_eq!(diffs[0].field, "a.First.0.[0].a.a.[0].a".to_owned());
        assert_eq!(diffs[1].field, "a.First.0.[0].b".to_owned());
        assert_eq!(format!("{:?}", diffs[0].left), format!("{:?}", "the end".to_owned()));
        assert_eq!(format!("{:?}", diffs[0].right), format!("{:?}", "20th century fox".to_owned()));
        assert_eq!(format!("{:?}", diffs[1].left), format!("{:?}", "c struct b field 1".to_owned()));
        assert_eq!(format!("{:?}", diffs[1].right), format!("{:?}", "c struct b field 2".to_owned()));
    }
}
