use strum_macros::EnumIter;

use std::ops;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum Orientation {
    North,
    West,
    South,
    East,
}

impl Orientation {
    pub fn turn_left(&self) -> Orientation {
        match self {
            Orientation::North => Orientation::West,
            Orientation::West => Orientation::South,
            Orientation::South => Orientation::East,
            Orientation::East => Orientation::North,
        }
    }

    pub fn turn_right(&self) -> Orientation {
        match self {
            Orientation::North => Orientation::East,
            Orientation::East => Orientation::South,
            Orientation::South => Orientation::West,
            Orientation::West => Orientation::North,
        }
    }

    pub fn as_delta(&self) -> PositionDelta {
        match self {
            Orientation::North => PositionDelta::new(1, 0),
            Orientation::East => PositionDelta::new(0, 1),
            Orientation::South => PositionDelta::new(-1, 0),
            Orientation::West => PositionDelta::new(0, -1),
        }
    }
}

impl std::fmt::Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Orientation::North => write!(f, "N"),
            Orientation::South => write!(f, "S"),
            Orientation::East => write!(f, "E"),
            Orientation::West => write!(f, "W"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Coordinate {
    pub rank: u8,
    pub file: u8,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PositionDelta {
    pub rank_delta: i8,
    pub file_delta: i8,
}

impl PositionDelta {
    pub fn new(rank_delta: i8, file_delta: i8) -> PositionDelta {
        PositionDelta { file_delta, rank_delta}
    }

    pub fn zero() -> PositionDelta {
        PositionDelta::new(0, 0)
    }

}

impl Coordinate {
    pub fn new (rank: u8, file: u8) -> Coordinate {
        Coordinate {rank, file}
    }
}

impl ops::Add<PositionDelta> for Coordinate {
    type Output = Self;

    fn add(self, rhs: PositionDelta) -> Self::Output {
        Coordinate { rank: (self.rank as i8 + rhs.rank_delta) as u8, file: (self.file as i8 + rhs.file_delta) as u8 }
    }
}

impl ops::Sub<PositionDelta> for Coordinate {
    type Output = Self;

    fn sub(self, rhs: PositionDelta) -> Self::Output {
        Coordinate { rank: (self.rank as i8 - rhs.rank_delta) as u8, file: (self.file as i8 - rhs.file_delta) as u8 }
    }
}

impl ops::Sub<Coordinate> for Coordinate {
    type Output= PositionDelta;

    fn sub(self, rhs: Coordinate) -> Self::Output {
        PositionDelta { rank_delta: self.rank as i8 - rhs.rank as i8, file_delta: self.file as i8 - rhs.file as i8 }
    }
}

impl ops::Add<PositionDelta> for PositionDelta {
    type Output = Self;

    fn add(self, rhs: PositionDelta) -> Self::Output {
        PositionDelta { rank_delta: self.rank_delta + rhs.rank_delta, file_delta: self.file_delta + rhs.file_delta}
    }
}

impl ops::Sub<PositionDelta> for PositionDelta {
    type Output = Self;

    fn sub(self, rhs: PositionDelta) -> Self::Output {
        PositionDelta { rank_delta: self.rank_delta - rhs.rank_delta, file_delta: self.file_delta - rhs.file_delta}
    }
}

impl ops::Mul<i8> for PositionDelta {
    type Output = Self;

    fn mul(self, rhs: i8) -> Self::Output {
        PositionDelta { rank_delta: self.rank_delta * rhs, file_delta: self.file_delta * rhs }
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