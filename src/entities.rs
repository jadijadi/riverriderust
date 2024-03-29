use crate::utilities::stout_ext::{AsLocationTuple, Located};

#[derive(PartialEq, Eq)]
pub enum DeathCause {
    Enemy,
    Ground,
    Fuel,
}

#[derive(PartialEq, Eq)]
pub enum PlayerStatus {
    Dead(DeathCause),
    Alive,
    Quit,
}

pub enum EntityStatus {
    Alive,
    DeadBody,
    Dead,
}

#[derive(Clone)]
pub struct Location {
    pub column: u16,
    pub line: u16,
}

impl From<(u16, u16)> for Location {
    fn from((col, line): (u16, u16)) -> Self {
        Location::new(col, line)
    }
}

impl From<Location> for (u16, u16) {
    fn from(value: Location) -> Self {
        (value.column, value.line)
    }
}

impl Located for Location {
    fn location(&self) -> &Location {
        self
    }
}

impl Located for &Location {
    fn location(&self) -> &Location {
        self
    }
}

impl Located for &mut Location {
    fn location(&self) -> &Location {
        self
    }
}

impl Location {
    pub fn from_loc_tuple(loc: impl AsLocationTuple) -> Self {
        let (c, l) = loc.as_loc_tuple();
        Self::new(c, l)
    }

    pub fn new(column: u16, line: u16) -> Self {
        Location { column, line }
    }

    pub fn go_up(&mut self) -> &mut Location {
        self.line = self.line.checked_sub(1).unwrap_or(0);
        self
    }

    pub fn go_down(&mut self) -> &mut Location {
        self.line += 1;
        self
    }

    pub fn go_left(&mut self) -> &mut Location {
        self.column = self.column.checked_sub(1).unwrap_or(0);
        self
    }

    pub fn go_right(&mut self) -> &mut Location {
        self.column += 1;
        self
    }

    pub fn up(&self) -> Self {
        Location::new(self.column, self.line.checked_sub(1).unwrap_or(0))
    }

    pub fn down(&self) -> Self {
        Location::new(self.column, self.line + 1)
    }

    pub fn left(&self) -> Self {
        Location::new(self.column.checked_sub(1).unwrap_or(0), self.line)
    }

    pub fn right(&self) -> Self {
        Location::new(self.column + 1, self.line)
    }

    // Checks if two locations are within a specified margin of each other
    pub fn hit_with_margin(
        &self,
        other: &Location,
        top: u16,
        right: u16,
        bottom: u16,
        left: u16,
    ) -> bool {
        (other.line > self.line || self.line - other.line <= bottom)
            && (self.line > other.line || other.line - self.line <= top)
            && (other.column > self.column || self.column - other.column <= left)
            && (self.column > other.column || other.column - self.column <= right)
    }

    // check if two locations is point to the same location
    pub fn hit(&self, other: &Location) -> bool {
        self.hit_with_margin(other, 0, 0, 0, 0)
    }
} // end of Location implementation.

pub struct Enemy {
    pub location: Location,
    pub status: EntityStatus,
    pub armor: u16,
}

impl_located!(Enemy);

impl Enemy {
    pub fn new(loc: impl AsLocationTuple, armor: u16) -> Enemy {
        Enemy {
            location: Location::from_loc_tuple(loc),
            status: EntityStatus::Alive,
            armor,
        }
    }
} // end of Enemy implementation.

pub struct Bullet {
    pub location: Location,
    pub energy: u16,
}

impl_located!(Bullet);

impl Bullet {
    pub fn new(loc: impl AsLocationTuple, energy: u16) -> Bullet {
        Bullet {
            location: Location::from_loc_tuple(loc),
            energy,
        }
    }
} // end of Bullet implementation.

pub struct Fuel {
    pub location: Location,
    pub status: EntityStatus,
}

impl_located!(Fuel);

impl Fuel {
    pub fn new(loc: impl AsLocationTuple, status: EntityStatus) -> Fuel {
        Fuel {
            location: Location::from_loc_tuple(loc),
            status,
        }
    }
} // end of Fuel implementation.

pub struct Player {
    pub location: Location,
    pub status: PlayerStatus,
    pub fuel: u16,
    pub score: u16,
    pub traveled: u16,
}

impl_located!(Player);

impl Player {
    pub fn new(loc: impl AsLocationTuple, fuel: u16) -> Self {
        Self {
            location: Location::from_loc_tuple(loc),
            status: PlayerStatus::Alive,
            fuel,
            score: 0,
            traveled: 0,
        }
    }

    pub fn go_up(&mut self) -> &mut Location {
        self.traveled += 1;
        self.location.go_up()
    }

    pub fn go_down(&mut self) -> &mut Location {
        self.traveled -= 1;
        self.location.go_down()
    }

    pub fn go_left(&mut self) -> &mut Location {
        self.location.go_left()
    }

    pub fn go_right(&mut self) -> &mut Location {
        self.location.go_right()
    }
}

macro_rules! impl_located {
    ($tp: ident) => {
        impl Located for $tp {
            fn location(&self) -> &Location {
                &self.location
            }
        }

        impl Located for &$tp {
            fn location(&self) -> &Location {
                &self.location
            }
        }

        impl Located for &mut $tp {
            fn location(&self) -> &Location {
                &self.location
            }
        }
    };
}

use impl_located;
