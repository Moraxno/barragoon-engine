use std::fmt::Write;
use std::hash::Hash;

use crate::navigation::{Orientation, PositionDelta};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Copy, Clone, PartialEq, EnumIter, Eq, Hash)]
pub enum TileType {
    Two,
    Three,
    Four,
}

impl TileType {
    #[must_use]
    pub fn full_stride_length(&self) -> u8 {
        match self {
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
        }
    }

    #[must_use]
    pub fn short_stride_length(&self) -> u8 {
        match self {
            Self::Two => 1,
            Self::Three => 2,
            Self::Four => 3,
        }
    }

    fn make_strides(&self, are_full_strides: bool) -> Vec<Stride> {
        let stride_length: u8;
        if are_full_strides {
            stride_length = self.full_stride_length();
        } else {
            stride_length = self.short_stride_length();
        }

        let mut all_strides = vec![];

        for start_direction in Orientation::iter() {
            for bend_point in 0..stride_length {
                if bend_point != 0 {
                    all_strides.push(Stride::new_bend(
                        start_direction,
                        bend_point,
                        start_direction.turn_left(),
                        stride_length - bend_point,
                        are_full_strides,
                    ));
                    all_strides.push(Stride::new_bend(
                        start_direction,
                        bend_point,
                        start_direction.turn_right(),
                        stride_length - bend_point,
                        are_full_strides,
                    ));
                } else {
                    all_strides.push(Stride::new_straight(start_direction, stride_length, are_full_strides));
                }
            }
        }
        all_strides
    }

    #[must_use]
    pub fn full_strides(&self) -> Vec<Stride> {
        self.make_strides(true)
    }

    #[must_use]
    pub fn short_strides(&self) -> Vec<Stride> {
        self.make_strides(false)
    }

    #[must_use]
    pub fn all_strides(&self) -> Vec<Stride> {
        let mut all_strides = self.full_strides();
        all_strides.append(&mut self.short_strides());

        all_strides
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Stride {
    start_direction: Orientation,
    start_length: u8,
    bend_direction: Orientation,
    bend_length: u8,
    is_full_stride: bool,
}

impl Stride {
    #[must_use]
    pub fn can_capture(&self) -> bool {
        self.is_full_stride
    }
}

pub struct StrideIterator<'a> {
    ref_stride: &'a Stride,
    index: u8,
    last_direction: Orientation,
    position_delta: PositionDelta,
}

impl<'a> StrideIterator<'a> {
    fn new(stride: &'a Stride) -> Self {
        StrideIterator {
            ref_stride: stride,
            index: 0,
            last_direction: stride.start_direction,
            position_delta: PositionDelta::zero(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Step {
    pub enter_direction: Orientation,
    pub leave_direction: Option<Orientation>,
    pub position_delta: PositionDelta,
}

impl std::fmt::Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut leave_str = String::new();
        if let Some(orientation) = self.leave_direction {
            write!(leave_str, "{orientation}")?;
        } else {
            write!(leave_str, "X")?;
        }
        write!(f, "{}>{}:{}", self.enter_direction, leave_str, self.position_delta)
    }
}

impl<'a> Iterator for StrideIterator<'a> {
    type Item = Step;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= (self.ref_stride.start_length + self.ref_stride.bend_length) {
            return None;
        }

        let leave_direction: Option<Orientation>;

        if self.index < self.ref_stride.start_length - 1 {
            leave_direction = Some(self.ref_stride.start_direction);
        } else if self.index < self.ref_stride.start_length + self.ref_stride.bend_length - 1 {
            leave_direction = Some(self.ref_stride.bend_direction);
        } else {
            leave_direction = None;
        }

        self.position_delta = self.position_delta + self.last_direction.as_delta();

        let result = Some(Step {
            enter_direction: self.last_direction,
            leave_direction,
            position_delta: self.position_delta,
        });

        if let Some(last_direction) = leave_direction {
            self.last_direction = last_direction;
        }

        self.index += 1;

        result
    }
}

impl Stride {
    #[must_use]
    pub fn new_bend(
        start_direction: Orientation,
        start_length: u8,
        bend_direction: Orientation,
        bend_length: u8,
        is_full_stride: bool,
    ) -> Self {
        Self {
            start_direction,
            start_length,
            bend_direction,
            bend_length,
            is_full_stride,
        }
    }

