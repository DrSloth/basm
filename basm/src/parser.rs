//! Parser for a basm program

use std::fmt::Display;

pub use winnow::error::ErrMode as ParserError;

use winnow::{
    branch::{alt, dispatch},
    bytes::{any, take_until0},
    character::{alpha1, dec_int, dec_uint, multispace0},
    combinator::{fail, success},
    error::{Error as WinnowError, ErrorKind},
    multi::fold_many0,
    prelude::*,
    sequence::delimited,
};

/// A single Instruction that will be compiled to brainfuck
#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    /// Receive input from the user and store it in the current cell (`,`)
    Input,
    /// Print the current cells value (`.`)
    Print,
    /// Write the given value into the current cell after clearing it
    Write(u32),
    /// Move to another cell, move right if positive (`>`) and move left if negative (`<`)
    Move(i32),
    /// Move a value from the current cell to the given offset.
    MoveValue(i32),
    /// Copy a given value
    CopyValue(i32, i32),
}

/// Parse a single instruction
fn parse_instruction(input: &str) -> IResult<&str, Option<Instruction>> {
    dispatch! {parse_word;
        "INPUT" => success(Some(Instruction::Input)),
        "PRINT" => success(Some(Instruction::Print)),
        "WRITE" => parse_u32_param.map(Instruction::Write).map(Some),
        "MOVE" => parse_i32_param.map(Instruction::Move).map(Some),
        "MOVEVAL" => parse_i32_param.map(Instruction::MoveValue).map(Some),
        "COPY" => (
            parse_i32_param,
            delimited(multispace0, ',', multispace0).void(),
            parse_i32_param
        ).map(|(first, _, second)| Some(Instruction::CopyValue(first, second))),
        ";" => take_until0("\n").map(|_| None),
        _ => {
            println!("bad word");
            fail
        }
    }
    .parse_next(input)
}

/// Parses a u32 parameter to an instruction
fn parse_u32_param(input: &str) -> IResult<&str, u32> {
    alt((
        delimited('\'', any, '\'').map(u32::from),
        dec_uint,
        parse_escape,
    ))
    .parse_next(input)
}

/// Parses an i32 parameter to an instruction
fn parse_i32_param(input: &str) -> IResult<&str, i32> {
    dec_int.parse_next(input)
}

/// Parses an escape sequence
fn parse_escape(input: &str) -> IResult<&str, u32> {
    let (rest, c) = delimited("'\\", any, '\'').parse_next(input)?;

    match c {
        'n' => Ok((rest, u32::from(b'\n'))),
        _ => Err(ParserError::Cut(WinnowError::new(input, ErrorKind::Fail))),
    }
}

/// Parses a "word" a control sequence specifying what the data after means.
/// This can either be an instruction or `;` for comments
fn parse_word(input: &str) -> IResult<&str, &str> {
    delimited(multispace0, alt((alpha1, ";")), multispace0).parse_next(input)
}

/// Parse the given basm string
pub fn parse(s: &str) -> Result<Vec<Instruction>, impl Display + '_> {
    // let words = many0(parse_instruction).parse_next(s).unwrap();
    let words = fold_many0(
        parse_instruction,
        Vec::new,
        |mut acc: Vec<Instruction>, item| {
            if let Some(item) = item {
                acc.push(item);
            }

            acc
        },
    )
    .parse_next(s);
    // eprintln!("{:?}", words);
    words.map(|(_, a)| a)
}
