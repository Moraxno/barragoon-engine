use std::{fmt::Display, hash::Hash};
use std::fmt::Write;


use crate::navigation::{Coordinate, Orientation, PositionDelta};
use strum_macros::EnumIter;
use strum::IntoEnumIterator;

#[derive(Debug, Copy, Clone, PartialEq, EnumIter, Eq, Hash)]
pub enum TileType {
    Two,
    Three,
    Four,
}

impl TileType {
    pub fn full_stride_length(&self) -> u8 {
        match self {
            TileType::Two => 2,
            TileType::Three => 3,
            TileType::Four => 4,
        }
    }

    pub fn short_stride_length(&self) -> u8 {
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
                    all_strides.push(Stride::new_straight(start_direction, stride));
                }
            }
        }

        all_strides
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Stride {
    start_direction: Orientation,
    start_length: u8,
    bend_direction: Orientation,
    bend_length: u8,
}

pub struct StrideIterator<'a> {
    ref_stride: &'a Stride,
    index: u8,
    last_direction: Orientation,
    position_delta: PositionDelta
}

impl<'a> StrideIterator<'a> {
    fn new(stride: &'a Stride) -> StrideIterator<'a> {
        StrideIterator {
            ref_stride: stride,
            index: 0,
            last_direction: stride.start_direction,
            position_delta: PositionDelta::zero()
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Step {
    pub enter_direction: Orientation,
    pub leave_direction: Option<Orientation>,
    pub position_delta: PositionDelta
}

impl<'a> Iterator for StrideIterator<'a> {
    type Item = Step;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= (self.ref_stride.start_length + self.ref_stride.bend_length) {
            return None;
        }

        let leave_direction: Option<Orientation>;

        if self.index < self.ref_stride.start_length {
            leave_direction = Some(self.ref_stride.start_direction);
        } else if self.index < self.ref_stride.start_length + self.ref_stride.bend_length - 1 {
            leave_direction = Some(self.ref_stride.bend_direction);
        } else {
            leave_direction = None
        }

        self.position_delta = self.position_delta + self.last_direction.as_delta();

        let result = Some ( Step {
            enter_direction: self.last_direction,
            leave_direction: leave_direction,
            position_delta: self.position_delta,
        });

        if let Some(last_direction) = leave_direction {
            self.last_direction = last_direction;
        }

        self.index += 1;

        return result;
        
    }
}

impl Stride {
    pub fn new_bend(start_direction: Orientation, start_length: u8, bend_direction: Orientation, bend_length: u8) -> Stride {
        Stride {
            start_direction,
            start_length,
            bend_direction,
            bend_length,
        }
    }

    pub fn new_straight(start_direction: Orientation, start_length: u8) -> Stride {
        Stride {
            start_direction,
            start_length,
            bend_direction: start_direction,
            bend_length: 0,
        }
    }

    pub fn steps(&self) -> StrideIterator {
       StrideIterator::new(&self)
    }

    pub fn to_string(&self) -> String {
        let mut s = String::new();
        write!(s, "{}", self).unwrap();
        s
    }

}

impl std::fmt::Display for Stride {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.bend_length > 0 {
            write!(f, "{}{}{}{}", self.start_direction, self.start_length, self.bend_direction, self.bend_length)
        } else {
            write!(f, "{}{}", self.start_direction, self.start_length)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::Tile;

    use super::*;

    #[test]
    fn tiles_move_strides_count() {
        
        assert_eq!(TileType::Two.full_strides().len(), 3 * 4);
        assert_eq!(TileType::Three.full_strides().len(), 5 * 4);
        assert_eq!(TileType::Four.full_strides().len(), 7 * 4);
    }

    #[test]
    fn tiles_moves_are_unique() {
        for tile_type in TileType::iter() {
            let strides = tile_type.full_strides();
            let unique_strides = strides.clone().into_iter().collect::<HashSet<Stride>>();
            assert_eq!(strides.len(), unique_strides.len())
        }
    }

    #[test]
    fn strides_are_of_proper_length() {
        for tile_type in TileType::iter() {
            for stride in tile_type.full_strides() {
                let steps: Vec<Step> = stride.steps().collect();
                assert_eq!(steps.len(), tile_type.full_stride_length().into())
            }
        }
        
    }
}
