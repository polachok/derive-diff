extern crate boolinator;
extern crate chrono;

use boolinator::Boolinator;
use chrono::DateTime;
use std::borrow::Borrow;
use std::cell::{Cell,RefCell};
use std::fmt::Debug;
use std::ops::Deref;
use std::path::{Path,PathBuf};
use std::rc::Rc;
use std::sync::Arc;

pub trait Diff {
    type Value: Debug + PartialEq + ?Sized;

    fn diff<'a>(&'a self, other: &'a Self::Value) -> Option<Vec<Difference<'a>>>;
}

/// Field that differs
#[derive(Debug)]
pub struct Difference<'a> {
    pub field: String,
    pub left: &'a Debug,
    pub right: &'a Debug,
}

macro_rules! impl_for_prim {
    ($t: ty) => {
        impl Diff for $t {
            type Value = $t;

            fn diff<'a>(&'a self, other: &'a Self::Value) -> Option<Vec<Difference<'a>>> {
                self.ne(other).as_some_from(||
                    vec![Difference {
                        field: String::new(),
                        left: self,
                        right: other,
                    }]
                )
            }
        }       
    };
}

macro_rules! impl_for_prim_ref {
    ($t: ty) => {
        impl<'b> Diff for &'b $t {
            type Value = &'b $t;
            fn diff<'a>(&'a self, other: &'a Self::Value) -> Option<Vec<Difference<'a>>> {
                self.ne(other).as_some_from(||
                    vec![Difference {
                        field: String::new(),
                        left: self,
                        right: other,
                    }]
                )
            }
        }
    };
}

macro_rules! impl_for_wrapper {
    ($t: tt) => {
        impl<T> Diff for $t<T> where $t<T>: Borrow<T>, T: Debug + Diff<Value=T> + PartialEq {
            type Value = T;
            fn diff<'a>(&'a self, other: &'a Self::Value) -> Option<Vec<Difference<'a>>> {
                self.borrow().ne(other.borrow()).as_some_from(||
                    vec![Difference {
                        field: String::new(),
                        left: self.borrow(),
                        right: other.borrow(),
                    }]
                )
            }
        }
    };
}

/*
impl<B Diff for B where B: Borrow<T>, T: Debug + Diff<Value=T> + PartialEq {
    type Value = T;
    fn diff<'a>(&'a self, other: &'a Self::Value) -> Option<Vec<Difference<'a>>> {
        if self.borrow() != other.borrow() {
            return Some(vec![Difference {
                field: String::new(),
                left: self.borrow(),
                right: other.borrow(),
            }]);
        }
        None
    }
}
*/

impl<T> Diff for Cell<T> where T: Copy + Debug + Diff<Value=T> + PartialEq {
    type Value = T;
    fn diff<'a>(&'a self, other: &'a Self::Value) -> Option<Vec<Difference<'a>>> {
        self.get().ne(other).as_some_from(||
            vec![Difference {
                field: String::new(),
                left: self,
                right: other,
            }]
        )
    }
}

impl<T: chrono::TimeZone> Diff for DateTime<T> {
    type Value = DateTime<T>;
    fn diff<'a>(&'a self, other: &'a Self::Value) -> Option<Vec<Difference<'a>>> {
        self.ne(other).as_some_from(||
            vec![Difference {
                field: String::new(),
                left: self,
                right: other,
            }]
        )
    }
}

impl<T> Diff for Option<T> where T: Debug + PartialEq + Diff<Value=T> {
    type Value = Option<T>;
    fn diff<'a>(&'a self, other: &'a Self::Value) -> Option<Vec<Difference<'a>>> {
        match (self, other) {
            (&Some(ref left), &Some(ref right)) => {
                left.diff(right)
            }
            (&None, &Some(_)) => {
                Some(vec![Difference { field: "none".into(), left: self, right: other }])
            },
            (&Some(_), &None) => {
                Some(vec![Difference { field: "some".into(), left: self, right: other }])
            },
            (&None, &None) => None,
        }
    }
}

impl<T> Diff for RefCell<T> where T: Debug + Diff<Value=T> + PartialEq {
    type Value = T;
    fn diff<'a>(&'a self, other: &'a Self::Value) -> Option<Vec<Difference<'a>>> {
        self.borrow().deref().ne(other).as_some_from(||
            vec![Difference {
                field: String::new(),
                left: self,
                right: other,
            }]
        )
    }
}

impl<T> Diff for [T] where T: Debug + Diff<Value=T> + PartialEq {
    type Value = [T];
    fn diff<'a>(&'a self, other: &'a Self::Value) -> Option<Vec<Difference<'a>>> {
        self.ne(other).and_option_from(|| {
            let diffs: Vec<_> = self
                .into_iter()
                .zip(other.iter())
                .enumerate()
                .filter_map(|(i, (left, right))| {
                    left
                        .diff(right)
                        .map(|inner_diffs|
                            inner_diffs.into_iter().map(move |diff| 
                                Difference {
                                    field: {
                                      // Use push for speed
                                      // guesstimate string size
                                      let mut s=String::with_capacity(3+10+diff.field.len());
                                      s.push_str("[");
                                      s.push_str(&i.to_string());
                                      if !diff.field.is_empty() {
                                        s.push_str("].");
                                        s.push_str(&diff.field);
                                      } else {
                                        s.push_str("]");
                                      }
                                      s
                                    },
                                    left: diff.left,
                                    right: diff.right,
                                }
                            )
                        )
                })
                .flat_map(|x|x)
                .collect();
            (!diffs.is_empty()).as_some(diffs)
        })
    }
}

