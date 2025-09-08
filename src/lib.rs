#![feature(iter_array_chunks)]

use std::error::Error;

pub type VikingChessError = Box<dyn Error>;
pub type VikingChessResult<T> = Result<T, VikingChessError>;

mod action;
mod bitboard;
mod board;
mod magics;
mod mask;
mod piece;
mod square;
mod state;
mod zobrist;

#[cfg(test)]
mod tests;

pub mod prelude {
    pub use crate::bitboard::Bitboard;
    pub use crate::board::Board;
    pub use crate::magics::MagicTable;
    pub use crate::mask::Mask;
    pub use crate::piece::Piece;
    pub use crate::square::Square;
    pub use crate::action::Action;
}
