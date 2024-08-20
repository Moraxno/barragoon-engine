use std::{
    io::{self, BufRead, Cursor, Read, Write},
    str::SplitWhitespace, sync::mpsc::{Receiver, Sender}, time::Duration,
};
use std::sync::mpsc;


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

    pub fn ubi(&mut self) -> Option<&str> {
        match self.state {
            UbiState::Unitialized => {
                println!("id name {} author {}", "Barragoon-Engine v0.1", "Moraxno");
                println!("ubiok");

                self.state = UbiState::WaitingForReady;

                Some("id name Barragoon-Engine v0.1 author Moraxno\nubiok")
            }
            _ => None,
        }
    }

    pub fn isready(&mut self) -> Option<&str> {
        match self.state {
            UbiState::Unitialized => None,
            UbiState::WaitingForReady =>
            /* initialize self ... */ /* after that ... */
            {
                self.state = UbiState::Ready;
                Some("readyok")
            }
            _ => Some("readyok")
        }
    }

    pub fn position(&mut self, mut args: SplitWhitespace) -> Option<&str> {
        let start_position_mode = args.next();

        match start_position_mode {
            Some("startpos") => self.game = Game::new(),
            Some("fen") => self.game = Game::from_fen(self.collect_residual_fen_args(&mut args).as_str()).unwrap(),
            _ => (),
        }

        // println!("{}", self.game);

        self.state = UbiState::PositionSet;
        None
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

pub fn ubi_loop<S, T>(input: &mut S, output: &mut T) -> io::Result<()>
where
    S: Read + BufRead,
    T: Write,
{
    println!("Barragoon Engine v0.1");
    println!("Now listening for UBI commands ...");

    let mut handler = UbiHandler::new();

    let mut input_buffer = String::new();

    loop {
        input_buffer.clear();
        input.read_line(&mut input_buffer)?;
        let input = input_buffer.trim_end();

        let mut args = input.split_whitespace();
        let cmd_maybe = args.next();

        if let Some(cmd) = cmd_maybe {
            let answer = match cmd {
                "ubi" => handler.ubi(),
                "isready" => handler.isready(),
                "position" => handler.position(args),
                "exit" => std::process::exit(0),
                _ => Some("Unknown command"), ///// \"{}\"", cmd),
            };
            
            if let Some(buffer) = answer {
                output.write(buffer.as_bytes()).unwrap();
            }
        }
    }
}

struct SyncWriter {
    inner: Sender<u8>
}

struct SyncReader {
    inner: Receiver<u8>
}

impl SyncReader {
    pub fn new(recv: Receiver<u8>) -> SyncReader {
        SyncReader { inner: recv }
    }
}

impl SyncWriter {
    pub fn new(send: Sender<u8>) -> SyncWriter {
        SyncWriter { inner: send}
    }
}

impl Write for SyncWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for byte in buf {
            self.inner.send(*byte).expect("Channel dead.");
        }

        io::Result::Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        io::Result::Ok(())
    }
}

impl Read for SyncReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut num_bytes = 0;
        loop {
            let r = self.inner.recv_timeout(Duration::from_millis(20));    
        
            match r {
                Ok(data) => {
                    buf[num_bytes] = data;
                    num_bytes += 1;
                },
                Err(..) => break,
            }
        }

        io::Result::Ok(num_bytes)
    }
}

// // // impl BufRead for SyncReader {
// // //     fn fill_buf(&mut self) -> io::Result<&[u8]> {
// // //         let mut send_byte = true;

// // //         if self.buffer_is_clear {
// // //             let r = self.inner.recv_timeout(Duration::from_millis(20));    
        
// // //             match r {
// // //                 Ok(data) => self.my_buf[0] = data,
// // //                 Err(..) => send_byte = false,
// // //             }
// // //         }

// // //         if send_byte {
// // //             Result::Ok(&self.my_buf)
// // //         } else {
// // //             Result::Ok(&self.my_buf[0..0])
// // //         }

        
// // //     }

// // //     fn consume(&mut self, amt: usize) {
// // //         /* ignore */ 
// // //     }
// // // }

#[cfg(test)]
mod tests {
    use std::{io::{BufRead, BufReader, BufWriter, Cursor, Write}, sync::mpsc, thread, time::Duration};

    use crate::ubi::{SyncReader, SyncWriter};

    use super::ubi_loop;

    #[test]
    pub fn detect_ubi() {

        let (input_tx, input_rx) = mpsc::channel();
        let (output_tx, output_rx) = mpsc::channel();
        

        let mut input_send = SyncWriter::new(input_tx);
        let mut input_recv = BufReader::new(SyncReader::new(input_rx));
        let mut output_send = SyncWriter::new(output_tx);
        let mut output_recv = BufReader::new(SyncReader::new(output_rx));

        let t = thread::spawn(move | | {
            ubi_loop(&mut input_recv, &mut output_send)
        });

        thread::sleep(Duration::from_millis(100));
        writeln!(input_send, "ubi").unwrap();
        thread::sleep(Duration::from_millis(100));

        // discard first line ...
        let mut buf = String::new();
        output_recv.read_line(&mut buf).unwrap();
        buf.clear();
        output_recv.read_line(&mut buf).unwrap();
        assert_eq!(buf, "ubiok");


        // let r = t.join().expect("Thread could not rejoin.").expect("tf");

        // println!("{:?}", output);


    }
}
