#[derive(Debug, Copy, Clone)]
enum TileType {
    Two,
    Three,
    Four,
}
#[derive(Debug, Copy, Clone)]
enum Player {
    White,
    Brown,
}
#[derive(Debug, Copy, Clone)]
enum BarragoonOrientation {
    North,
    West,
    South,
    East,
}
#[derive(Debug, Copy, Clone)]
enum BarragoonAlignment {
    Horizontal,
    Vertical,
}
#[derive(Debug, Copy, Clone)]
enum BarragoonFace {
    Blocking,
    Straight { alignment: BarragoonAlignment },
    OneWay { orientation: BarragoonOrientation },
    OneWayTurnLeft { orientation: BarragoonOrientation },
    OneWayTurnRight { orientation: BarragoonOrientation },
    ForceTurn,
}

#[derive(Debug, Copy, Clone)]
enum SquareContent {
    Empty,
    Tile(TileType, Player),
    Barragoon(BarragoonFace),
}

type SC = SquareContent;

#[derive(Debug, Copy, Clone)]
struct Game {
    board: [[SC; 7]; 9],
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
    JumpCol(u32),
    SkipRow,
    Square(SquareContent),
    InvalidChar,
}
type FPO = FenParseObject;
type BA = BarragoonAlignment;
type BO = BarragoonOrientation;
type BF = BarragoonFace;

impl Game {
    pub fn new(fen: &str) -> Result<Game, FenError> {
        let mut board: [[SC; 7]; 9] = [[SC::Empty; 7]; 9];

        let mut row_ptr: u32 = 0;
        let mut col_ptr: u32 = 0;

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
                '1'..='7' => FPO::JumpCol(c.to_digit(10).unwrap()),
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
                    if col_ptr > 7 {
                        return Result::Err(FenError::OverfullLine { char_index: index });
                    }
                }
                FPO::SkipRow => {
                    if col_ptr == 7 {
                        col_ptr = 0;
                        row_ptr += 1;
                    } else {
                        return Result::Err(FenError::UnderfullLine { char_index: index });
                    }

                    if row_ptr > 8 {
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
}

fn main() {
    println!("Hello, world!");

    println!("{:?}", Game::new("1vd1dv1/2zdz2/7/1x3x1/x1x1x1x/1x3x1/7/2ZDZ2/1VD1DV1"))
}
