#[derive(Debug,Copy,Clone)]
enum TileType {
    Two,
    Three,
    Four
}
#[derive(Debug,Copy,Clone)]
enum Player {
    White,
    Brown
}
#[derive(Debug,Copy,Clone)]
enum BarragoonOrientation {
    North,
    West,
    South,
    East
}
#[derive(Debug,Copy,Clone)]
enum BarragoonAlignment {
    Horizontal,
    Vertical
}
#[derive(Debug,Copy,Clone)]
enum BarragoonFace {
    Blocking,
    Straight {alignment: BarragoonAlignment},
    OneWay {orientation: BarragoonOrientation},
    OneWayTurnLeft {orientation: BarragoonOrientation},
    OneWayTurnRight {orientation: BarragoonOrientation},
    ForceTurn
}

#[derive(Debug,Copy,Clone)]
enum SquareContent {
    Empty,
    Tile(TileType, Player),
    Barragoon(BarragoonFace)
}


type SC = SquareContent;

#[derive(Debug,Copy,Clone)]
struct Game {
    board: [[SC; 7]; 9]
}

#[derive(Debug,Copy,Clone)]
enum FenError {
    UnderfullLine {char_index: usize},
    OverfullLine{char_index: usize},
    TooManyLines{char_index: usize},
    InvalidChar{char_index: usize},
}

#[derive(Debug,Copy,Clone)]
enum FenParseObject {
    JumpCol(u32),
    SkipRow,
    Square(SquareContent),
    InvalidChar
}


impl Game {
    pub fn new(fen: &str) -> Result<Game, FenError> {
        let mut board: [[SC; 7]; 9] = [[SC::Empty; 7]; 9];

        let mut row_ptr: u32 = 0;
        let mut col_ptr: u32 = 0;

        for (index, c) in fen.char_indices() {
            let obj: FenParseObject = match c {
                'Z'       => FenParseObject::Square(SC::Tile(TileType::Two, Player::White)),
                'z'       => FenParseObject::Square(SC::Tile(TileType::Two, Player::Brown)),
                'D'       => FenParseObject::Square(SC::Tile(TileType::Three, Player::White)),
                'd'       => FenParseObject::Square(SC::Tile(TileType::Three, Player::Brown)),
                'V'       => FenParseObject::Square(SC::Tile(TileType::Four, Player::White)),
                'v'       => FenParseObject::Square(SC::Tile(TileType::Four, Player::Brown)),
                '1'..='7' => FenParseObject::JumpCol(c.to_digit(10).unwrap()),
                '/'       => FenParseObject::SkipRow,
                '+'       => FenParseObject::Square(SC::Barragoon(BarragoonFace::ForceTurn)),
                '|'       => FenParseObject::Square(SC::Barragoon(BarragoonFace::Straight { alignment: BarragoonAlignment::Vertical })),
                '-'       => FenParseObject::Square(SC::Barragoon(BarragoonFace::Straight { alignment: BarragoonAlignment::Horizontal })),
                'Y'       => FenParseObject::Square(SC::Barragoon(BarragoonFace::OneWay { orientation: BarragoonOrientation::South })),
                '^'       => FenParseObject::Square(SC::Barragoon(BarragoonFace::OneWay { orientation: BarragoonOrientation::North })),
                '<'       => FenParseObject::Square(SC::Barragoon(BarragoonFace::OneWay { orientation: BarragoonOrientation::West })),
                '>'       => FenParseObject::Square(SC::Barragoon(BarragoonFace::OneWay { orientation: BarragoonOrientation::East })),
                'x'       => FenParseObject::Square(SC::Barragoon(BarragoonFace::Blocking)),
                'S'       => FenParseObject::Square(SC::Barragoon(BarragoonFace::OneWayTurnLeft { orientation: BarragoonOrientation::South })),
                'N'       => FenParseObject::Square(SC::Barragoon(BarragoonFace::OneWayTurnLeft { orientation: BarragoonOrientation::North })),
                'E'       => FenParseObject::Square(SC::Barragoon(BarragoonFace::OneWayTurnLeft { orientation: BarragoonOrientation::East })),
                'W'       => FenParseObject::Square(SC::Barragoon(BarragoonFace::OneWayTurnLeft { orientation: BarragoonOrientation::West })),
                's'       => FenParseObject::Square(SC::Barragoon(BarragoonFace::OneWayTurnRight { orientation: BarragoonOrientation::South })),
                'n'       => FenParseObject::Square(SC::Barragoon(BarragoonFace::OneWayTurnRight { orientation: BarragoonOrientation::North })),
                'e'       => FenParseObject::Square(SC::Barragoon(BarragoonFace::OneWayTurnRight { orientation: BarragoonOrientation::East })),
                'w'       => FenParseObject::Square(SC::Barragoon(BarragoonFace::OneWayTurnRight { orientation: BarragoonOrientation::West })),
                _ => FenParseObject::InvalidChar
            };

            match obj { 
                FenParseObject::Square(content) => {
                    board[row_ptr as usize][col_ptr as usize] = content;
                    col_ptr += 1;
                },
                FenParseObject::JumpCol(cols) => {
                    col_ptr += cols;
                    if col_ptr > 7 { 
                        return Result::Err(FenError::OverfullLine{char_index: index}); 
                    }
                },
                FenParseObject::SkipRow => {
                    if col_ptr == 7 {
                        col_ptr = 0;
                        row_ptr += 1;
                    }

                    if row_ptr > 8 {
                        return Result::Err(FenError::TooManyLines{char_index: index}); 
                    }
                },
                FenParseObject::InvalidChar => {
                    return Result::Err(FenError::InvalidChar {char_index: index}); 
                }
            }
        }


        return Ok(Game {
            board: board
        })
    }
}


fn main() {
    println!("Hello, world!");

    println!("{:?}", Game::new("1vd1dv1/2zdz2/7/1x3x1/x1x1x1x/1x3x1/7/2ZDZ2/1VD1DV1"))
}
