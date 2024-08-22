use strum_macros::EnumIter;

use std::ops;

use crate::{BOARD_HEIGHT, BOARD_WIDTH, FILE_NAMES, RANK_NAMES};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum Direction {
    North,
    West,
    South,
    East,
}

impl Direction {
    #[must_use]
    pub fn turn_left(&self) -> Self {
        match self {
            Self::North => Self::West,
            Self::West => Self::South,
            Self::South => Self::East,
            Self::East => Self::North,
        }
    }

    #[must_use]
    pub fn turn_right(&self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }

    #[must_use]
    pub fn as_delta(&self) -> PositionDelta {
        match self {
            Self::North => PositionDelta::new(1, 0),
            Self::East => PositionDelta::new(0, 1),
            Self::South => PositionDelta::new(-1, 0),
            Self::West => PositionDelta::new(0, -1),
        }
    }
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::North => write!(f, "N"),
            Self::South => write!(f, "S"),
            Self::East => write!(f, "E"),
            Self::West => write!(f, "W"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Coordinate {
    pub rank: u8,
    pub file: u8,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PositionDelta {
    pub rank_delta: i8,
    pub file_delta: i8,
}

impl PositionDelta {
    #[must_use]
    pub fn new(rank_delta: i8, file_delta: i8) -> Self {
        Self { rank_delta, file_delta }
    }

    #[must_use]
    pub fn zero() -> Self {
        Self::new(0, 0)
    }
}

impl std::fmt::Display for PositionDelta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "d({},{})", self.rank_delta, self.file_delta)
    }
}

impl Coordinate {
    #[must_use]
    pub fn new(rank: u8, file: u8) -> Self {
        Self { rank, file }
    }
}

impl std::fmt::Display for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.file >= BOARD_WIDTH {
            write!(f, "?")?;
        } else {
            write!(f, "{}", FILE_NAMES[self.file as usize])?;
        }

        if self.rank >= BOARD_HEIGHT {
            write!(f, "?")
        } else {
            write!(f, "{}", RANK_NAMES[self.rank as usize])
        }
    }
}

impl ops::Add<PositionDelta> for Coordinate {
    type Output = Self;

    fn add(self, rhs: PositionDelta) -> Self::Output {
        Self::new((self.rank as i8 + rhs.rank_delta) as u8, (self.file as i8 + rhs.file_delta) as u8)
    }
}

impl ops::Sub<PositionDelta> for Coordinate {
    type Output = Self;

    fn sub(self, rhs: PositionDelta) -> Self::Output {
        Self::new((self.rank as i8 - rhs.rank_delta) as u8, (self.file as i8 - rhs.file_delta) as u8)
    }
}

impl ops::Sub<Self> for Coordinate {
    type Output = PositionDelta;

    fn sub(self, rhs: Self) -> Self::Output {
        PositionDelta::new(self.rank as i8 - rhs.rank as i8, self.file as i8 - rhs.file as i8)
    }
}

impl ops::Add<Self> for PositionDelta {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.rank_delta + rhs.rank_delta, self.file_delta + rhs.file_delta)
    }
}

impl ops::Sub<Self> for PositionDelta {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.rank_delta - rhs.rank_delta, self.file_delta - rhs.file_delta)
    }
}

impl ops::Mul<i8> for PositionDelta {
    type Output = Self;

    fn mul(self, rhs: i8) -> Self::Output {
        Self::new(self.rank_delta * rhs, self.file_delta * rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adding_deltas() {
        assert_eq!(PositionDelta::new(4, 3) + PositionDelta::new(4, 3), PositionDelta::new(8, 6));
        assert_eq!(PositionDelta::new(4, -2) + PositionDelta::new(-9, 17), PositionDelta::new(-5, 15));
        assert_eq!(PositionDelta::new(4, -2) + PositionDelta::new(0, 0), PositionDelta::new(4, -2));
        assert_eq!(PositionDelta::new(0, 0) + PositionDelta::new(4, -2), PositionDelta::new(4, -2));
    }

    #[test]
    fn subtracting_deltas() {
        assert_eq!(PositionDelta::new(4, 3) - PositionDelta::new(4, 3), PositionDelta::new(0, 0));
        assert_eq!(PositionDelta::new(4, -2) - PositionDelta::new(-9, 17), PositionDelta::new(13, -19));
        assert_eq!(PositionDelta::new(4, -2) - PositionDelta::new(0, 0), PositionDelta::new(4, -2));
        assert_eq!(PositionDelta::new(0, 0) - PositionDelta::new(4, -2), PositionDelta::new(-4, 2));
    }

    #[test]
    fn add_delta_to_coordinate() {
        assert_eq!(Coordinate::new(4, 3) + PositionDelta::new(4, 3), Coordinate::new(8, 6));
        assert_eq!(Coordinate::new(4, 2) + PositionDelta::new(-1, 5), Coordinate::new(3, 7));
        assert_eq!(Coordinate::new(4, 2) + PositionDelta::new(0, 0), Coordinate::new(4, 2));
    }

    #[test]
    fn subtract_delta_from_coordinate() {
        assert_eq!(Coordinate::new(4, 3) - PositionDelta::new(4, 3), Coordinate::new(0, 0));
        assert_eq!(Coordinate::new(4, 2) - PositionDelta::new(-1, 1), Coordinate::new(5, 1));
        assert_eq!(Coordinate::new(4, 2) - PositionDelta::new(0, 0), Coordinate::new(4, 2));
    }

    #[test]
    fn multiply_deltas() {
        assert_eq!(PositionDelta::new(4, 3) * 1, PositionDelta::new(4, 3));
        assert_eq!(PositionDelta::new(4, 3) * -1, PositionDelta::new(-4, -3));
        assert_eq!(PositionDelta::new(4, 3) * 0, PositionDelta::new(0, 0));
        assert_eq!(PositionDelta::new(0, 0) * 1, PositionDelta::new(0, 0));
        assert_eq!(PositionDelta::new(0, 0) * -1, PositionDelta::new(0, 0));
        assert_eq!(PositionDelta::new(0, 0) * 0, PositionDelta::new(0, 0));
    }
}
