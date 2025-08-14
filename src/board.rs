use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use crate::VikingChessResult;
use crate::bitboard::Bitboard;
use crate::bitboard::BitboardIter;
use crate::piece::Piece;
use crate::square::Square;
use crate::zobrist::ZobristTable;

pub struct Board {
    bitboard: Bitboard,
    zobrist_table: ZobristTable,
    pub zobrist_hash: u64,
}

impl Default for Board {
    fn default() -> Self {
        Self::from_fen(Self::STARTING_FEN).expect("Invalid starting FEN.")
    }
}

impl Board {
    pub const STARTING_FEN: &'static str = "3AAA3/4A4/4D4/A3D3A/AADDKDDAA/A3D3A/4D4/4A4/3AAA3 B";
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_fen(str: &'static str) -> VikingChessResult<Self> {
        let mut bitboard = Bitboard::default();
        let zobrist_table = ZobristTable::new();

        let mut col = 0;
        let mut row = 0;
        const BOARD_LENGTH: u8 = Bitboard::BOARD_LENGTH as u8;
        for ch in str.chars() {
            if matches!(ch, 'A' | 'D' | 'K') {
                bitboard[Piece::from(ch)] |= Square::try_from((col, row))?.bit();
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

        let initial_hash = Board::calculate_hash(&bitboard, &zobrist_table);

        Ok(Self {
            bitboard,
            zobrist_table,
            zobrist_hash: initial_hash,
        })
    }

    pub fn iter_bitboard<'a>(&'a self) -> BitboardIter<'a> {
        self.bitboard.iter()
    }

    fn calculate_hash(bitboard: &Bitboard, zobrist_table: &ZobristTable) -> u64 {
        let mut hash = 0;
        for (piece, square) in bitboard.iter() {
            hash ^= zobrist_table[(piece, square)];
        }

        hash
    }

    pub fn move_piece(&mut self, piece: Piece, start_square: Square, end_square: Square) -> VikingChessResult<()> {
        if self.bitboard[piece] & start_square.bit() <= 0 {
            panic!("There is no {piece:?} in start_square {start_square:?}");
        }

        if self.bitboard.all() & end_square.bit() > 0 {
            return Err(format!("There is already a {piece:?} in end_square {end_square:?}").into());
        }

        self.bitboard[piece] &= !(start_square.bit());
        self.bitboard[piece] |= end_square.bit();

        self.zobrist_hash ^= self.zobrist_table[(piece, start_square)];
        self.zobrist_hash ^= self.zobrist_table[(piece, end_square)];

        Ok(())
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.bitboard)
    }
}
