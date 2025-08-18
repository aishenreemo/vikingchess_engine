use std::ops::Deref;
use std::ops::Index;
use std::ops::IndexMut;

use rand::RngCore;
use rand::rng;

use crate::bitboard::Bitboard;
use crate::piece::Piece;
use crate::square::Square;

pub struct ZobristTable([u64; ZobristTable::TABLE_LENGTH]);

impl ZobristTable {
    pub const TABLE_LENGTH: usize = Bitboard::TOTAL_SQUARES * Piece::Length as usize;

    pub fn new() -> Self {
        let mut keys = [0u64; Self::TABLE_LENGTH];
        let mut r = rng();
        for key in keys.iter_mut().take(Self::TABLE_LENGTH) {
            *key = r.next_u64();
        }

        Self(keys)
    }
}

impl Deref for ZobristTable {
    type Target = [u64; ZobristTable::TABLE_LENGTH];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IndexMut<(Piece, Square)> for ZobristTable {
    fn index_mut(&mut self, index: (Piece, Square)) -> &mut Self::Output {
        let piece = index.0 as usize;
        let square = index.1.row as usize * Bitboard::BOARD_LENGTH + index.1.col as usize;
        &mut self.0[piece * Bitboard::TOTAL_SQUARES + square]
    }
}

impl Index<(Piece, Square)> for ZobristTable {
    type Output = u64;

    fn index(&self, index: (Piece, Square)) -> &Self::Output {
        let piece = index.0 as usize;
        let square = index.1.row as usize * Bitboard::BOARD_LENGTH + index.1.col as usize;
        &self.0[piece * Bitboard::TOTAL_SQUARES + square]
    }
}
