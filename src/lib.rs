use std::error::Error;

pub type VikingChessError = Box<dyn Error>;
pub type VikingChessResult<T> = Result<T, VikingChessError>;

mod bitboard;
mod board;
mod piece;
mod square;
mod zobrist;

#[cfg(test)]
mod tests;

pub mod prelude {
    pub use crate::bitboard::Bitboard;
    pub use crate::bitboard::Mask;
    pub use crate::board::Board;
    pub use crate::piece::Piece;
    pub use crate::square::Square;
}
