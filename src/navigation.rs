use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use std::ops;

#[derive(Debug, Copy, Clone, PartialEq, EnumIter)]
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

    pub fn as_coordinate(&self) -> Coordinate {
        match self {
            Orientation::North => Coordinate::new(1, 0),
            Orientation::East => Coordinate::new(0, 1),
            Orientation::South => Coordinate::new(-1, 0),
            Orientation::West => Coordinate::new(0, -1),
        }
    }
}



#[derive(Debug, Copy, Clone)]
pub struct Coordinate {
    pub rank: i8,
    pub file: i8,
}

impl Coordinate {
    pub fn new (rank: i8, file: i8) -> Coordinate {
        Coordinate {rank, file}
    }
}

impl ops::Add<Coordinate> for Coordinate {
    type Output = Self;

    fn add(self, rhs: Coordinate) -> Self::Output {
        Coordinate { rank: self.rank + rhs.rank, file: self.file + rhs.file }
    }
}

impl ops::Sub<Coordinate> for Coordinate {
    type Output = Self;

    fn sub(self, rhs: Coordinate) -> Self::Output {
        Coordinate { rank: self.rank - rhs.rank, file: self.file - rhs.file }
    }
}

impl ops::Mul<i8> for Coordinate {
    type Output = Self;

    fn mul(self, rhs: i8) -> Self::Output {
        Coordinate { rank: self.rank * rhs, file: self.file * rhs }
    }
}