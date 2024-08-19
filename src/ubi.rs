use std::{
    future::Ready,
    io::{self, Read},
    str::SplitWhitespace,
};

use crate::{FenError, Game};

struct UbiHandler {
    state: UbiState,

    game: Game,
}

#[derive(Debug, PartialEq, Clone)]
enum UbiState {
    Unitialized,
    WaitingForReady,
    Ready,
    PositionSet,
}

impl UbiHandler {
    pub fn new() -> Self {
        UbiHandler {
            state: UbiState::Unitialized,
            game: Game::empty(),
        }
    }

    pub fn ubi(&mut self) {
        match self.state {
            UbiState::Unitialized => {
                println!("id name {} author {}", "Barragoon-Engine v0.1", "Moraxno");
                println!("ubiok");

                self.state = UbiState::WaitingForReady;
            }
            _ => (),
        }
    }

    pub fn isready(&mut self) {
        match self.state {
            UbiState::Unitialized => (),
            UbiState::WaitingForReady =>
            /* initialize self ... */ /* after that ... */
            {
                println!("readyok");
                self.state = UbiState::Ready;
            }
            _ => println!("readyok"),
        }
    }

    pub fn position(&mut self, mut args: SplitWhitespace) -> Result<(), FenError> {
        let start_position_mode = args.next();

        match start_position_mode {
            Some("startpos") => self.game = Game::new(),
            Some("fen") => self.game = Game::from_fen(self.collect_residual_fen_args(&mut args).as_str())?,
            _ => (),
        }

        println!("{}", self.game);

        self.state = UbiState::PositionSet;
        Ok(())
    }

    fn collect_residual_fen_args(&self, residual_args: &mut SplitWhitespace) -> String {
        let mut fen_string = String::new();

        let i: usize;
        for arg in residual_args {
            if arg != "moves" {
                fen_string.push_str(arg)
            } else {
                break;
            }
        }

        fen_string
    }
}

pub fn ubi_loop<T>(&mut buffer: T) -> io::Result<()>
where
    T: Read,
{
    println!("Barragoon Engine v0.1");
    println!("Now listening for UBI commands ...");

    let mut handler = UbiHandler::new();

    let mut input_buffer = String::new();

    loop {
        input_buffer.clear();
        buffer.read_line(&mut input_buffer)?;
        let input = input_buffer.trim_end();

        let mut args = input.split_whitespace();
        let cmd_maybe = args.next();

        if let Some(cmd) = cmd_maybe {
            match cmd {
                "ubi" => handler.ubi(),
                "isready" => handler.isready(),
                "position" => handler.position(args).unwrap(),
                "exit" => std::process::exit(0),
                _ => println!("Unknown command \"{}\"", cmd),
            }
        }
    }
}

#[cfg(tests)]
mod tests {
    #[test]
    pub fn detect_ubi() {}
}
