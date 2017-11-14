extern crate slog;
extern crate struct_diff;

use struct_diff::{Diff, Difference};
use slog::{KV, Result, Record, Serializer};

pub struct Differences<'a>(pub Vec<SlogDifference<'a>>);
pub struct SlogDifference<'a>(pub Difference<'a>);

impl<'a> KV for SlogDifference<'a> {

    fn serialize(
        &self,
        _record: &Record,
        serializer: &mut Serializer,
    ) -> Result {
        serializer.emit_str("field", self.0.field.as_str())?;
        serializer.emit_str("left", format!("{:?}", self.0.left).as_str())?;
        serializer.emit_str("right", format!("{:?}", self.0.right).as_str())?;
        Ok(())
    }
}

impl<'a> KV for Differences<'a> {
    fn serialize(
        &self,
        _record: &Record,
        serializer: &mut Serializer,
    ) -> Result {
        for (index, item) in self.0.iter().enumerate() {
            serializer.emit_usize("field_number", index)?;
            item.serialize(_record, serializer)?;
        }
        Ok(())
    }
}

