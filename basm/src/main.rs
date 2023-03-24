//! A super simple assembly like language that compiles to brainfuck.
//!
//! This was mainly written to learn a bit about winnow.

mod parser;

use std::{
    cmp,
    io::{self, Read, Write},
    process::ExitCode,
};

use parser::Instruction;

/// Instruction to clear a register
const CLEAR_REG: &[u8] = b"[-]";

fn main() -> ExitCode {
    let mut stdin = io::stdin().lock();
    let mut s = String::new();
    if let Err(e) = stdin.read_to_string(&mut s) {
        eprintln!("Error reading from stdin: {}", e);
        return ExitCode::from(101u8);
    }

    let instructions = match parser::parse(&s) {
        Ok(instructions) => instructions,
        Err(e) => {
            eprintln!("Error Parsing input: {}", e);
            return ExitCode::from(102u8);
        }
    };

    let mut stdout = io::stdout().lock();
    if let Err(e) = compile(&mut stdout, &instructions) {
        eprintln!("Error writing to stdout: {}", e);
        ExitCode::from(103u8)
    } else {
        ExitCode::SUCCESS
    }
}

/// Compile a list of instructions to brainfuck
fn compile<W: Write>(writer: &mut W, instructions: &[Instruction]) -> io::Result<()> {
    for instruction in instructions {
        write_instruction(writer, instruction)?;
        writer.write_all(b"\n")?;
    }
    Ok(())
}

/// Compile a single instruction to brainfuck
fn write_instruction<W: Write>(writer: &mut W, instruction: &Instruction) -> io::Result<()> {
    match *instruction {
        Instruction::Print => {
            writer.write_all(b".")?;
        }
        Instruction::Input => {
            writer.write_all(b",")?;
        }
        Instruction::Write(n) => {
            writer.write_all(b"[-]")?;
            for _ in 0..n {
                writer.write_all(b"+")?;
            }
        }
        Instruction::Move(n) => {
            let sign = if n < 0i32 { b"<" } else { b">" };

            write_multiple(writer, sign.as_slice(), n.abs())?;
        }
        Instruction::MoveValue(n) => {
            let (sign_fwd, sign_back) = if n > 0i32 { (b">", b"<") } else { (b"<", b">") };
            let n = n.abs();

            // set other cell zero
            write_multiple(writer, sign_fwd, n)?;
            writer.write_all(CLEAR_REG)?;
            write_multiple(writer, sign_back, n)?;

            write_move_val_loop(writer, n, sign_fwd, sign_back)?;
        }
        Instruction::CopyValue(to, tmp) => {
            // This works pretty much as:
            // `to` = 0; `tmp` = 0; move(`original`, both(`tmp`, `to`)); move(`tmp`, `original`);

            // Move to the to register and clear it
            write_move(writer, to)?;
            writer.write_all(CLEAR_REG)?;
            // Move to the `tmp` register from the `to` register and clear it
            write_move(writer, tmp.wrapping_sub(to))?;
            writer.write_all(CLEAR_REG)?;
            // Move back to the `original` register
            write_move(writer, tmp.wrapping_neg())?;
            // start loop
            writer.write_all(b"[-")?;
            // Increment the `to` register by one
            write_move(writer, to)?;
            writer.write_all(b"+")?;
            // Increment the `tmp` register by one
            write_move(writer, tmp.wrapping_sub(to))?;
            writer.write_all(b"+")?;
            // Go back to the `original` register
            write_move(writer, tmp.wrapping_neg())?;
            // End the loop
            writer.write_all(b"]")?;
            // Move the value from the `tmp` register back to the `original` register
            write_move(writer, tmp)?;
            write_move_val_loop(
                writer,
                tmp.abs(),
                move_sign_of(tmp.wrapping_neg()),
                move_sign_of(tmp),
            )?;
            write_move(writer, tmp.wrapping_neg())?;
        }
    }

    Ok(())
}

/// Write a movement of `moven` places negative means move to the left and positive to the right
fn write_move<W: Write>(writer: &mut W, moven: i32) -> io::Result<()> {
    match moven.cmp(&0i32) {
        cmp::Ordering::Equal => Ok(()),
        cmp::Ordering::Greater => write_multiple(writer, b">", moven),
        cmp::Ordering::Less => write_multiple(writer, b"<", moven.abs()),
    }
}

/// Write a loop to move a value over to another register
fn write_move_val_loop<W: Write>(
    writer: &mut W,
    dist: i32,
    sign_fwd: &[u8],
    sign_back: &[u8],
) -> io::Result<()> {
    // start loop
    writer.write_all(b"[-")?;
    // carry over a one to the other cell
    write_multiple(writer, sign_fwd, dist)?;
    writer.write_all(b"+")?;
    write_multiple(writer, sign_back, dist)?;
    // end the loop
    writer.write_all(b"]")
}

/// Get the correct movement sign `>` for positive and `<` for negative
fn move_sign_of(n: i32) -> &'static [u8; 1] {
    if n >= 0i32 {
        b">"
    } else {
        b"<"
    }
}

/// Write the given bytes multiple times
fn write_multiple<W: Write>(writer: &mut W, bytes: &[u8], n: i32) -> io::Result<()> {
    for _ in 0i32..n {
        writer.write_all(bytes)?;
    }

    Ok(())
}
