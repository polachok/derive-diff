extern crate chrono;

use std::fmt;
use chrono::DateTime;

pub trait Diff where Self: PartialEq + std::fmt::Debug {
    fn diff(&self, other: &Self) -> Option<Vec<Difference>>;
}

#[derive(Debug)]
pub struct Difference {
    pub field: String,
}

macro_rules! impl_for_prim {
    ($t: ty) => {
        impl Diff for $t {
            fn diff(&self, other: &Self) -> Option<Vec<Difference>> {
                if self != other {
                    return Some(vec![Difference {
                        field: String::new(),
                    }]);
                }
                None
            }
        }       
    };
}

impl<T: chrono::TimeZone> Diff for DateTime<T> {
    fn diff(&self, other: &Self) -> Option<Vec<Difference>> {
        if self != other {
            return Some(vec![Difference {
                field: String::new(),
            }]);
        }
        None
    }
}

impl<T> Diff for Option<T> where T: std::fmt::Debug + PartialEq + Diff {
    fn diff(&self, other: &Self) -> Option<Vec<Difference>> {
        match (self, other) {
            (&Some(ref val), &Some(ref other)) => {
                val.diff(other)
            }
            (&None, &Some(_)) => {
                Some(vec![Difference { field: format!("none") }])
            },
            (&Some(_), &None) => {
                Some(vec![Difference { field: format!("some") }])
            },
            (&None, &None) => None,
        }
    }
}

impl<T> Diff for [T] where T: std::fmt::Debug + PartialEq + Diff {
    fn diff(&self, other: &Self) -> Option<Vec<Difference>> {
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
impl_for_prim!(str);
impl_for_prim!(String);
impl_for_prim!(chrono::NaiveDateTime);

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}