    #[must_use]
    pub fn new_straight(start_direction: Orientation, start_length: u8, is_full_stride: bool) -> Self {
        Self {
            start_direction,
            start_length,
            bend_direction: start_direction,
            bend_length: 0,
            is_full_stride,
        }
    }

    #[must_use]
    pub fn steps(&self) -> StrideIterator {
        StrideIterator::new(self)
    }

    #[must_use]
    pub fn to_string(&self) -> String {
        let mut s = String::new();
        write!(s, "{self}").unwrap();
        s
    }

    #[must_use]
    pub fn full_delta(&self) -> PositionDelta {
        self.start_direction.as_delta() * self.start_length as i8 + self.bend_direction.as_delta() * self.bend_length as i8
    }
}

impl std::fmt::Display for Stride {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.bend_length > 0 {
            write!(
                f,
                "{}{}{}{}",
                self.start_direction, self.start_length, self.bend_direction, self.bend_length
            )
        } else {
            write!(f, "{}{}", self.start_direction, self.start_length)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{SquareContent, Tile};

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
            assert_eq!(strides.len(), unique_strides.len());
        }
    }

    #[test]
    fn strides_are_of_proper_length() {
        for tile_type in TileType::iter() {
            for stride in tile_type.full_strides() {
                let steps: Vec<Step> = stride.steps().collect();
                assert_eq!(steps.len(), tile_type.full_stride_length() as usize);
            }
        }
    }

    #[test]
    fn single_tile_on_board_has_all_valid_moves() {
        for tile_type in TileType::iter() {
            let mut game = crate::Game::empty();
            game.board[4][3] = SquareContent::Tile(Tile {
                tile_type,
                player: crate::Player::White,
            });
            let moves = game.valid_moves();

            match tile_type {
                TileType::Two => assert_eq!(moves.len(), 4 + 8),
                TileType::Three => assert_eq!(moves.len(), 8 + 12),
                TileType::Four => assert_eq!(moves.len(), 12 + 14), // Four collides with sides of the board
            }
        }
    }

    #[test]
    fn straight_stride_deltas_are_consistent() {
        assert_eq!(
            Stride::new_straight(Orientation::East, 7, true).full_delta(),
            PositionDelta::new(0, 7)
        );
        assert_eq!(
            Stride::new_straight(Orientation::West, 3, false).full_delta(),
            PositionDelta::new(0, -3)
        );
        assert_eq!(
            Stride::new_straight(Orientation::North, 4, true).full_delta(),
            PositionDelta::new(4, 0)
        );
        assert_eq!(
            Stride::new_straight(Orientation::South, 5, false).full_delta(),
            PositionDelta::new(-5, 0)
        );
    }

    #[test]
    fn bend_stride_deltas_are_consistent() {
        assert_eq!(
            Stride::new_bend(Orientation::East, 7, Orientation::North, 2, true).full_delta(),
            PositionDelta::new(2, 7)
        );
        assert_eq!(
            Stride::new_bend(Orientation::West, 3, Orientation::South, 4, false).full_delta(),
            PositionDelta::new(-4, -3)
        );
        assert_eq!(
            Stride::new_bend(Orientation::North, 4, Orientation::East, 6, true).full_delta(),
            PositionDelta::new(4, 6)
        );
        assert_eq!(
            Stride::new_bend(Orientation::South, 5, Orientation::West, 8, false).full_delta(),
            PositionDelta::new(-5, -8)
        );
    }
}
