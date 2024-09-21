#![allow(clippy::trivially_copy_pass_by_ref)]
use std::collections::HashSet;
use std::io::{self, BufReader};

use barragoon_engine::common::{navigation, tiles, pieces};

use crate::navigation::{Coordinate, FILE_NAMES, RANK_NAMES, BOARD_HEIGHT, BOARD_HEIGHT_SIGNED, BOARD_WIDTH, Direction};

use barragoon_engine::common::tiles::{Renderable, TileType};

pub mod application;
pub mod ubi;

use ubi::run_loop;
use crate::pieces::barragoon::{Alignment, BarragoonFace};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Player {
    Light,
    Dark,
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
    pub const fn as_fen_char(&self) -> char {
        match self.player {
            Player::Light => match self.tile_type {
                TileType::Two => 'Z',
                TileType::Three => 'D',
                TileType::Four => 'V',
            },
            Player::Dark => match self.tile_type {
                TileType::Two => 'z',
                TileType::Three => 'd',
                TileType::Four => 'v',
            },
        }
    }
}

impl Renderable for Tile {
    fn as_cli_char(&self) -> char {
        match self.player {
            Player::Light => match self.tile_type {
                TileType::Two => '➋',
                TileType::Three => '➌',
                TileType::Four => '➍',
            },
            Player::Dark => match self.tile_type {
                TileType::Two => '➁',
                TileType::Three => '➂',
                TileType::Four => '➃',
            },
        }
    }
}

impl Renderable for SquareContent {
    fn as_cli_char(&self) -> char {
        match self {
            Self::Empty => ' ',
            Self::Tile(tile) => tile.as_cli_char(),
            Self::Barragoon(face) => face.as_cli_char(),
        }
    }
}


// no need to wrap Barragoons and TIles into seperate structs. They can just be fat enums... ? hmm, but then there are no seperate impls for captures and stuff
impl SquareContent {
    pub const fn as_fen_char(&self) -> char {
        match self {
            Self::Empty => ' ',
            Self::Tile(tile) => tile.as_fen_char(),
            Self::Barragoon(Bf::ForceTurn) => '+',
            Self::Barragoon(Bf::Straight { alignment: Ba::Vertical }) => '|',
            Self::Barragoon(Bf::Straight { alignment: Ba::Horizontal }) => '-',
            Self::Barragoon(Bf::OneWay { direction: Bd::South }) => 'Y',
            Self::Barragoon(Bf::OneWay { direction: Bd::North }) => '^',
            Self::Barragoon(Bf::OneWay { direction: Bd::West }) => '<',
            Self::Barragoon(Bf::OneWay { direction: Bd::East }) => '>',
            Self::Barragoon(Bf::Blocking) => 'x',
            Self::Barragoon(Bf::OneWayTurnLeft { direction: Bd::South }) => 'S',
            Self::Barragoon(Bf::OneWayTurnLeft { direction: Bd::North }) => 'N',
            Self::Barragoon(Bf::OneWayTurnLeft { direction: Bd::East }) => 'E',
            Self::Barragoon(Bf::OneWayTurnLeft { direction: Bd::West }) => 'W',
            Self::Barragoon(Bf::OneWayTurnRight { direction: Bd::South }) => 's',
            Self::Barragoon(Bf::OneWayTurnRight { direction: Bd::North }) => 'n',
            Self::Barragoon(Bf::OneWayTurnRight { direction: Bd::East }) => 'e',
            Self::Barragoon(Bf::OneWayTurnRight { direction: Bd::West }) => 'w',
        }
    }
}

const INITIAL_FEN_STRING: &str = "1vd1dv1/2zdz2/7/1x3x1/x1x1x1x/1x3x1/7/2ZDZ2/1VD1DV1";
const EMPTY_FEN_STRING: &str = "7/7/7/7/7/7/7/7/7";

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
type Fpo = FenParseObject;
type Ba = Alignment;
type Bd = Direction;
type Bf = BarragoonFace;

struct SquareIterator<'a> {
    owner_game: &'a Game,

    ifile: u8,
    irank: u8,
}

#[derive(Debug)]
enum MoveError {
    ForeignConstructedMoveUsed = 0,
}

