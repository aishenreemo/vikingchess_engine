use crate::mask::Mask;
use crate::piece::Piece;
use crate::prelude::Bitboard;
use crate::square::Square;

#[derive(Clone, Copy)]
pub struct Action {
    pub piece: Piece,
    pub from: Square,
    pub to: Square,
}

impl Action {
    pub fn new(piece: Piece, from: Square, to: Square) -> Self {
        Self {
            piece,
            from,
            to,
        }
    }

    pub fn valid(&self, bitboard: &Bitboard) -> bool {
        bitboard[self.piece] & self.from.mask() > Mask(0)
    }
    
    pub fn turn_valid(&self, turn_mask: Mask) -> bool {
        self.from.mask() & turn_mask > Mask(0)
    }
}
