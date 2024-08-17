use strum::IntoEnumIterator;

use navigation::Coordinate;

use crate::tiles::TileType;
use crate::navigation::Orientation;

pub mod tiles;
pub mod navigation;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Player {
    White,
    Brown,
}


#[derive(Debug, Copy, Clone, PartialEq)]
enum BarragoonAlignment {
    Horizontal,
    Vertical,
}
#[derive(Debug, Copy, Clone, PartialEq)]
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

    pub fn can_be_captured_by(&self, tile_type: TileType) -> bool{
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

        return match self {
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
        };
    }
}

struct Square<'a> {
    coordinate: Coordinate,
    content: &'a SquareContent
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum SquareContent {
    Empty,
    Tile(TileType, Player),
    Barragoon(BarragoonFace),
}

impl SquareContent {
    pub fn to_fen_char(&self) -> char {
        return match self {
            SC::Empty => ' ',
            SC::Tile(TileType::Two, Player::White) => 'Z',
            SC::Tile(TileType::Two, Player::Brown) => 'z',
            SC::Tile(TileType::Three, Player::White) => 'D',
            SC::Tile(TileType::Three, Player::Brown) => 'd',
            SC::Tile(TileType::Four, Player::White) => 'V',
            SC::Tile(TileType::Four, Player::Brown) => 'v',
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
        };
    }
}

const BOARD_WIDTH: i8 = 7;
const BOARD_HEIGHT: i8 = 9;

type SC = SquareContent;

#[derive(Debug, Copy, Clone)]
struct Game {
    board: [[SC; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
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
    JumpCol(i8),
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

    ifile: usize,
    irank: usize,
}

impl<'a> Iterator for SquareIterator<'a> {
    type Item = Square<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ifile >= BOARD_WIDTH as usize {
            self.irank += 1;
        }

        if self.irank >= BOARD_HEIGHT as usize {
            None
        } else {
            Some(
                Square {
                coordinate: Coordinate::new(self.irank as i8, self.ifile as i8),
                content: &self.owner_game.board[self.irank][self.ifile]
            })
            
        }
    }
}

impl Game {
    pub fn new() -> Game {
        Game::from_fen("1vd1dv1/2zdz2/7/1x3x1/x1x1x1x/1x3x1/7/2ZDZ2/1VD1DV1").unwrap()
    }

    pub fn squares<'a>(&'a self) -> SquareIterator<'a> {
        SquareIterator { owner_game: &self, ifile: 0, irank: 0 }
    }

    pub fn from_fen(fen: &str) -> Result<Game, FenError> {
        let mut board: [[SC; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize] = [[SC::Empty; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize];

        let mut row_ptr: i8 = BOARD_HEIGHT - 1;
        let mut col_ptr: i8 = 0;

        for (index, c) in fen.char_indices() {
            let obj: FenParseObject = match c {
                'Z' => FPO::Square(SC::Tile(TileType::Two, Player::White)),
                'z' => FPO::Square(SC::Tile(TileType::Two, Player::Brown)),
                'D' => FPO::Square(SC::Tile(TileType::Three, Player::White)),
                'd' => FPO::Square(SC::Tile(TileType::Three, Player::Brown)),
                'V' => FPO::Square(SC::Tile(TileType::Four, Player::White)),
                'v' => FPO::Square(SC::Tile(TileType::Four, Player::Brown)),
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
                '1'..='7' => FPO::JumpCol(c.to_digit(10).map(|d| d as i8).unwrap()),
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

        return Ok(Game { board: board });
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
            if let SC::Tile(tile, moving_piece_player) = square.content {
                let full_strides = tile.full_strides();

                for full_stride in full_strides {
                    for ifullstep in 0..tile.full_stride_length() - 1 {
                        let (new_orientation, coord_offset, is_last_step) = full_stride.step(ifullstep as u8);
                        let new_coordinate = square.coordinate + coord_offset;
                        if new_coordinate.rank >= BOARD_HEIGHT || new_coordinate.file >= BOARD_WIDTH || new_coordinate.rank < 0 || new_coordinate.file < 0 {
                            //todo(robo) maybe breaking here is fine ... please test this later
                            continue 
                        }

                        match square.content {
                            SC::Tile(_, colliding_piece_player) => {
                                if moving_piece_player == colliding_piece_player || !is_last_step {
                                    break
                                }
    
                                moves.push(Move::TileCapture { start: square.coordinate, stop: new_coordinate });
                            },
                            SC::Empty => {
                                if is_last_step {
                                    moves.push(Move::Straight { start: square.coordinate, stop: new_coordinate });
                                }
                            },
                            SC::Barragoon(face) => {
                                if is_last_step && face.can_be_captured_by(*tile) && face.can_be_captured_from(&new_orientation) {
                                    moves.push(Move::BarragoonCapture { start: square.coordinate, stop: new_coordinate });
                                } else {
                                    // todo! further
                                }
                            }
                        }
                        if let SC::Tile(_, colliding_piece_player) = square.content {
                            // cannot collide in anyway with own pieces during move
                            
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
        write!(f, "+\n")?;

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
            write!(f, "+\n")?;
        }

        write!(f, "  ")?;
        for ifile in 0..BOARD_WIDTH as usize {
            f.write_fmt(format_args!("  {} ", FILES_NAMES[ifile]))?;
        }

        write!(f, "")
    }
}


const RANK_NAMES: [char; BOARD_HEIGHT as usize] = ['1', '2', '3', '4', '5', '6', '7', '8', '9'];
const FILES_NAMES: [char; BOARD_WIDTH as usize] = ['a', 'b', 'c', 'd', 'e', 'f', 'g'];

impl std::fmt::Display for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}{}",
            RANK_NAMES[self.rank as usize], FILES_NAMES[self.file as usize]
        ))
    }
}

#[derive(Debug, Copy, Clone)]
enum Move {
    Straight {
        start: Coordinate,
        stop: Coordinate,
    },
    TileCapture {
        start: Coordinate,
        stop: Coordinate,
    },
    BarragoonCapture {
        start: Coordinate,
        stop: Coordinate,
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Move::Straight { start, stop } = self {
            f.write_fmt(format_args!("{}{}", start, stop))?;
        } else if let Move::TileCapture { start, stop} = self {
            f.write_fmt(format_args!("{}x{}", start, stop))?;
        }

        write!(f, "")
    }
}

fn main() {
    println!("Hello, world!");

    let game = Game::new();
    println!("{:?}", game);
    println!("{}", game);
    println!("1vd1dv1/2zdz2/7/1x3x1/x1x1x1x/1x3x1/7/2ZDZ2/1VD1DV1");
    println!("{}", game.to_fen());

    for tile_move in game.valid_moves() {
        println!("{}", tile_move)
    }

    println!("{:?}", TileType::Three.full_strides());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiles_move_strides() {
        assert_eq!(TileType::Two.full_strides().len(), 3 * 4);
        assert_eq!(TileType::Three.full_strides().len(), 5 * 4);
        assert_eq!(TileType::Four.full_strides().len(), 7 * 4);
    }
}
