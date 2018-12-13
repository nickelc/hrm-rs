use std::fmt;

use crate::machine::Tile;

pub type Result<T> = std::result::Result<T, RuntimeError>;

#[derive(Debug)]
pub enum RuntimeError {
    BadTileAddress(Tile),
    EmptyHands,
    EmptyTile(Tile),
    InvalidOperation(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::RuntimeError::*;
        match *self {
            BadTileAddress(tile) => write!(f, "Bad tile address: {}", tile),
            EmptyHands => write!(f, "Empty hands"),
            EmptyTile(tile) => write!(f, "Empty tile: {}", tile),
            InvalidOperation(ref msg) => msg.fmt(f),
        }
    }
}

pub fn bad_address<T>(tile: Tile) -> Result<T> {
    Err(RuntimeError::BadTileAddress(tile))
}

pub fn empty_tile<T>(tile: Tile) -> Result<T> {
    Err(RuntimeError::EmptyTile(tile))
}

pub fn invalid_op<T>(msg: &str) -> Result<T> {
    Err(RuntimeError::InvalidOperation(msg.to_string()))
}
