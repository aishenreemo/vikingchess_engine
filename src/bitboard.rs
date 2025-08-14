use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::ops::Index;
use std::ops::IndexMut;

use crate::piece::Piece;
use crate::square::Square;

#[derive(Default)]
pub struct Bitboard([u128; Piece::Length as usize]);

impl Bitboard {
    pub const BOARD_LENGTH: usize = 9;
    pub const TOTAL_SQUARES: usize = Bitboard::BOARD_LENGTH * Bitboard::BOARD_LENGTH;

    pub fn iter<'a>(&'a self) -> BitboardIter<'a> {
        BitboardIter::new(&self)
    }

    pub fn all(&self) -> u128 {
        Piece::PIECES.into_iter().map(Piece::from).fold(0, |a, b| a | self[b])
    }
}

pub struct BitboardIter<'a> {
    counter: usize,
    bitboard: &'a Bitboard,
}

impl<'a> BitboardIter<'a> {
    pub fn new(bitboard: &'a Bitboard) -> Self {
        Self { counter: 0, bitboard }
    }
}

impl<'a> Iterator for BitboardIter<'a> {
    type Item = (Piece, Square);

    fn next(&mut self) -> Option<Self::Item> {
        let mut square = Square::try_from(self.counter).ok()?;
        let mut piece = None;

        while piece.is_none() {
            let pieces = Piece::PIECES.map(Piece::from);

            piece = pieces.into_iter().find(|&p| (self.bitboard[p] & square.bit()) > 0);

            if piece.is_none() {
                self.counter += 1;
                square = Square::try_from(self.counter).ok()?;
            }
        }

        self.counter += 1;
        return Some((piece?, square));
    }
}

impl Display for Bitboard {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for i in 0..Bitboard::TOTAL_SQUARES {
            let col = i % Bitboard::BOARD_LENGTH;
            let ch = match i {
                i if (self[Piece::King] >> i) & 1 == 1 => "K",
                i if (self[Piece::Defender] >> i) & 1 == 1 => "D",
                i if (self[Piece::Attacker] >> i) & 1 == 1 => "A",
                _ => ".",
            };

            write!(f, "{}", ch)?;

            if col + 1 == Bitboard::BOARD_LENGTH {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}

impl Index<Piece> for Bitboard {
    type Output = u128;

    fn index(&self, index: Piece) -> &Self::Output {
        if index >= Piece::Length {
            panic!("Cannot index {index:?} to the bitboard.");
        }
        &self.0[index as usize]
    }
}

impl IndexMut<Piece> for Bitboard {
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        if index >= Piece::Length {
            panic!("Cannot index mut {index:?} to the bitboard.");
        }
        &mut self.0[index as usize]
    }
}
