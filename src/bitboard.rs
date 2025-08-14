use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::ops::BitAnd;
use std::ops::BitAndAssign;
use std::ops::BitOr;
use std::ops::BitOrAssign;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Index;
use std::ops::IndexMut;
use std::ops::Not;

use serde::Serialize;
use serde::Serializer;

use crate::VikingChessResult;
use crate::piece::Piece;
use crate::square::Square;

#[derive(Default)]
pub struct Bitboard([Mask; Piece::Length as usize]);

impl Bitboard {
    pub const BOARD_LENGTH: usize = 9;
    pub const TOTAL_SQUARES: usize = Bitboard::BOARD_LENGTH * Bitboard::BOARD_LENGTH;

    pub fn from_fen(str: &'static str) -> VikingChessResult<Self> {
        let mut bitboard = Self::default();
        let mut col = 0;
        let mut row = 0;

        const BOARD_LENGTH: u8 = Bitboard::BOARD_LENGTH as u8;
        for ch in str.chars() {
            if matches!(ch, 'A' | 'D' | 'K') {
                bitboard[Piece::from(ch)] |= Square::try_from((col, row))?.mask();
                col += 1;
            } else if let Some(digit) = ch.to_digit(10) {
                col += digit as u8;
            } else if (ch == '/' && col % BOARD_LENGTH != 0) || col > BOARD_LENGTH {
                return Err(format!("Invalid notation {str}.").into());
            } else if ch == '/' {
                row += 1;
                col = 0;
            }
        }

        Ok(bitboard)
    }

    pub fn iter<'a>(&'a self) -> BitboardIter<'a> {
        BitboardIter::new(&self)
    }

    pub fn all(&self) -> Mask {
        Piece::PIECES.into_iter().map(Piece::from).fold(Mask(0), |a, b| a | self[b])
    }

    pub fn moves(square: Square) -> Mask {
        let square_col_mask = Mask(0x1008040201008040201u128 << square.col);
        let square_row_mask = Mask(0x1ff << (9 * square.row));
        let mask = (square_col_mask | square_row_mask) & !square.mask();

        mask
    }

    pub fn legal_moves(square: Square, blockers: Mask) -> Mask {
        let mut legal_moves = Mask(0);
        let rank = square.row;
        let file = square.col;

        for r in (rank + 1)..9 {
            let current_square = Square::try_from((file, r)).unwrap();
            if (blockers & current_square.mask()).0 != 0 {
                break;
            }

            legal_moves |= current_square.mask();
        }

        if rank > 0 {
            for r in (0..rank).rev() {
                let current_square = Square::try_from((file, r)).unwrap();
                if (blockers & current_square.mask()).0 != 0 {
                    break;
                }

                legal_moves |= current_square.mask();
            }
        }

        for f in (file + 1)..9 {
            let current_square = Square::try_from((f, rank)).unwrap();
            if (blockers & current_square.mask()).0 != 0 {
                break; 
            }

            legal_moves |= current_square.mask();
        }

        if file > 0 {
            for f in (0..file).rev() {
                let current_square = Square::try_from((f, rank)).unwrap();
                if (blockers & current_square.mask()).0 != 0 {
                    break;
                }

                legal_moves |= current_square.mask();
            }
        }

        legal_moves
    }
}

#[derive(Default, Debug, PartialEq, PartialOrd, Eq, Hash, Clone, Copy)]
pub struct Mask(pub u128);

impl Serialize for Mask {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl Deref for Mask {
    type Target = u128;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Mask {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<u128> for Mask {
    fn from(value: u128) -> Self {
        Self(value)
    }
}

impl BitOr for Mask {
    type Output = Mask;

    fn bitor(self, rhs: Self) -> Self::Output {
        Mask(self.0 | rhs.0)
    }
}

impl BitOrAssign for Mask {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl Not for Mask {
    type Output = Self;

    fn not(self) -> Self::Output {
        Mask(!self.0)
    }
}

impl BitAnd for Mask {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Mask(self.0 & rhs.0)
    }
}

impl BitAndAssign for Mask {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
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

            piece = pieces.into_iter().find(|&p| (self.bitboard[p] & square.mask()) > Mask(0));

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
        for i in 0..Bitboard::TOTAL_SQUARES as u128 {
            let col = i % Bitboard::BOARD_LENGTH as u128;
            let ch = match i {
                i if (self[Piece::King].0 >> i) & 1 == 1 => "K",
                i if (self[Piece::Defender].0 >> i) & 1 == 1 => "D",
                i if (self[Piece::Attacker].0 >> i) & 1 == 1 => "A",
                _ => ".",
            };

            write!(f, "{}", ch)?;

            if col + 1 == Bitboard::BOARD_LENGTH as u128 {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}

impl Index<Piece> for Bitboard {
    type Output = Mask;

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

impl Display for Mask {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for i in 0..Bitboard::TOTAL_SQUARES as u128 {
            let col = i % Bitboard::BOARD_LENGTH as u128;

            let mut ch = ".";
            if (self.0 >> i) & 1 == 1 {
                ch = "1";
            }

            write!(f, "{}", ch)?;

            if col + 1 == Bitboard::BOARD_LENGTH as u128 {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}

impl Debug for Bitboard {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Bitboard:\n{}", self)?;

        for piece in Piece::PIECES.into_iter().map(Piece::from) {
            writeln!(f, "Mask {piece:?}:\n{}", self[piece])?;
        }

        Ok(())
    }
}