impl_for_wrapper!(Arc);
impl_for_wrapper!(Box);
impl_for_wrapper!(Rc);
impl_for_prim!(bool);
impl_for_prim!(isize);
impl_for_prim!(i8);
impl_for_prim!(i16);
impl_for_prim!(i32);
impl_for_prim!(i64);
impl_for_prim!(usize);
impl_for_prim!(u8);
impl_for_prim!(u16);
impl_for_prim!(u32);
impl_for_prim!(u64);
impl_for_prim!(f32);
impl_for_prim!(f64);
impl_for_prim!(char);
impl_for_prim!(String);
impl_for_prim!(chrono::NaiveDateTime);
impl_for_prim!(PathBuf);
impl_for_prim_ref!(Path);
impl_for_prim_ref!(str);

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test_diff_wrappers {
        ( $a: expr, $b: expr, $f:ident $(, $w:tt )* ) => {
            $(
                $f($w::new($a.clone()), $b.clone());
                $f($a.clone(), $w::new($b.clone()));
                $f($w::new($a.clone()), $w::new($b.clone()));
            )*

        }
    }

    impl<'a> PartialEq for Difference<'a> {
        fn eq(&self, other: &Self) -> bool {
            self.field.eq(&other.field) && format!("{:?}", self.left) == format!("{:?}", other.left) && format!("{:?}", self.right) == format!("{:?}", other.right)
        }
    }

    fn test_diff_simple<T, T1, T2>(i_a: T1, i_b: T2) 
        where T: Debug + Diff<Value=T> + PartialEq,
              T1: Diff<Value=T> + Borrow<T>,
              T2: Diff<Value=T> + Borrow<T> {
        let r = i_a.diff(i_b.borrow());
        if i_a.borrow() != i_b.borrow() {
            assert_eq!(r, Some(vec![Difference { field: String::new(), left: i_a.borrow(), right: i_b.borrow() }]));
        } else {
            assert_eq!(r, None);
        }
    }

    fn test_diff_array<T>(i_a: &[T], i_b: &[T]) 
        where T: Debug + Diff<Value=T> + PartialEq {
        let r = i_a.diff(i_b);
        let truth: Vec<_> = i_a.iter().zip(i_b).enumerate().filter_map(|(i, (a,b))| {
            if a != b {
                Some(vec![Difference { field: format!("[{}]", i), left: a, right: b}])
            } else {
                None
            }
        })
        .flat_map(|x|x)
        .collect();
        assert_eq!(r, (i_a!=i_b).as_some(truth));
    }

    fn test_diff<T>(i_a: T, i_b: T) 
        where T: Clone + Debug + PartialEq + Diff<Value=T> {

        test_diff_wrappers!(i_a, i_b, test_diff_simple, Arc, Box, Rc);
        // FIXME: Test Cell and RefCell
        //test_diff_wrappers!(i_a, i_b, test_diff_refcell, RefCell);
        //test_diff_wrappers!(i_a, i_b, test_diff_cell, Cell);
        test_diff_array(&[i_a.clone()], &[i_b.clone()]);
        test_diff_simple(i_a, i_b);
    }

    #[test]
    fn test_bool() {
        test_diff(true, true);
        test_diff(true, false);
    }

    
    #[test]
    fn test_isiz() {
        test_diff(0isize, 0isize);
        test_diff(0isize, 1isize);
    }

    #[test]
    fn test_i8() {
        test_diff(0i8, 0i8);
        test_diff(0i8, 1i8);
    }

    #[test]
    fn test_i16() {
        test_diff(0i16, 0i16);
        test_diff(0i16, 1i16);
    }

    #[test]
    fn test_i32() {
        test_diff(0i32, 0i32);
        test_diff(0i32, 1i32);
    }

    #[test]
    fn test_i64() {
        test_diff(0i64, 0i64);
        test_diff(0i64, 1i64);
    }
 
    #[test]
    fn test_usize() {
        test_diff(0usize, 0usize);
        test_diff(0usize, 1usize);
    }
  
    #[test]
    fn test_u8() {
        test_diff(0u8, 0u8);
        test_diff(0u8, 1u8);
    }

    #[test]
    fn test_u16() {
        test_diff(0u16, 0u16);
        test_diff(0u16, 1u16);
    }

    #[test]
    fn test_u32() {
        test_diff(0u32, 0u32);
        test_diff(0u32, 1u32);
    }

    #[test]
    fn test_u64() {
        test_diff(0u64, 0u64);
        test_diff(0u64, 1u64);
    }

    #[test]
    fn test_f32() {
        test_diff(0.0f32, 0.0f32);
        test_diff(0.0f32, 1.0f32);
    }

    #[test]
    fn test_f64() {
        test_diff(0.0f64, 0.0f64);
        test_diff(0.0f64, 1.0f64);
    }

    #[test]
    fn test_char() {
        test_diff('a', 'a');
        test_diff('a', 'b');
    }

    #[test]
    fn test_str() {
        test_diff("a", "a");
        test_diff("a", "b");
    }

    #[test]
    fn test_string() {
        test_diff(String::from("a"), String::from("a"));
        test_diff(String::from("a"), String::from("b"));
    }

    #[test]
    fn test_chrono() {
        test_diff(chrono::NaiveDateTime::from_timestamp(0,0), chrono::NaiveDateTime::from_timestamp(0,0));
        test_diff(chrono::NaiveDateTime::from_timestamp(0,0), chrono::NaiveDateTime::from_timestamp(1,0));
    }

    #[test]
    fn test_path() {
        test_diff(Path::new("/a"), Path::new("/a"));
        test_diff(Path::new("/a"), Path::new("/b"));
    }

    #[test]
    fn test_pathbuf() {
        test_diff(PathBuf::from("/a"), PathBuf::from("/a"));
        test_diff(PathBuf::from("/a"), PathBuf::from("/b"));
    }
}
