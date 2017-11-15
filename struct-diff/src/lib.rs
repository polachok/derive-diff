extern crate chrono;

use chrono::DateTime;
use std::fmt::Debug;

pub trait Diff: Debug + PartialEq {
    fn diff<'a>(&'a self, other: &'a Self) -> Option<Vec<Difference<'a>>>;
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
            fn diff<'a>(&'a self, other: &'a Self) -> Option<Vec<Difference<'a>>> {
                if self != other {
                    return Some(vec![Difference {
                        field: String::new(),
                        left: self,
                        right: other,
                    }]);
                }
                None
            }
        }       
    };
}



impl<'b> Diff for &'b str {
    fn diff<'a>(&'a self, other: &'a Self) -> Option<Vec<Difference<'a>>> {
        if self != other {
            return Some(vec![Difference {
                field: String::new(),
                left: self,
                right: other,
            }])
        }
        None
    }
}

impl<T: chrono::TimeZone> Diff for DateTime<T> {
    fn diff<'a>(&'a self, other: &'a Self) -> Option<Vec<Difference<'a>>> {
        if self != other {
            return Some(vec![Difference {
                field: String::new(),
                left: self,
                right: other,
            }]);
        }
        None
    }
}


impl<T> Diff for Option<T> where T: std::fmt::Debug + PartialEq + Diff {
    fn diff<'a>(&'a self, other: &'a Self) -> Option<Vec<Difference<'a>>> {
        match (self, other) {
            (&Some(ref left), &Some(ref right)) => {
                left.diff(right)
            }
            (&None, &Some(_)) => {
                Some(vec![Difference { field: format!("none"), left: self, right: other }])
            },
            (&Some(_), &None) => {
                Some(vec![Difference { field: format!("some"), left: self, right: other }])
            },
            (&None, &None) => None,
        }
    }
}


impl<T> Diff for [T] where T: Diff {
    fn diff<'a>(&'a self, other: &'a Self) -> Option<Vec<Difference<'a>>> {
        if self != other {
            let mut diffs = Vec::new();
            for (i, (left, right)) in self.iter().zip(other.iter()).enumerate() {
                if let Some(inner_diffs) = left.diff(right) {
                    for diff in inner_diffs {
                        let mut path = format!("[{}]", i);
                        if !diff.field.is_empty() {
                            path.push_str(".");
                        }
                        path.push_str(&diff.field);
                        diffs.push(Difference {
                            field: path,
                            left,
                            right,
                        });
                    }
                }
            }
            if diffs.len() > 0 {
                return Some(diffs)
            }
        }
        None
    }
}

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