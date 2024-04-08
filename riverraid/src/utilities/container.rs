#![allow(dead_code)]

use std::ops::Range;

use super::stout_ext::Located;

pub struct Container<Idx> {
    lines: Range<Idx>,
    columns: Range<Idx>,
}

impl Container<u16> {
    pub fn contains_loc(&self, location: impl Located) -> bool {
        let location = location.location();
        self.contains(&location.line, &location.column)
    }

    pub fn is_under_loc(&self, location: impl Located) -> bool {
        self.is_under(location.location().line)
    }

    pub fn is_upper_loc(&self, location: impl Located) -> bool {
        self.is_upper(location.location().line)
    }

    pub fn is_righter_loc(&self, location: impl Located) -> bool {
        self.is_righter(location.location().column)
    }

    pub fn is_lefter_loc(&self, location: impl Located) -> bool {
        self.is_lefter(location.location().column)
    }
}

impl<Idx> Container<Idx>
where
    Idx: PartialOrd<Idx>,
{
    pub fn contains(&self, line: &Idx, column: &Idx) -> bool {
        self.lines.contains(line) && self.columns.contains(column)
    }

    pub fn is_under(&self, line: Idx) -> bool {
        line > self.lines.start
    }

    pub fn is_upper(&self, line: Idx) -> bool {
        line < self.lines.end
    }

    pub fn is_righter(&self, column: Idx) -> bool {
        column > self.columns.start
    }

    pub fn is_lefter(&self, column: Idx) -> bool {
        column < self.columns.start
    }
}

impl<Idx> Container<Idx> {
    pub fn new(lines: impl Into<Range<Idx>>, columns: impl Into<Range<Idx>>) -> Self {
        Self {
            lines: lines.into(),
            columns: columns.into(),
        }
    }

    pub fn lines(&self) -> &Range<Idx> {
        &self.lines
    }

    pub fn columns(&self) -> &Range<Idx> {
        &self.columns
    }
}
