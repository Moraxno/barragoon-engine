enum PieceType {
    Two,
    Three,
    Four
}

enum Player {
    White,
    Brown
}

enum BarragoonFace {
    Blocking,
    Straight,
    OneWay,
    OneWayTurn,
    ForceTurn
}


enum SpaceInhabitant {
    Nothing,
    Piece(PieceType, Player)
}

struct Game {
    
}


fn main() {
    println!("Hello, world!");
}
