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
        /*
        #[derive(Debug, PartialEq, Diff)]
        struct A {
            v: [u8; 3]
        }
        */
        let a = [5, 6, 8u8];
        let b = [5, 7, 8];
        let aa = a.as_ref();
        let bb = b.as_ref();
        let diff = aa.diff(&bb);
        assert_eq!(diff, Some(vec![Difference { field: "ab".to_owned(), left: &1, right: &2 }]));
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
}
