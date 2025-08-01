use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::ops::Index;
use std::ops::IndexMut;

#[derive(Default)]
pub struct Bitboard([u128; Piece::Length as usize]);

impl Bitboard {
    pub const BOARD_LENGTH: usize = 11;
    pub const TOTAL_SQUARES: usize = Bitboard::BOARD_LENGTH * Bitboard::BOARD_LENGTH;
}

impl Display for Bitboard {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        for i in 0..Bitboard::TOTAL_SQUARES {
            let col = i % Bitboard::BOARD_LENGTH;

            write!(f, "{}", match i {
                i if (self[Piece::King] >> i) & 1 == 1 => "K",
                i if (self[Piece::Defender] >> i) & 1 == 1 => "D",
                i if (self[Piece::Attacker] >> i) & 1 == 1 => "A",
                _ => ".",
            })?;

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
#[derive(Debug, PartialEq, PartialOrd)]
pub enum Piece {
    King = 0,
    Defender = 1,
    Attacker = 2,
    Length,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitboard_test() {
        let mut board = Bitboard::default();

        assert_eq!(board[Piece::King], 0);
        assert_eq!(board[Piece::Defender], 0);
        assert_eq!(board[Piece::Attacker], 0);

        board[Piece::King] |= 1 << 60;
        assert_eq!(board[Piece::King], 1 << 60);
        println!("Board:\n{board}");
    }

    #[test]
    #[should_panic]
    fn bitboard_index_panic() {
        let board = Bitboard::default();
        board[Piece::Length];
    }
}
