use std::slice::Iter;

use crate::common::navigation::Direction;
use crate::common::tiles::TileType;
use crate::common::tiles::Renderable;

type Dir = Direction;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Alignment {
    Horizontal,
    Vertical,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BarragoonFace {
    Blocking,
    Straight { alignment: Alignment },
    OneWay { direction: Direction },
    OneWayTurnLeft { direction: Direction },
    OneWayTurnRight { direction: Direction },
    ForceTurn,
}

impl BarragoonFace {
    pub fn can_be_captured_from(&self, enter_dir: &Direction) -> bool {
        match self {
            Self::ForceTurn | Self::Blocking => true,
            Self::Straight {
                alignment: Alignment::Vertical,
            } => *enter_dir == Dir::North || *enter_dir == Dir::South,
            Self::Straight {
                alignment: Alignment::Horizontal,
            } => *enter_dir == Dir::West || *enter_dir == Dir::East,
            Self::OneWay { direction: one_way_dir } => one_way_dir == enter_dir,
            Self::OneWayTurnLeft { direction: Dir::South } | Self::OneWayTurnRight { direction: Dir::North } => *enter_dir == Dir::West,
            Self::OneWayTurnLeft { direction: Dir::North } | Self::OneWayTurnRight { direction: Dir::South } => *enter_dir == Dir::East,
            Self::OneWayTurnLeft { direction: Dir::East } | Self::OneWayTurnRight { direction: Dir::West } => *enter_dir == Dir::South,
            Self::OneWayTurnLeft { direction: Dir::West } | Self::OneWayTurnRight { direction: Dir::East } => *enter_dir == Dir::North,
        }
    }

    pub fn can_be_captured_by(&self, tile_type: TileType) -> bool {
        tile_type != TileType::Two || *self != Self::ForceTurn
    }

    pub fn can_be_traversed(self, enter_dir: Direction, leave_dir: Direction) -> bool {
        let is_horizontal = enter_dir == Dir::East && leave_dir == Dir::West || enter_dir == Dir::West && leave_dir == Dir::East;

        let is_vertical = enter_dir == Dir::South && leave_dir == Dir::North || enter_dir == Dir::North && leave_dir == Dir::South;

        let is_left_turn = enter_dir == Dir::North && leave_dir == Dir::West
            || enter_dir == Dir::South && leave_dir == Dir::East
            || enter_dir == Dir::East && leave_dir == Dir::North
            || enter_dir == Dir::West && leave_dir == Dir::South;

        let is_right_turn = enter_dir == Dir::North && leave_dir == Dir::East
            || enter_dir == Dir::South && leave_dir == Dir::West
            || enter_dir == Dir::East && leave_dir == Dir::South
            || enter_dir == Dir::West && leave_dir == Dir::North;

        if u8::from(is_horizontal) + u8::from(is_vertical) + u8::from(is_left_turn) + u8::from(is_right_turn) != 1 {
            return false;
        }

        match self {
            Self::Blocking => false,
            Self::ForceTurn => is_left_turn || is_right_turn,
            Self::Straight {
                alignment: Alignment::Vertical,
            } => is_vertical,
            Self::Straight {
                alignment: Alignment::Horizontal,
            } => is_horizontal,
            Self::OneWay { direction } => direction == enter_dir && (is_horizontal || is_vertical),
            Self::OneWayTurnLeft { direction } => is_left_turn && leave_dir == direction,
            Self::OneWayTurnRight { direction } => is_right_turn && leave_dir == direction,
        }
    }

    pub fn as_fen_char(&self) -> char {
        match self {
            Self::ForceTurn => '+',
            Self::Straight {
                alignment: Alignment::Vertical,
            } => '|',
            Self::Straight {
                alignment: Alignment::Horizontal,
            } => '-',
            Self::OneWay { direction: Dir::South } => 'Y',
            Self::OneWay { direction: Dir::North } => '^',
            Self::OneWay { direction: Dir::West } => '<',
            Self::OneWay { direction: Dir::East } => '>',
            Self::Blocking => 'x',
            Self::OneWayTurnLeft { direction: Dir::South } => 'S',
            Self::OneWayTurnLeft { direction: Dir::North } => 'N',
            Self::OneWayTurnLeft { direction: Dir::East } => 'E',
            Self::OneWayTurnLeft { direction: Dir::West } => 'W',
            Self::OneWayTurnRight { direction: Dir::South } => 's',
            Self::OneWayTurnRight { direction: Dir::North } => 'n',
            Self::OneWayTurnRight { direction: Dir::East } => 'e',
            Self::OneWayTurnRight { direction: Dir::West } => 'w',
        }
    }

    pub fn all_faces() -> Iter<'static, Self> {
        static FACES: [BarragoonFace; 16] = [
            BarragoonFace::Blocking,
            BarragoonFace::Straight {
                alignment: Alignment::Horizontal,
            },
            BarragoonFace::Straight {
                alignment: Alignment::Vertical,
            },
            BarragoonFace::OneWay { direction: Dir::North },
            BarragoonFace::OneWay { direction: Dir::South },
            BarragoonFace::OneWay { direction: Dir::East },
            BarragoonFace::OneWay { direction: Dir::West },
            BarragoonFace::OneWayTurnLeft { direction: Dir::North },
            BarragoonFace::OneWayTurnLeft { direction: Dir::South },
            BarragoonFace::OneWayTurnLeft { direction: Dir::East },
            BarragoonFace::OneWayTurnLeft { direction: Dir::West },
            BarragoonFace::OneWayTurnRight { direction: Dir::North },
            BarragoonFace::OneWayTurnRight { direction: Dir::South },
            BarragoonFace::OneWayTurnRight { direction: Dir::East },
            BarragoonFace::OneWayTurnRight { direction: Dir::West },
            BarragoonFace::ForceTurn,
        ];
        FACES.iter()
    }
}

impl Renderable for BarragoonFace {
    fn as_cli_char(&self) -> char {
        match self {
            Self::Blocking => '⨯',
            Self::Straight { alignment: Alignment::Horizontal } => '↔',
            Self::Straight { alignment: Alignment::Vertical } => '↕',
            Self::OneWay { direction: Direction::South }  => '↓',
            Self::OneWay { direction: Direction::North }  => '↑',
            Self::OneWay { direction: Direction::West } => '←',
            Self::OneWay { direction: Direction::East } => '→',
            Self::OneWayTurnLeft { direction: Direction::South } => '⬐',
            Self::OneWayTurnLeft { direction: Direction::North } => '⬏',
            Self::OneWayTurnLeft { direction: Direction::East } => '↳',
            Self::OneWayTurnLeft { direction: Direction::West } => '↰',
            Self::OneWayTurnRight { direction: Direction::South } => '⬎',
            Self::OneWayTurnRight { direction: Direction::North } => '⬑',
            Self::OneWayTurnRight { direction: Direction::East } => '↱',
            Self::OneWayTurnRight { direction: Direction::West } => '↵',
            Self::ForceTurn => '⯌'
        }
    }
}
