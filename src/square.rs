use serde::Serialize;

use crate::{VikingChessError, VikingChessResult};
use crate::bitboard::Bitboard;
use crate::mask::Mask;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize)]
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

    pub fn mask(&self) -> Mask {
        Mask(1 << self.index())
    }

    pub fn try_from_offset(&self, offset: i8) -> VikingChessResult<Self> {
        Self::try_from((
            (offset % 5 - 2 + self.col as i8) as f32,
            (offset / 5 - 2 + self.row as i8) as f32,
        ))
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
