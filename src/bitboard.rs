use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::ops::Index;
use std::ops::IndexMut;

use crate::mask::Mask;
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
        BitboardIter::new(self)
    }

    pub fn all(&self) -> Mask {
        Piece::PIECES.into_iter().map(Piece::from).fold(Mask(0), |a, b| a | self[b])
    }

    pub fn moves(square: Square) -> Mask {
        let square_col_mask = Mask(0x1008040201008040201u128 << square.col);
        let square_row_mask = Mask(0x1ff << (9 * square.row));

        (square_col_mask | square_row_mask) & !square.mask()
    }

    pub fn blockers(square: Square) -> Mask {
        const COLUMNS: u128 = 0x1008040201008040201u128;
        const ROWS: u128 = 0x1ff;
        let cols = Mask(COLUMNS) | Mask(COLUMNS << 8);
        let rows = Mask(ROWS) | Mask(ROWS << (9 * 8));
        let corners = (1 << 0) | (1 << 8) | (1 << 72) | (1 << 80);
        let mut potential_blockers = Self::moves(square) & !(cols | rows);

        match (square.col, square.row) {
            (0 | 8, 0 | 8) => {
                potential_blockers |= Mask(COLUMNS & !corners);
                potential_blockers |= Mask(ROWS & !corners);
            },
            (0 | 8, _) => {
                potential_blockers |= Mask(COLUMNS & !corners & !square.mask().0);
            },
            (_, 0 | 8) => {
                potential_blockers |= Mask(ROWS & !corners & !square.mask().0);
            },
            _ => {},
        }

        potential_blockers
    }

    pub fn legal_moves(square: Square, blockers: Mask) -> Mask {
        let mut legal_moves = Mask(0);
        let rank = square.row;
        let file = square.col;

        let rank_into_square = |r: u8| Square::try_from((file, r)).unwrap();
        let file_into_square = |f: u8| Square::try_from((f, rank)).unwrap();
        let predicate = |s: &Square| (blockers & s.mask()).0 == 0;
        let fold = |a: Mask, b: Square| a | b.mask();

        legal_moves |= ((rank + 1)..9).map(rank_into_square).take_while(predicate).fold(Mask(0), fold);
        legal_moves |= (0..rank).rev().map(rank_into_square).take_while(predicate).fold(Mask(0), fold);
        legal_moves |= ((file + 1)..9).map(file_into_square).take_while(predicate).fold(Mask(0), fold);
        legal_moves |= (0..file).rev().map(file_into_square).take_while(predicate).fold(Mask(0), fold);

        legal_moves
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
        Some((piece?, square))
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

            write!(f, "{ch}")?;

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

            write!(f, "{ch}")?;

            if col + 1 == Bitboard::BOARD_LENGTH as u128 {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}

impl Debug for Bitboard {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Bitboard:\n{self}")?;

        for piece in Piece::PIECES.into_iter().map(Piece::from) {
            writeln!(f, "Mask {piece:?}:\n{}", self[piece])?;
        }

        Ok(())
    }
}
