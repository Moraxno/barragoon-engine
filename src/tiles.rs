use crate::navigation::{Coordinate, Orientation};
use strum::IntoEnumIterator;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TileType {
    Two,
    Three,
    Four,
}

impl TileType {
    pub fn full_stride_length(&self) -> i8 {
        match self {
            TileType::Two => 2,
            TileType::Three => 3,
            TileType::Four => 4,
        }
    }

    pub fn short_stride_length(&self) -> i8 {
        match self {
            TileType::Two => 1,
            TileType::Three => 2,
            TileType::Four => 3,
        }
    }

    pub fn full_strides(&self) -> Vec<Stride> {
        let stride = self.full_stride_length();
        let mut all_strides = vec![];

        for start_direction in Orientation::iter() {
            for bend_point in 0..stride {
                let possible_directions: Vec<Orientation>;
                if bend_point != 0 {
                    all_strides.push(Stride::new_bend(
                        start_direction,
                        bend_point,
                        start_direction.turn_left(),
                        stride - bend_point,
                    ));
                    all_strides.push(Stride::new_bend(
                        start_direction,
                        bend_point,
                        start_direction.turn_right(),
                        stride - bend_point,
                    ));
                } else {
                    all_strides.push(Stride::new_straight(start_direction, bend_point));
                }
            }
        }

        all_strides
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Stride {
    start_direction: Orientation,
    start_length: i8,
    bend_direction: Orientation,
    bend_length: i8,
}



impl Stride {
    pub fn new_bend(start_direction: Orientation, start_length: i8, bend_direction: Orientation, bend_length: i8) -> Stride {
        Stride {
            start_direction,
            start_length,
            bend_direction,
            bend_length,
        }
    }

    pub fn new_straight(start_direction: Orientation, start_length: i8) -> Stride {
        Stride {
            start_direction,
            start_length,
            bend_direction: start_direction,
            bend_length: 0,
        }
    }

    pub fn step(self, index: u8) -> (Orientation, Coordinate, bool) {
        let index_i8 = index as i8;
        if index_i8 < self.start_length {
            return (
                self.start_direction, 
                self.start_direction.as_coordinate() * (index_i8 + 1), 
                index_i8 == (self.start_length + self.bend_length - 1)
            )
        } else if index_i8 < self.start_length + self.bend_length {
            return (
                self.bend_direction, 
                self.start_direction.as_coordinate() * (self.start_length ) + self.bend_direction.as_coordinate() * (index_i8 + 1),
                index_i8 == (self.start_length + self.bend_length - 1)
            )
        } else {
            panic!()
        }
    }
}