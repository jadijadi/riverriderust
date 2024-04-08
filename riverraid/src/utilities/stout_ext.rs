//! This module contains extension to [`Stdout`] object.

use std::{fmt::Display, io::Stdout};

use crossterm::{cursor::MoveTo, style::Print, terminal::Clear, QueueableCommand};

use crate::entities::Location;

pub type StdoutResult<'a> = Result<&'a mut Stdout, std::io::Error>;

pub trait Located {
    fn location(&self) -> &Location;
}

impl<T: Located> AsLocationTuple for T {
    fn as_loc_tuple(&self) -> (u16, u16) {
        (self.location().column, self.location().line)
    }
}

pub trait AsLocationTuple {
    fn as_loc_tuple(&self) -> (u16, u16);
}

impl AsLocationTuple for (u16, u16) {
    fn as_loc_tuple(&self) -> (u16, u16) {
        (self.0, self.1)
    }
}

impl AsLocationTuple for u16 {
    fn as_loc_tuple(&self) -> (u16, u16) {
        (*self, *self)
    }
}

pub trait StdoutExt {
    fn clear_all(&mut self) -> StdoutResult;

    fn move_cursor(&mut self, loc: impl AsLocationTuple) -> StdoutResult;

    fn print(&mut self, display: impl Display) -> StdoutResult;

    fn draw(&mut self, loc: impl AsLocationTuple, display: impl Display) -> StdoutResult;
}

impl StdoutExt for Stdout {
    fn move_cursor(&mut self, loc: impl AsLocationTuple) -> StdoutResult {
        let (c, l) = loc.as_loc_tuple();
        self.queue(MoveTo(c, l))
    }

    fn draw(&mut self, loc: impl AsLocationTuple, display: impl Display) -> StdoutResult {
        self.move_cursor(loc)?.print(display)
    }

    fn clear_all(&mut self) -> StdoutResult {
        self.queue(Clear(crossterm::terminal::ClearType::All))
    }

    fn print(&mut self, display: impl Display) -> StdoutResult {
        self.queue(Print(display))
    }
}
