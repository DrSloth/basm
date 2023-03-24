//! A very simple brainfuck VM.
//!
//! This doesn't read the programm in whole but uses buffered io and a simple look back buffer
//! which is filled to handle loops.

use std::{
    env,
    fs::File,
    io::{self, BufReader, Read},
    process::ExitCode,
};

fn main() -> ExitCode {
    if let Some(fpath) = env::args().nth(1) {
        let f = File::open(fpath);
        let res = match f {
            Ok(f) => interpret(&mut BufReader::new(f)),
            Err(e) => Err(e),
        };

        match res {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("Error: {}", e);
                ExitCode::FAILURE
            }
        }
    } else {
        eprintln!("Please specify the file to read");
        ExitCode::FAILURE
    }
}

/// A brainfuck vm holding all the state needed to run a brainfuck program
#[derive(Debug)]
struct BfVm {
    /// A stack with jump back addresses inside `buf`
    jump_stack: Vec<usize>,
    /// A buffer which is filled to look back in the programm with
    buf: Vec<u8>,
    /// The board which actually stores the programms memory
    board: Vec<u32>,
    /// The current cell the program is accessing
    cur_cell_idx: usize,
    /// Current position in the look back buffer
    cur_buf_idx: usize,
}

impl BfVm {
    /// Get a mutable reference to the current memory cell
    fn cur_cell_mut(&mut self) -> &mut u32 {
        self.board
            .get_mut(self.cur_cell_idx)
            .unwrap_or_else(|| unreachable!("cur_cell should always be inside the board"))
    }
}

/// Interpret a Brainfuck program read from the reader
fn interpret<R: Read>(reader: &mut R) -> io::Result<()> {
    let mut vm = BfVm {
        jump_stack: Vec::with_capacity(5),
        buf: Vec::with_capacity(256),
        board: vec![0; 32],
        cur_cell_idx: 0,
        cur_buf_idx: 0,
    };

    let mut stdin = io::stdin().lock();

    'outer: loop {
        let Some(b) = retrieve_byte(&mut vm, reader)? else {
            return Ok(());
        };

        // eprintln!("b: {}\nvm: {:?}", b, vm);
        match b {
            b'+' => {
                let cell = vm.cur_cell_mut();
                let val = *cell;
                *cell = val.saturating_add(1);
            }
            b'-' => {
                let cell = vm.cur_cell_mut();
                let val = *cell;
                *cell = val.saturating_sub(1);
            }
            b'>' => {
                // at last cell of board
                if vm.cur_cell_idx == vm.board.len() {
                    vm.board.push(0);
                }

                vm.cur_cell_idx = vm.cur_cell_idx.wrapping_add(1);
            }
            b'<' => {
                // we just stop at the 0 cell
                vm.cur_cell_idx = vm.cur_cell_idx.saturating_sub(1);
            }
            b'.' => {
                // println!("{}", *vm.cur_cell_mut());
                if let Some(val) = char::from_u32(*vm.cur_cell_mut()) {
                    print!("{}", val);
                } else {
                    print!("r({})", *vm.cur_cell_mut());
                }
            }
            b',' => {
                let mut buf = [0u8; 1];
                stdin.read_exact(&mut buf)?;
                *vm.cur_cell_mut() = u32::from(buf[0]);
            }
            b'[' => {
                if *vm.cur_cell_mut() == 0 {
                    loop {
                        let Some(b) = retrieve_byte(&mut vm, reader)? else {
                            return Ok(());
                        };

                        if b == b']' {
                            continue 'outer;
                        }
                    }
                } else {
                    vm.jump_stack.push(vm.cur_buf_idx);
                }
            }
            b']' => {
                if *vm.cur_cell_mut() == 0 {
                    vm.jump_stack.pop();
                    if vm.jump_stack.is_empty() {
                        vm.buf.clear();
                        vm.cur_buf_idx = 0;
                    }
                } else if let Some(n) = vm.jump_stack.pop() {
                    if vm.cur_buf_idx >= vm.buf.len() {
                        vm.buf.push(b']');
                    }
                    vm.cur_buf_idx = n;
                    continue;
                }
            }
            _ => (),
        }

        if !vm.jump_stack.is_empty() {
            if vm.cur_buf_idx >= vm.buf.len() {
                vm.buf.push(b);
            }
            vm.cur_buf_idx = vm.cur_buf_idx.wrapping_add(1);
        }
    }
}

/// Read a byte either from the vms look back buffer or pull a new one from the reader
fn retrieve_byte<R: Read>(vm: &mut BfVm, reader: &mut R) -> Result<Option<u8>, io::Error> {
    let mut buf = [0u8; 1];
    let b = if let Some(b) = vm.buf.get(vm.cur_buf_idx) {
        *b
    } else {
        if reader.read(&mut buf)? == 0 {
            return Ok(None);
        }
        buf[0]
    };

    Ok(Some(b))
}
