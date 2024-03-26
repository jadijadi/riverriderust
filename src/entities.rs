use crate::utilities::stout_ext::AsLocationTuple;

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
    pub c: u16,
    pub l: u16,
}

impl Location {
    pub fn from_loc_tuple(loc: impl AsLocationTuple) -> Self {
        let (c, l) = loc.as_loc_tuple();
        Self::new(c, l)
    }

    pub fn new(c: u16, l: u16) -> Self {
        Location { c, l }
    }

    pub fn up(&self) -> Self {
        Location::new(self.c, self.l.checked_sub(1).unwrap_or(0))
    }

    pub fn down(&self) -> Self {
        Location::new(self.c, self.l + 1)
    }

    pub fn left(&self) -> Self {
        Location::new(self.c.checked_sub(1).unwrap_or(0), self.l)
    }

    pub fn right(&self) -> Self {
        Location::new(self.c + 1, self.l)
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
        (other.l > self.l || self.l - other.l <= bottom)
            && (self.l > other.l || other.l - self.l <= top)
            && (other.c > self.c || self.c - other.c <= left)
            && (self.c > other.c || other.c - self.c <= right)
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

    pub fn go_up(&mut self) {
        self.traveled += 1;
        self.location = self.location.up()
    }

    pub fn go_down(&mut self) {
        self.traveled -= 1;
        self.location = self.location.down()
    }

    pub fn go_left(&mut self) {
        self.location = self.location.left()
    }

    pub fn go_right(&mut self) {
        self.location = self.location.right()
    }
}
