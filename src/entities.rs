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

#[derive(PartialEq, Eq)]
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
    pub armor: u16,
}

impl Enemy {
    pub fn new(armor: u16) -> Enemy {
        Enemy { armor }
    }
}

impl From<Enemy> for EntityType {
    fn from(value: Enemy) -> Self {
        EntityType::Enemy(value)
    }
}

pub struct Bullet {
    pub energy: u16,
    pub location: Location,
}

impl Bullet {
    pub fn new(loc: impl AsLocationTuple, energy: u16) -> Self {
        Self {
            energy,
            location: Location::from_loc_tuple(loc),
        }
    }
}

impl_located!(Bullet);

pub struct Fuel;

impl From<Fuel> for EntityType {
    fn from(value: Fuel) -> Self {
        EntityType::Fuel(value)
    }
}

pub enum EntityType {
    Enemy(Enemy),
    Fuel(Fuel),
}

impl EntityType {
    /// Returns `true` if the entity type is [`Fuel`].
    ///
    /// [`Fuel`]: EntityType::Fuel
    #[must_use]
    pub fn is_fuel(&self) -> bool {
        matches!(self, Self::Fuel(..))
    }

    /// Returns `true` if the entity type is [`Enemy`].
    ///
    /// [`Enemy`]: EntityType::Enemy
    #[must_use]
    pub fn is_enemy(&self) -> bool {
        matches!(self, Self::Enemy(..))
    }

    pub fn as_fuel(&self) -> Option<&Fuel> {
        if let Self::Fuel(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_enemy(&self) -> Option<&Enemy> {
        if let Self::Enemy(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

pub struct Entity {
    pub location: Location,
    pub status: EntityStatus,
    pub entity_type: EntityType,
}

impl Entity {
    pub fn new(loc: impl AsLocationTuple, entity_type: impl Into<EntityType>) -> Self {
        Self {
            status: EntityStatus::Alive,
            location: Location::from_loc_tuple(loc),
            entity_type: entity_type.into(),
        }
    }
}

impl_located!(Entity);

pub struct Player {
    pub location: Location,
    pub status: PlayerStatus,
    pub fuel: u16,
    pub score: u16,
    pub traveled: u16,

    pub bullets: Vec<Bullet>,
}

impl Player {
    pub fn new(loc: impl AsLocationTuple, fuel: u16) -> Self {
        Self {
            location: Location::from_loc_tuple(loc),
            status: PlayerStatus::Alive,
            fuel,
            score: 0,
            traveled: 0,

            bullets: Vec::new(),
        }
    }

    pub fn go_up(&mut self) -> &mut Location {
        // Must not be here
        self.traveled += 1;
        self.location.go_up()
    }

    pub fn go_down(&mut self) -> &mut Location {
        // Must not be here
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

impl_located!(Player);

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
