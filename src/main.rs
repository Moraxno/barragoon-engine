use std::arch::x86_64::_mm_cmpeq_pd;
use std::collections::{HashMap, HashSet};
use std::io::{self, BufReader};

use strum::IntoEnumIterator;

use navigation::Coordinate;
use tiles::Step;

use crate::navigation::Orientation;
use crate::tiles::TileType;
use crate::ubi::ubi_loop;

pub mod application;
pub mod navigation;
pub mod tiles;
pub mod ubi;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Player {
    White,
    Brown,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum BarragoonAlignment {
    Horizontal,
    Vertical,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum BarragoonFace {
    Blocking,
    Straight { alignment: BarragoonAlignment },
    OneWay { orientation: Orientation },
    OneWayTurnLeft { orientation: Orientation },
    OneWayTurnRight { orientation: Orientation },
    ForceTurn,
}

impl BarragoonFace {
    pub fn can_be_captured_from(&self, enter_orientation: &Orientation) -> bool {
        match self {
            BF::ForceTurn => true,
            BF::Straight { alignment: BA::Vertical } => *enter_orientation == BO::North || *enter_orientation == BO::South,
            BF::Straight { alignment: BA::Horizontal } => *enter_orientation == BO::West || *enter_orientation == BO::East,
            BF::OneWay {
                orientation: one_way_orientation,
            } => one_way_orientation == enter_orientation,
            BF::Blocking => true,
            BF::OneWayTurnLeft { orientation: BO::South } => *enter_orientation == BO::West,
            BF::OneWayTurnLeft { orientation: BO::North } => *enter_orientation == BO::East,
            BF::OneWayTurnLeft { orientation: BO::East } => *enter_orientation == BO::South,
            BF::OneWayTurnLeft { orientation: BO::West } => *enter_orientation == BO::North,
            BF::OneWayTurnRight { orientation: BO::South } => *enter_orientation == BO::East,
            BF::OneWayTurnRight { orientation: BO::North } => *enter_orientation == BO::West,
            BF::OneWayTurnRight { orientation: BO::East } => *enter_orientation == BO::North,
            BF::OneWayTurnRight { orientation: BO::West } => *enter_orientation == BO::South,
        }
    }

    pub fn can_be_captured_by(&self, tile_type: TileType) -> bool {
        tile_type != TileType::Two || *self != BF::ForceTurn
    }

    pub fn can_be_traversed(&self, enter: &Orientation, leave: &Orientation) -> bool {
        let is_horizontal = *enter == BO::East && *leave == BO::West || *enter == BO::West && *leave == BO::East;

        let is_vertical = *enter == BO::South && *leave == BO::North || *enter == BO::North && *leave == BO::South;

        let is_left_turn = *enter == BO::North && *leave == BO::West
            || *enter == BO::South && *leave == BO::East
            || *enter == BO::East && *leave == BO::North
            || *enter == BO::West && *leave == BO::South;

        let is_right_turn = *enter == BO::North && *leave == BO::East
            || *enter == BO::South && *leave == BO::West
            || *enter == BO::East && *leave == BO::South
            || *enter == BO::West && *leave == BO::North;

        if (is_horizontal as u8) + (is_vertical as u8) + (is_left_turn as u8) + (is_right_turn as u8) != 1 {
            return false;
        }

        match self {
            BF::ForceTurn => is_left_turn || is_right_turn,
            BF::Straight { alignment: BA::Vertical } => is_vertical,
            BF::Straight { alignment: BA::Horizontal } => is_horizontal,
            BF::OneWay {
                orientation: one_way_orientation,
            } => one_way_orientation == enter && (is_horizontal || is_vertical),
            BF::Blocking => false,
            BF::OneWayTurnLeft {
                orientation: barragoon_orientation,
            } => is_left_turn && leave == barragoon_orientation,
            BF::OneWayTurnRight {
                orientation: barragoon_orientation,
            } => is_right_turn && leave == barragoon_orientation,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct SquareView<'a> {
    coordinate: Coordinate,
    content: &'a SquareContent,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum SquareContent {
    Empty,
    Tile(Tile),
    Barragoon(BarragoonFace),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Tile {
    tile_type: TileType,
    player: Player,
}

impl Tile {
    pub fn to_fen_char(&self) -> char {
        match self.player {
            Player::White => match self.tile_type {
                TileType::Two => 'Z',
                TileType::Three => 'D',
                TileType::Four => 'V',
            },
            Player::Brown => match self.tile_type {
                TileType::Two => 'z',
                TileType::Three => 'd',
                TileType::Four => 'v',
            },
        }
    }
}

impl SquareContent {
    pub fn to_fen_char(&self) -> char {
        match self {
            SC::Empty => ' ',
            SC::Tile(tile) => tile.to_fen_char(),
            SC::Barragoon(BF::ForceTurn) => '+',
            SC::Barragoon(BF::Straight { alignment: BA::Vertical }) => '|',
            SC::Barragoon(BF::Straight { alignment: BA::Horizontal }) => '-',
            SC::Barragoon(BF::OneWay { orientation: BO::South }) => 'Y',
            SC::Barragoon(BF::OneWay { orientation: BO::North }) => '^',
            SC::Barragoon(BF::OneWay { orientation: BO::West }) => '<',
            SC::Barragoon(BF::OneWay { orientation: BO::East }) => '>',
            SC::Barragoon(BF::Blocking) => 'x',
            SC::Barragoon(BF::OneWayTurnLeft { orientation: BO::South }) => 'S',
            SC::Barragoon(BF::OneWayTurnLeft { orientation: BO::North }) => 'N',
            SC::Barragoon(BF::OneWayTurnLeft { orientation: BO::East }) => 'E',
            SC::Barragoon(BF::OneWayTurnLeft { orientation: BO::West }) => 'W',
            SC::Barragoon(BF::OneWayTurnRight { orientation: BO::South }) => 's',
            SC::Barragoon(BF::OneWayTurnRight { orientation: BO::North }) => 'n',
            SC::Barragoon(BF::OneWayTurnRight { orientation: BO::East }) => 'e',
            SC::Barragoon(BF::OneWayTurnRight { orientation: BO::West }) => 'w',
        }
    }
}

const BOARD_WIDTH: u8 = 7;
const BOARD_HEIGHT: u8 = 9;
const INITIAL_FEN_STRING: &str = "1vd1dv1/2zdz2/7/1x3x1/x1x1x1x/1x3x1/7/2ZDZ2/1VD1DV1";

type SC = SquareContent;

#[derive(Debug, Copy, Clone)]
struct Game {
    board: [[SC; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
    current_player: Player,
}

#[derive(Debug, Copy, Clone)]
enum FenError {
    UnderfullLine { char_index: usize },
    OverfullLine { char_index: usize },
    TooManyLines { char_index: usize },
    InvalidChar { char_index: usize },
}

#[derive(Debug, Copy, Clone)]
enum FenParseObject {
    JumpCol(u8),
    SkipRow,
    Square(SquareContent),
    InvalidChar,
}
type FPO = FenParseObject;
type BA = BarragoonAlignment;
type BO = Orientation;
type BF = BarragoonFace;

struct SquareIterator<'a> {
    owner_game: &'a Game,

    ifile: u8,
    irank: u8,
}

impl<'a> Iterator for SquareIterator<'a> {
    type Item = SquareView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ifile >= BOARD_WIDTH {
            self.irank += 1;
            self.ifile = 0;
        }

        let result: Option<Self::Item>;

        if self.irank >= BOARD_HEIGHT {
            result = None
        } else {
            result = Some(SquareView {
                coordinate: Coordinate::new(self.irank, self.ifile),
                content: &self.owner_game.board[self.irank as usize][self.ifile as usize],
            })
        }

        self.ifile += 1;

        result
    }
}

impl Game {
    pub fn new() -> Game {
        Game::from_fen(INITIAL_FEN_STRING).unwrap()
    }

    pub fn empty() -> Game {
        Game::from_fen("7/7/7/7/7/7/7/7/7").unwrap()
    }

    pub fn squares(&self) -> SquareIterator<'_> {
        SquareIterator {
            owner_game: self,
            ifile: 0,
            irank: 0,
        }
    }

    pub fn contains_coordinate(&self, coordinate: &Coordinate) -> bool {
        coordinate.rank < BOARD_HEIGHT && coordinate.file < BOARD_WIDTH
    }

    pub fn get_content(&self, coordinate: &Coordinate) -> &SquareContent {
        &self.board[coordinate.rank as usize][coordinate.file as usize]
    }

    pub fn from_fen(fen: &str) -> Result<Game, FenError> {
        let mut board: [[SC; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize] = [[SC::Empty; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize];

        let mut row_ptr: i8 = BOARD_HEIGHT as i8 - 1;
        let mut col_ptr: u8 = 0;

        for (index, c) in fen.char_indices() {
            let obj: FenParseObject = match c {
                'Z' => FPO::Square(SC::Tile(Tile {
                    tile_type: TileType::Two,
                    player: Player::White,
                })),
                'z' => FPO::Square(SC::Tile(Tile {
                    tile_type: TileType::Two,
                    player: Player::Brown,
                })),
                'D' => FPO::Square(SC::Tile(Tile {
                    tile_type: TileType::Three,
                    player: Player::White,
                })),
                'd' => FPO::Square(SC::Tile(Tile {
                    tile_type: TileType::Three,
                    player: Player::Brown,
                })),
                'V' => FPO::Square(SC::Tile(Tile {
                    tile_type: TileType::Four,
                    player: Player::White,
                })),
                'v' => FPO::Square(SC::Tile(Tile {
                    tile_type: TileType::Four,
                    player: Player::Brown,
                })),
                '+' => FPO::Square(SC::Barragoon(BF::ForceTurn)),
                '|' => FPO::Square(SC::Barragoon(BF::Straight { alignment: BA::Vertical })),
                '-' => FPO::Square(SC::Barragoon(BF::Straight { alignment: BA::Horizontal })),
                'Y' => FPO::Square(SC::Barragoon(BF::OneWay { orientation: BO::South })),
                '^' => FPO::Square(SC::Barragoon(BF::OneWay { orientation: BO::North })),
                '<' => FPO::Square(SC::Barragoon(BF::OneWay { orientation: BO::West })),
                '>' => FPO::Square(SC::Barragoon(BF::OneWay { orientation: BO::East })),
                'x' => FPO::Square(SC::Barragoon(BF::Blocking)),
                'S' => FPO::Square(SC::Barragoon(BF::OneWayTurnLeft { orientation: BO::South })),
                'N' => FPO::Square(SC::Barragoon(BF::OneWayTurnLeft { orientation: BO::North })),
                'E' => FPO::Square(SC::Barragoon(BF::OneWayTurnLeft { orientation: BO::East })),
                'W' => FPO::Square(SC::Barragoon(BF::OneWayTurnLeft { orientation: BO::West })),
                's' => FPO::Square(SC::Barragoon(BF::OneWayTurnRight { orientation: BO::South })),
                'n' => FPO::Square(SC::Barragoon(BF::OneWayTurnRight { orientation: BO::North })),
                'e' => FPO::Square(SC::Barragoon(BF::OneWayTurnRight { orientation: BO::East })),
                'w' => FPO::Square(SC::Barragoon(BF::OneWayTurnRight { orientation: BO::West })),
                '1'..='7' => FPO::JumpCol(c.to_digit(10).map(|d| d as u8).unwrap()),
                '/' => FPO::SkipRow,
                _ => FPO::InvalidChar,
            };

            match obj {
                FPO::Square(content) => {
                    board[row_ptr as usize][col_ptr as usize] = content;
                    col_ptr += 1;
                }
                FPO::JumpCol(cols) => {
                    col_ptr += cols;
                    if col_ptr > BOARD_WIDTH {
                        return Result::Err(FenError::OverfullLine { char_index: index });
                    }
                }
                FPO::SkipRow => {
                    if col_ptr == BOARD_WIDTH {
                        col_ptr = 0;
                        row_ptr -= 1;
                    } else {
                        return Result::Err(FenError::UnderfullLine { char_index: index });
                    }

                    if row_ptr < 0 {
                        return Result::Err(FenError::TooManyLines { char_index: index });
                    }
                }
                FPO::InvalidChar => {
                    return Result::Err(FenError::InvalidChar { char_index: index });
                }
            }
        }

        // todo: initialize player from fen string
        Ok(Game {
            board,
            current_player: Player::White,
        })
    }

    pub fn to_fen(&self) -> String {
        let mut fen_string = String::new();

        for row in self.board.iter().rev() {
            let mut empty_count = 0;
            for square in row {
                if *square == SquareContent::Empty {
                    empty_count += 1;
                } else {
                    if empty_count > 0 {
                        fen_string.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }
                    fen_string.push(square.to_fen_char());
                }
            }
            if empty_count > 0 {
                fen_string.push_str(&empty_count.to_string());
            }

            fen_string.push('/');
        }

        fen_string.pop(); /* remove the last slash we just pushed */
        fen_string
    }

    pub fn valid_moves(&self) -> Vec<Move> {
        let mut moves = vec![];

        for square in self.squares() {
            let mut covered_squares = HashSet::<Coordinate>::new();

            if let SC::Tile(moving_tile) = square.content {
                let Tile {
                    tile_type: moving_tile_type,
                    player: moving_piece_player,
                } = moving_tile;

                // skip other players pieces
                if *moving_piece_player != self.current_player {
                    continue;
                }

                let all_strides = moving_tile_type.all_strides();
                for stride in all_strides {
                    let coordinate_to_cover = square.coordinate + stride.full_delta();
                    if !self.contains_coordinate(&coordinate_to_cover) {
                        // non-existent square
                        continue;
                    }

                    if covered_squares.contains(&coordinate_to_cover) {
                        // already have a way there, don't need to check
                        continue;
                    }

                    for full_step in stride.steps() {
                        let new_coordinate = square.coordinate + full_step.position_delta;
                        if !self.contains_coordinate(&new_coordinate) {
                            //todo(robo) maybe breaking here is fine ... please test this later
                            continue;
                        }

                        let target_square_content = self.get_content(&new_coordinate);
                        let is_last_step = full_step.leave_direction.is_none();

                        match target_square_content {
                            SC::Tile(attacked_tile) => {
                                let Tile {
                                    tile_type: attacked_tile_type,
                                    player: colliding_piece_player,
                                } = attacked_tile;
                                if (moving_piece_player == colliding_piece_player) || !is_last_step || !stride.can_capture() {
                                    break;
                                }

                                moves.push(Move::TileCapture {
                                    from: (*moving_tile, square.coordinate),
                                    to: (*attacked_tile, new_coordinate),
                                });
                                covered_squares.insert(new_coordinate);
                            }
                            SC::Empty => {
                                if is_last_step {
                                    moves.push(Move::Straight {
                                        moving_tile: *moving_tile,
                                        start: square.coordinate,
                                        stop: new_coordinate,
                                    });
                                    covered_squares.insert(new_coordinate);
                                }
                            }
                            SC::Barragoon(face) => {
                                if is_last_step {
                                    if stride.can_capture()
                                        && face.can_be_captured_by(*moving_tile_type)
                                        && face.can_be_captured_from(&full_step.enter_direction)
                                    {
                                        moves.push(Move::BarragoonCapture {
                                            start: square.coordinate,
                                            stop: new_coordinate,
                                        });
                                        covered_squares.insert(new_coordinate);
                                    } else {
                                        break;
                                    }
                                } else if !face.can_be_traversed(&full_step.enter_direction, &full_step.leave_direction.unwrap()) {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        moves
    }
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "  ")?;
        for _ in 0..BOARD_WIDTH {
            write!(f, "+---")?;
        }
        writeln!(f, "+")?;

        for irank in (0..BOARD_HEIGHT as usize).rev() {
            let rank = self.board[irank];
            f.write_fmt(format_args!("{} ", RANK_NAMES[irank]))?;
            for square in rank {
                write!(f, "| ")?;
                f.write_fmt(format_args!("{}", square.to_fen_char()))?;
                write!(f, " ")?;
            }
            write!(f, "|\n  ")?;
            for _ in 0..BOARD_WIDTH {
                write!(f, "+---")?;
            }
            writeln!(f, "+")?;
        }

        write!(f, "  ")?;
        for ifile in 0..BOARD_WIDTH as usize {
            f.write_fmt(format_args!("  {} ", FILE_NAMES[ifile]))?;
        }

        write!(f, "")
    }
}

const RANK_NAMES: [char; BOARD_HEIGHT as usize] = ['1', '2', '3', '4', '5', '6', '7', '8', '9'];
const FILE_NAMES: [char; BOARD_WIDTH as usize] = ['a', 'b', 'c', 'd', 'e', 'f', 'g'];

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Move {
    Straight {
        moving_tile: Tile,
        start: Coordinate,
        stop: Coordinate,
    },
    TileCapture {
        from: (Tile, Coordinate),
        to: (Tile, Coordinate),
    },
    BarragoonCapture {
        start: Coordinate,
        stop: Coordinate,
    },
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Move::Straight { moving_tile, start, stop } = self {
            f.write_fmt(format_args!("{}{}{}", moving_tile.to_fen_char(), start, stop))?;
        } else if let Move::TileCapture {
            from: (attacker, start),
            to: (victim, stop),
        } = self
        {
            f.write_fmt(format_args!("{}{}x{}{}", attacker.to_fen_char(), start, victim.to_fen_char(), stop))?;
        }

        write!(f, "")
    }
}

fn main() {
    println!("Hello, world!");

    let game = Game::new();
    println!("{:?}", game);
    println!("{}", game);
    println!("{}", INITIAL_FEN_STRING);
    println!("{}", game.to_fen());

    for tile_move in game.valid_moves() {
        println!("{}", tile_move)
    }

    println!("{:?}", TileType::Three.full_strides());

    let mut buf_stdin = BufReader::new(io::stdin());

    ubi_loop(&mut buf_stdin, &mut io::stdout());
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::Tile;

    use super::*;

    #[test]

    fn initial_gamestate_allowed_moves() {
        let moves = Game::new().valid_moves();
        assert_eq!(moves.len(), 28)
    }

    #[test]
    fn initial_gamestate_moves_are_unique() {
        let moves = Game::new().valid_moves();
        let unique_moves: HashSet<Move> = moves.clone().into_iter().collect();

        assert_eq!(moves.len(), unique_moves.len())
    }
}
