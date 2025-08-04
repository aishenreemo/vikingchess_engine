use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::ops::Deref;
use std::ops::Index;
use std::ops::IndexMut;

use rand::RngCore;
use rand::rng;

pub type VikingChessError = Box<dyn Error>;
pub type VikingChessResult<T> = Result<T, VikingChessError>;

#[derive(Default)]
pub struct Bitboard([u128; Piece::Length as usize]);

impl Bitboard {
    pub const BOARD_LENGTH: usize = 11;
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

#[repr(usize)]
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum Piece {
    King = 0,
    Defender = 1,
    Attacker = 2,
    Length,
}

impl Piece {
    pub const PIECES: [char; 3] = ['A', 'D', 'K'];
}

impl From<char> for Piece {
    fn from(value: char) -> Self {
        match value {
            'A' => Piece::Attacker,
            'D' => Piece::Defender,
            'K' => Piece::King,
            _ => panic!("Failure to convert {value} to Piece."),
        }
    }
}

pub struct ZobristTable([u64; ZobristTable::TABLE_LENGTH]);

impl ZobristTable {
    pub const TABLE_LENGTH: usize = Bitboard::TOTAL_SQUARES * Piece::Length as usize;

    pub fn new() -> Self {
        let mut keys = [0u64; Self::TABLE_LENGTH];
        let mut r = rng();
        for i in 0..Self::TABLE_LENGTH {
            keys[i] = r.next_u64();
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

impl Index<(Piece, Square)> for ZobristTable {
    type Output = u64;

    fn index(&self, index: (Piece, Square)) -> &Self::Output {
        let piece = index.0 as usize;
        let square = index.1.row as usize * Bitboard::BOARD_LENGTH + index.1.col as usize;
        &self.0[piece as usize * Bitboard::TOTAL_SQUARES + square]
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Square {
    pub row: u8,
    pub col: u8,
}

impl Square {
    pub fn new(row: u8, col: u8) -> Self {
        Self { row, col }
    }

    pub fn index(&self) -> usize {
        self.row as usize * Bitboard::BOARD_LENGTH + self.col as usize
    }

    pub fn bit(&self) -> u128 {
        1 << self.index() as u128
    }
}

impl TryFrom<(u8, u8)> for Square {
    type Error = VikingChessError;

    fn try_from(value: (u8, u8)) -> Result<Self, Self::Error> {
        const BOARD_LENGTH: u8 = Bitboard::BOARD_LENGTH as u8;
        if value.0 >= BOARD_LENGTH || value.1 >= BOARD_LENGTH {
            return Err(format!("Invalid square position ({}, {})", value.0, value.1).into());
        }

        Ok(Square {
            row: value.1,
            col: value.0,
        })
    }
}

impl TryFrom<(f32, f32)> for Square {
    type Error = VikingChessError;

    fn try_from(value: (f32, f32)) -> Result<Self, Self::Error> {
        if value.0 < 0. || value.1 < 0. {
            return Err(format!("Invalid square position ({}, {})", value.0, value.1).into());
        }

        Square::try_from((value.0 as u8, value.1 as u8))
    }
}

impl TryFrom<usize> for Square {
    type Error = VikingChessError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value >= Bitboard::TOTAL_SQUARES {
            return Err(format!("Invalid square index {value}").into());
        }

        Ok(Square {
            row: (value / Bitboard::BOARD_LENGTH) as u8,
            col: (value % Bitboard::BOARD_LENGTH) as u8,
        })
    }
}

pub struct Board {
    bitboard: Bitboard,
    zobrist_table: ZobristTable,
    zobrist_hash: u64,
}

impl Default for Board {
    fn default() -> Self {
        Self::from_fen(Self::STARTING_FEN).expect("Invalid starting FEN.")
    }
}

impl Board {
    pub const STARTING_FEN: &'static str = "4AAA4/5A5/92/5D5/A4D4A/AA1DDKDD1AA/A4D4A/5D5/92/5A5/4AAA4 B";
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn bitboard_test() -> VikingChessResult<()> {
        let mut board = Bitboard::default();

        assert_eq!(board[Piece::King], 0);
        assert_eq!(board[Piece::Defender], 0);
        assert_eq!(board[Piece::Attacker], 0);

        board[Piece::King] |= Square::try_from((5, 5))?.bit();
        assert_eq!(board[Piece::King], 1 << 60);
        println!("Board:\n{board}");
        Ok(())
    }

    #[test]
    #[should_panic]
    fn bitboard_index_panic() {
        let board = Bitboard::default();
        board[Piece::Length];
    }

    #[test]
    fn zobrist_hash_update_test() -> VikingChessResult<()> {
        let mut board = Board::new();
        let initial_hash = board.zobrist_hash;

        println!("Board 1:\n{board}");
        board.move_piece(Piece::King, 60.try_into()?, 0.try_into()?)?;
        assert_ne!(board.zobrist_hash, initial_hash);
        println!("Board 2:\n{board}");

        board.move_piece(Piece::King, 0.try_into()?, 60.try_into()?)?;
        assert_eq!(board.zobrist_hash, initial_hash);
        println!("Board 3:\n{board}");
        Ok(())
    }

    #[test]
    fn zobristkeys_no_dup() {
        let mut set = HashSet::new();
        let table = ZobristTable::new();
        for &number in table.iter() {
            assert!(set.insert(number));
        }
    }

    #[test]
    fn zobrist_table_index_test() {
        let table = ZobristTable::new();
        let piece = Piece::Defender;
        let square = Square::new(2, 3);
        let expected_index = piece as usize * Bitboard::TOTAL_SQUARES + square.index();
        assert_eq!(table[(piece, square)], table.0[expected_index]);
    }

    #[test]
    fn square_from_usize_test() -> VikingChessResult<()> {
        let square_index = 15;
        let square = Square::try_from(square_index)?;

        assert_eq!(square.row, 1);
        assert_eq!(square.col, 4);
        Ok(())
    }

    #[test]
    fn square_from_tuple_test() -> VikingChessResult<()> {
        let square_tuple = (4, 1);
        let square = Square::try_from(square_tuple)?;

        assert_eq!(square.row, 1);
        assert_eq!(square.col, 4);
        Ok(())
    }

    #[test]
    fn test_bitboard_iter() -> VikingChessResult<()> {
        let mut bitboard = Bitboard::default();
        let squares = [
            Square::try_from((5, 5))?,
            Square::try_from((0, 0))?,
            Square::try_from((10, 10))?,
        ];
            
        bitboard[Piece::King] |= squares[0].bit();
        bitboard[Piece::Attacker] |= squares[1].bit();
        bitboard[Piece::Defender] |= squares[2].bit();

        let mut iter = bitboard.iter();
        assert_eq!(iter.next(), Some((Piece::Attacker, squares[1])));
        assert_eq!(iter.next(), Some((Piece::King, squares[0])));
        assert_eq!(iter.next(), Some((Piece::Defender, squares[2])));
        assert_eq!(iter.next(), None);
        Ok(())
    }
}