impl<'a> Iterator for SquareIterator<'a> {
    type Item = SquareView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ifile >= BOARD_WIDTH {
            self.irank += 1;
            self.ifile = 0;
        }

        let result = if self.irank >= BOARD_HEIGHT {
            None
        } else {
            Some(SquareView {
                coordinate: Coordinate::new(self.irank, self.ifile),
                content: &self.owner_game.board[self.irank as usize][self.ifile as usize],
            })
        };

        self.ifile += 1;

        result
    }
}

impl Game {
    pub fn new() -> Self {
        Self::from_fen(INITIAL_FEN_STRING).expect("Start position FEN string is corrupted.")
    }

    pub fn empty() -> Self {
        Self::from_fen(EMPTY_FEN_STRING).expect("Empty position FEN string is corrupted.")
    }

    pub const fn squares(&self) -> SquareIterator<'_> {
        SquareIterator {
            owner_game: self,
            ifile: 0,
            irank: 0,
        }
    }

    pub const fn contains_coordinate(coordinate: &Coordinate) -> bool {
        coordinate.rank < BOARD_HEIGHT && coordinate.file < BOARD_WIDTH
    }

    pub const fn get_content(&self, coordinate: &Coordinate) -> &SquareContent {
        &self.board[coordinate.rank as usize][coordinate.file as usize]
    }

    pub fn set_content(&mut self, coordinate: &Coordinate, content: &SquareContent) {
        self.board[coordinate.rank as usize][coordinate.file as usize] = *content;
    }

    pub fn move_content(&mut self, from: &Coordinate, to: &Coordinate) {
        self.board[to.rank as usize][to.file as usize] = self.board[from.rank as usize][from.file as usize];
        self.board[from.rank as usize][from.file as usize] = SquareContent::Empty;
    }

    pub fn from_fen(fen: &str) -> Result<Self, FenError> {
        let mut board: [[SC; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize] = [[SC::Empty; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize];

        let mut row_ptr: i8 = BOARD_HEIGHT_SIGNED - 1;
        let mut col_ptr: u8 = 0;

        for (index, c) in fen.char_indices() {
            let obj: FenParseObject = match c {
                'Z' => Fpo::Square(SC::Tile(Tile {
                    tile_type: TileType::Two,
                    player: Player::Light,
                })),
                'z' => Fpo::Square(SC::Tile(Tile {
                    tile_type: TileType::Two,
                    player: Player::Dark,
                })),
                'D' => Fpo::Square(SC::Tile(Tile {
                    tile_type: TileType::Three,
                    player: Player::Light,
                })),
                'd' => Fpo::Square(SC::Tile(Tile {
                    tile_type: TileType::Three,
                    player: Player::Dark,
                })),
                'V' => Fpo::Square(SC::Tile(Tile {
                    tile_type: TileType::Four,
                    player: Player::Light,
                })),
                'v' => Fpo::Square(SC::Tile(Tile {
                    tile_type: TileType::Four,
                    player: Player::Dark,
                })),
                '+' => Fpo::Square(SC::Barragoon(Bf::ForceTurn)),
                '|' => Fpo::Square(SC::Barragoon(Bf::Straight { alignment: Ba::Vertical })),
                '-' => Fpo::Square(SC::Barragoon(Bf::Straight { alignment: Ba::Horizontal })),
                'Y' => Fpo::Square(SC::Barragoon(Bf::OneWay { direction: Bd::South })),
                '^' => Fpo::Square(SC::Barragoon(Bf::OneWay { direction: Bd::North })),
                '<' => Fpo::Square(SC::Barragoon(Bf::OneWay { direction: Bd::West })),
                '>' => Fpo::Square(SC::Barragoon(Bf::OneWay { direction: Bd::East })),
                'x' => Fpo::Square(SC::Barragoon(Bf::Blocking)),
                'S' => Fpo::Square(SC::Barragoon(Bf::OneWayTurnLeft { direction: Bd::South })),
                'N' => Fpo::Square(SC::Barragoon(Bf::OneWayTurnLeft { direction: Bd::North })),
                'E' => Fpo::Square(SC::Barragoon(Bf::OneWayTurnLeft { direction: Bd::East })),
                'W' => Fpo::Square(SC::Barragoon(Bf::OneWayTurnLeft { direction: Bd::West })),
                's' => Fpo::Square(SC::Barragoon(Bf::OneWayTurnRight { direction: Bd::South })),
                'n' => Fpo::Square(SC::Barragoon(Bf::OneWayTurnRight { direction: Bd::North })),
                'e' => Fpo::Square(SC::Barragoon(Bf::OneWayTurnRight { direction: Bd::East })),
                'w' => Fpo::Square(SC::Barragoon(Bf::OneWayTurnRight { direction: Bd::West })),
                '1'..='7' => Fpo::JumpCol(
                    c.to_digit(10)
                        .map(|d| u8::try_from(d).expect("Cannot parse digit."))
                        .ok_or(FenError::InvalidChar { char_index: index })?,
                ),
                '/' => Fpo::SkipRow,
                _ => Fpo::InvalidChar,
            };

            let row_idx = usize::try_from(row_ptr).expect("Row pointer was negative");

            match obj {
                Fpo::Square(content) => {
                    board[row_idx][col_ptr as usize] = content;
                    col_ptr += 1;
                }
                Fpo::JumpCol(cols) => {
                    col_ptr += cols;
                    if col_ptr > BOARD_WIDTH {
                        return Result::Err(FenError::OverfullLine { char_index: index });
                    }
                }
                Fpo::SkipRow => {
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
                Fpo::InvalidChar => {
                    return Result::Err(FenError::InvalidChar { char_index: index });
                }
            }
        }

        // todo: initialize player from fen string
        Ok(Self {
            board,
            current_player: Player::Light,
        })
    }

    pub fn make_move(&mut self, board_move: &BoardMove) -> Result<(), MoveError> {
        let valid_moves: HashSet<BoardMove> = self.valid_moves().into_iter().collect();

        if !valid_moves.contains(board_move) {
            return Err(MoveError::ForeignConstructedMoveUsed);
        }

        match board_move {
            BoardMove::Straight { start, stop, tile }
            | BoardMove::TileCapture {
                start,
                stop,
                tile,
                victim: _,
            } => {
                self.set_content(stop, &SquareContent::Tile(*tile));
                self.set_content(start, &SquareContent::Empty);
            }
            BoardMove::BarragoonCapture {
                start,
                stop,
                tile,
                victim: _,
                target,
                barragoon,
            } => {
                self.set_content(stop, &SquareContent::Tile(*tile));
                self.set_content(start, &SquareContent::Empty);
                self.set_content(target, &SquareContent::Barragoon(*barragoon));
            }
            BoardMove::BarragoonPlacement { target, barragoon } => {
                self.set_content(target, &SquareContent::Barragoon(*barragoon));
            }
        }

        Ok(())
    }

    pub fn as_fen(&self) -> String {
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
                    fen_string.push(square.as_fen_char());
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

    pub fn valid_moves(&self) -> Vec<BoardMove> {
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
                    if !Self::contains_coordinate(&coordinate_to_cover) {
                        // non-existent square
                        continue;
                    }

                    if covered_squares.contains(&coordinate_to_cover) {
                        // already have a way there, don't need to check
                        continue;
                    }

                    for full_step in stride.steps() {
                        let new_coordinate = square.coordinate + full_step.position_delta;
                        if !Self::contains_coordinate(&new_coordinate) {
                            //todo(robo) maybe breaking here is fine ... please test this later
                            continue;
                        }

                        let target_square_content = self.get_content(&new_coordinate);

                        let is_last_step = full_step.leave_direction.is_none();

                        match target_square_content {
                            SC::Tile(attacked_tile) => {
                                let Tile {
                                    tile_type: _,
                                    player: colliding_piece_player,
                                } = attacked_tile;
                                if (moving_piece_player == colliding_piece_player) || !is_last_step || !stride.can_capture() {
                                    break;
                                }

                                moves.push(BoardMove::TileCapture {
                                    start: square.coordinate,
                                    stop: new_coordinate,
                                    tile: *moving_tile,
                                    victim: *attacked_tile,
                                });
                                covered_squares.insert(new_coordinate);
                            }
                            SC::Empty => {
                                if is_last_step {
                                    moves.push(BoardMove::Straight {
                                        tile: *moving_tile,
                                        start: square.coordinate,
                                        stop: new_coordinate,
                                    });
                                    covered_squares.insert(new_coordinate);
                                }
                            }
                            SC::Barragoon(face) => {
                                if let Some(leave_direction) = full_step.leave_direction {
                                    if !face.can_be_traversed(full_step.enter_direction, leave_direction) {
                                        break;
                                    }
                                } else if stride.can_capture()
                                    && face.can_be_captured_by(*moving_tile_type)
                                    && face.can_be_captured_from(&full_step.enter_direction)
                                {
                                    for target_square in self.squares() {
                                        if *target_square.content != SquareContent::Empty && target_square.coordinate != square.coordinate
                                        {
                                            continue;
                                        }

                                        for new_face in BarragoonFace::all_faces() {
                                            moves.push(BoardMove::BarragoonCapture {
                                                start: square.coordinate,
                                                stop: new_coordinate,
                                                tile: *moving_tile,
                                                victim: *face,
                                                barragoon: *new_face,
                                                target: target_square.coordinate,
                                            });
                                        }
                                    }
                                    covered_squares.insert(new_coordinate);
                                } else {
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
            write!(f, "┼───")?;
        }
        writeln!(f, "┼")?;

        for irank in (0..BOARD_HEIGHT as usize).rev() {
            let rank = self.board[irank];
            f.write_fmt(format_args!("{} ", RANK_NAMES[irank]))?;
            for square in rank {
                write!(f, "│ ")?;
                f.write_fmt(format_args!("{}", square.as_cli_char()))?;
                write!(f, " ")?;
            }
            write!(f, "│\n  ")?;
            for _ in 0..BOARD_WIDTH {
                write!(f, "┼───")?;
            }
            writeln!(f, "┼")?;
        }

        write!(f, "  ")?;
        for name_of_file in FILE_NAMES {
            f.write_fmt(format_args!("  {name_of_file} "))?;
        }

        write!(f, "")
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct BarragoonPlacement {
    coordinate: Coordinate,
    face: BarragoonFace,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum BoardMove {
    Straight {
        start: Coordinate,
        stop: Coordinate,
        tile: Tile,
    },
    TileCapture {
        start: Coordinate,
        stop: Coordinate,
        tile: Tile,
        victim: Tile,
    },
    BarragoonCapture {
        start: Coordinate,
        stop: Coordinate,
        tile: Tile,
        victim: BarragoonFace,
        target: Coordinate,
        barragoon: BarragoonFace,
    },
    BarragoonPlacement {
        target: Coordinate,
        barragoon: BarragoonFace,
    },
}

impl std::fmt::Display for BoardMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match &self {
            Self::Straight { start, stop, tile } => write!(f, "{}{}{}", tile.as_fen_char(), start, stop),
            Self::TileCapture { start, stop, tile, victim } => {
                write!(f, "{}{}x{}{}", tile.as_fen_char(), start, victim.as_fen_char(), stop)
            }
            Self::BarragoonCapture {
                start,
                stop,
                tile,
                victim,
                target,
                barragoon,
            } => write!(
                f,
                "{}{}o{}{}!{}{}",
                tile.as_fen_char(),
                start,
                victim.as_fen_char(),
                stop,
                barragoon.as_fen_char(),
                target
            ),
            Self::BarragoonPlacement { target, barragoon } => write!(f, "!{}{}", barragoon.as_fen_char(), target),
        };
    }
}

fn main() {
    println!("Hello, world!");

    let mut game: Game = Game::new();
    game.set_content(&Coordinate { rank: 4, file: 2 }, &SquareContent::Barragoon(BarragoonFace::OneWayTurnLeft { direction: Direction::South }));
    game.set_content(&Coordinate { rank: 5, file: 6 }, &SquareContent::Barragoon(BarragoonFace::Straight { alignment: Alignment::Vertical }));
    println!("{game:?}");
    println!("{game}");
    println!("{INITIAL_FEN_STRING}");
    println!("{}", game.as_fen());


    for tile_move in game.valid_moves() {
        println!("{tile_move}");
    }

    println!("{:?}", TileType::Three.full_strides());

    let mut buf_stdin = BufReader::new(io::stdin());

    run_loop(&mut buf_stdin, &mut io::stdout()).expect("Something went wrong while reading input.");
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn empty_game_is_empty() {
        let game = Game::empty();

        for row in game.board {
            for cell in row {
                assert_eq!(cell, SquareContent::Empty);
            }
        }
    }

    #[test]
    fn game_startpos_according_to_rules() {
        let game = Game::new();

        // first rank
        assert_eq!(*game.get_content(&Coordinate::new(0, 0)), SquareContent::Empty);
        assert_eq!(
            *game.get_content(&Coordinate::new(0, 1)),
            SquareContent::Tile(Tile {
                tile_type: TileType::Four,
                player: Player::Light
            })
        );
        assert_eq!(
            *game.get_content(&Coordinate::new(0, 2)),
            SquareContent::Tile(Tile {
                tile_type: TileType::Three,
                player: Player::Light
            })
        );
        assert_eq!(*game.get_content(&Coordinate::new(0, 3)), SquareContent::Empty);
        assert_eq!(
            *game.get_content(&Coordinate::new(0, 4)),
            SquareContent::Tile(Tile {
                tile_type: TileType::Three,
                player: Player::Light
            })
        );
        assert_eq!(
            *game.get_content(&Coordinate::new(0, 5)),
            SquareContent::Tile(Tile {
                tile_type: TileType::Four,
                player: Player::Light
            })
        );
        assert_eq!(*game.get_content(&Coordinate::new(0, 6)), SquareContent::Empty);

        // second rank
        assert_eq!(*game.get_content(&Coordinate::new(1, 0)), SquareContent::Empty);
        assert_eq!(*game.get_content(&Coordinate::new(1, 1)), SquareContent::Empty);
        assert_eq!(
            *game.get_content(&Coordinate::new(1, 2)),
            SquareContent::Tile(Tile {
                tile_type: TileType::Two,
                player: Player::Light
            })
        );
        assert_eq!(
            *game.get_content(&Coordinate::new(1, 3)),
            SquareContent::Tile(Tile {
                tile_type: TileType::Three,
                player: Player::Light
            })
        );
        assert_eq!(
            *game.get_content(&Coordinate::new(1, 4)),
            SquareContent::Tile(Tile {
                tile_type: TileType::Two,
                player: Player::Light
            })
        );
        assert_eq!(*game.get_content(&Coordinate::new(1, 5)), SquareContent::Empty);
        assert_eq!(*game.get_content(&Coordinate::new(1, 6)), SquareContent::Empty);

        // third rank
        for file in 0..7 {
            assert_eq!(*game.get_content(&Coordinate::new(2, file)), SquareContent::Empty);
        }

        // forth rank
        assert_eq!(*game.get_content(&Coordinate::new(3, 0)), SquareContent::Empty);
        assert_eq!(
            *game.get_content(&Coordinate::new(3, 1)),
            SquareContent::Barragoon(BarragoonFace::Blocking)
        );
        assert_eq!(*game.get_content(&Coordinate::new(3, 2)), SquareContent::Empty);
        assert_eq!(*game.get_content(&Coordinate::new(3, 3)), SquareContent::Empty);
        assert_eq!(*game.get_content(&Coordinate::new(3, 4)), SquareContent::Empty);
        assert_eq!(
            *game.get_content(&Coordinate::new(3, 5)),
            SquareContent::Barragoon(BarragoonFace::Blocking)
        );
        assert_eq!(*game.get_content(&Coordinate::new(3, 6)), SquareContent::Empty);

        // fifth rank
        assert_eq!(
            *game.get_content(&Coordinate::new(4, 0)),
            SquareContent::Barragoon(BarragoonFace::Blocking)
        );
        assert_eq!(*game.get_content(&Coordinate::new(4, 1)), SquareContent::Empty);
        assert_eq!(
            *game.get_content(&Coordinate::new(4, 2)),
            SquareContent::Barragoon(BarragoonFace::Blocking)
        );
        assert_eq!(*game.get_content(&Coordinate::new(4, 3)), SquareContent::Empty);
        assert_eq!(
            *game.get_content(&Coordinate::new(4, 4)),
            SquareContent::Barragoon(BarragoonFace::Blocking)
        );
        assert_eq!(*game.get_content(&Coordinate::new(4, 5)), SquareContent::Empty);
        assert_eq!(
            *game.get_content(&Coordinate::new(4, 6)),
            SquareContent::Barragoon(BarragoonFace::Blocking)
        );

        // sixth rank
        assert_eq!(*game.get_content(&Coordinate::new(5, 0)), SquareContent::Empty);
        assert_eq!(
            *game.get_content(&Coordinate::new(5, 1)),
            SquareContent::Barragoon(BarragoonFace::Blocking)
        );
        assert_eq!(*game.get_content(&Coordinate::new(5, 2)), SquareContent::Empty);
        assert_eq!(*game.get_content(&Coordinate::new(5, 3)), SquareContent::Empty);
        assert_eq!(*game.get_content(&Coordinate::new(5, 4)), SquareContent::Empty);
        assert_eq!(
            *game.get_content(&Coordinate::new(5, 5)),
            SquareContent::Barragoon(BarragoonFace::Blocking)
        );
        assert_eq!(*game.get_content(&Coordinate::new(5, 6)), SquareContent::Empty);

        // seventh rank
        for file in 0..7 {
            assert_eq!(*game.get_content(&Coordinate::new(6, file)), SquareContent::Empty);
        }

        // eigth rank
        assert_eq!(*game.get_content(&Coordinate::new(7, 0)), SquareContent::Empty);
        assert_eq!(*game.get_content(&Coordinate::new(7, 1)), SquareContent::Empty);
        assert_eq!(
            *game.get_content(&Coordinate::new(7, 2)),
            SquareContent::Tile(Tile {
                tile_type: TileType::Two,
                player: Player::Dark
            })
        );
        assert_eq!(
            *game.get_content(&Coordinate::new(7, 3)),
            SquareContent::Tile(Tile {
                tile_type: TileType::Three,
                player: Player::Dark
            })
        );
        assert_eq!(
            *game.get_content(&Coordinate::new(7, 4)),
            SquareContent::Tile(Tile {
                tile_type: TileType::Two,
                player: Player::Dark
            })
        );
        assert_eq!(*game.get_content(&Coordinate::new(7, 5)), SquareContent::Empty);
        assert_eq!(*game.get_content(&Coordinate::new(7, 6)), SquareContent::Empty);

        // ninth rank
        assert_eq!(*game.get_content(&Coordinate::new(8, 0)), SquareContent::Empty);
        assert_eq!(
            *game.get_content(&Coordinate::new(8, 1)),
            SquareContent::Tile(Tile {
                tile_type: TileType::Four,
                player: Player::Dark
            })
        );
        assert_eq!(
            *game.get_content(&Coordinate::new(8, 2)),
            SquareContent::Tile(Tile {
                tile_type: TileType::Three,
                player: Player::Dark
            })
        );
        assert_eq!(*game.get_content(&Coordinate::new(8, 3)), SquareContent::Empty);
        assert_eq!(
            *game.get_content(&Coordinate::new(8, 4)),
            SquareContent::Tile(Tile {
                tile_type: TileType::Three,
                player: Player::Dark
            })
        );
        assert_eq!(
            *game.get_content(&Coordinate::new(8, 5)),
            SquareContent::Tile(Tile {
                tile_type: TileType::Four,
                player: Player::Dark
            })
        );
        assert_eq!(*game.get_content(&Coordinate::new(8, 6)), SquareContent::Empty);
    }

    #[test]
    fn initial_gamestate_has_28_straight_moves() {
        let moves = Game::new().valid_moves();
        assert_eq!(moves.len(), 28);
        let straight_moves = moves.iter().filter(|move_| match move_ {
            BoardMove::Straight {
                start: _,
                stop: _,
                tile: _,
            } => true,
            _ => false,
        });
        assert_eq!(straight_moves.collect::<Vec<&BoardMove>>().len(), 28)
    }

    #[test]
    fn game_makes_a_valid_move() {
        let mut g = Game::new();
        let start_pos = Coordinate { rank: 1, file: 2 };
        let stop_pos = Coordinate { rank: 3, file: 2 };
        let tile = Tile {
            tile_type: TileType::Two,
            player: Player::Light,
        };
        let board_move = BoardMove::Straight {
            start: start_pos,
            stop: stop_pos,
            tile: tile,
        };
        
        let r = g.make_move(&board_move);

        assert!(r.is_ok());
        assert_eq!(g.get_content(&start_pos), &SC::Empty);
        assert_eq!(g.get_content(&stop_pos), &SC::Tile(tile));
    }

    #[test]
    fn game_denies_invalid_move() {
        let mut g = Game::new();
        let start_pos = Coordinate { rank: 1, file: 2 };
        let stop_pos = Coordinate { rank: 8, file: 2 };
        let tile = Tile {
            tile_type: TileType::Two,
            player: Player::Light,
        };
        let board_move = BoardMove::Straight {
            start: start_pos,
            stop: stop_pos,
            tile: tile,
        };
        let result = g.make_move(&board_move);

        assert!(result.is_err());
        assert_eq!(g.get_content(&start_pos), &SC::Tile(tile));
        assert_eq!(
            g.get_content(&stop_pos),
            &SC::Tile(Tile {
                tile_type: TileType::Three,
                player: Player::Dark
            })
        );
    }

    #[test]
    fn initial_gamestate_moves_are_unique() {
        let moves = Game::new().valid_moves();
        let unique_moves: HashSet<BoardMove> = moves.clone().into_iter().collect();

        assert_eq!(moves.len(), unique_moves.len());
    }

    macro_rules! piece_has_n_moves {
        ($($name:ident: $type:expr, $move_num:expr ), *) => {
        $(
            #[test]
            fn $name() {
                let mut game = Game::empty();
                game.board[4][3] = SC::Tile(Tile {
                    tile_type: $type,
                    player: Player::Light,
                });
                let moves = game.valid_moves();

                for move_ in &moves {
                    println!("{}", move_);
                }
                assert_eq!(moves.len(), $move_num);
            }
        )*
        }
    }

    piece_has_n_moves! {
        two_has_twelve_moves: TileType::Two, 12,
        three_has_twenty_moves: TileType::Three, 20,
        four_has_twenty_six_moves: TileType::Four, 26
    }

    #[test]
    fn two_piece_cannot_capture_force_turn() {
        let mut game = Game::empty();
        game.board[4][3] = SC::Tile(Tile {
            tile_type: TileType::Two,
            player: Player::Light,
        });
        game.board[4][1] = SC::Barragoon(BarragoonFace::ForceTurn);

        let moves = game.valid_moves();

        for move_ in &moves {
            if let BoardMove::BarragoonCapture { .. } = move_ {
                assert!(false);
            }
        }
    }

    #[test]
    fn two_piece_and_a_barragoon_have_1003_moves() {
        let mut game = Game::empty();
        game.set_content(
            &Coordinate::new(4, 3),
            &SquareContent::Tile(Tile {
                tile_type: TileType::Two,
                player: Player::Light,
            }),
        );
        game.set_content(&Coordinate::new(2, 3), &SquareContent::Barragoon(BarragoonFace::Blocking));

        let valid_moves = game.valid_moves();
        let unique_valid_moves: HashSet<BoardMove> = valid_moves.into_iter().collect();

        assert_eq!(unique_valid_moves.len(), 7 + 4 + 62 * 16);
    }

    #[test]
    fn three_piece_can_capture_force_turn() {
        let mut game = Game::empty();

        game.board[4][3] = SC::Tile(Tile {
            tile_type: TileType::Three,
            player: Player::Light,
        });

        for [file_idx, rank_idx] in [[4, 0], [3, 1], [2, 2], [1, 3]] {
            game.board[file_idx][rank_idx] = SC::Barragoon(BarragoonFace::ForceTurn);

            let moves = game.valid_moves();

            let mut found_capture = false;

            for move_ in &moves {
                if let BoardMove::BarragoonCapture { .. } = move_ {
                    found_capture = true;
                }
            }

            assert!(found_capture);
        }
    }
}
