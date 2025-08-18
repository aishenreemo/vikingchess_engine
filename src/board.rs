use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use crate::VikingChessResult;
use crate::bitboard::Bitboard;
use crate::bitboard::BitboardIter;
use crate::magics::MagicTable;
use crate::mask::Mask;
use crate::piece::Piece;
use crate::square::Square;
use crate::zobrist::ZobristTable;

pub struct Board {
    bitboard: Bitboard,
    zobrist_table: ZobristTable,
    pub zobrist_hash: u64,
    turn: Piece,
}

impl Default for Board {
    fn default() -> Self {
        Self::from_fen(Self::STARTING_FEN).expect("Invalid starting FEN.")
    }
}

impl Board {
    pub const STARTING_FEN: &'static str = "3AAA3/4A4/4D4/A3D3A/AADDKDDAA/A3D3A/4D4/4A4/3AAA3 B";
    pub const EMPTY_FEN: &'static str = "9/9/9/9/9/9/9/9/9 B";

    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_fen(str: &'static str) -> VikingChessResult<Self> {
        let mut fen_iter = str.split(" ");
        let bitboard = Bitboard::from_fen(fen_iter.next().expect("Invalid FEN; No state specified."))?;
        let zobrist_table = ZobristTable::new();
        let initial_hash = Board::calculate_hash(&bitboard, &zobrist_table);
        let turn = match fen_iter.next() {
            Some("B") => Piece::Attacker,
            Some("W") => Piece::Defender,
            x => panic!("Invalid FEN; Current turn is not specified. {x:?}"),
        };

        Ok(Self {
            bitboard,
            zobrist_table,
            zobrist_hash: initial_hash,
            turn,
        })
    }

    pub fn iter_bitboard<'a>(&'a self) -> BitboardIter<'a> {
        self.bitboard.iter()
    }

    pub fn turn_mask(&self) -> Mask {
        match self.turn {
            Piece::Attacker => self.bitboard[Piece::Attacker],
            Piece::Defender => self.bitboard[Piece::Defender] | self.bitboard[Piece::King],
            _ => panic!("Invalid current turn."),
        }
    }

    fn calculate_hash(bitboard: &Bitboard, zobrist_table: &ZobristTable) -> u64 {
        let mut hash = 0;
        for (piece, square) in bitboard.iter() {
            hash ^= zobrist_table[(piece, square)];
        }

        hash
    }

    fn toggle_turn(&mut self) {
        self.turn = match self.turn {
            Piece::Attacker => Piece::Defender,
            Piece::Defender => Piece::Attacker,
            _ => panic!("Invalid current turn."),
        }
    }

    pub fn move_piece(
        &mut self,
        piece: Piece,
        start_square: Square,
        end_square: Square,
        magic_table: Option<&MagicTable>,
    ) -> VikingChessResult<()> {
        if self.bitboard[piece] & start_square.mask() <= Mask(0) {
            panic!("There is no {piece:?} in start_square {start_square:?}");
        }

        if start_square.mask() & self.turn_mask() <= Mask(0) {
            return Err(format!("{piece:?} does not have the current turn yet.").into());
        }

        if (piece != Piece::King) && ((end_square.mask() & Mask::CORNER_MASK) > Mask(0)) {
            return Err(format!("Pieces can't move to the corner besides the king.").into());
        }

        if end_square.mask() & Mask::THRONE_MASK > Mask(0) {
            return Err(format!("No one can go to the throne.").into());
        }

        let blockers = Bitboard::moves(start_square) & self.bitboard.all();
        let moves = match magic_table {
            Some(magic_table) => {
                let blockers = blockers & Bitboard::blockers(start_square);
                let square_index = start_square.index();
                let magic = magic_table.magics[square_index];
                let shift = MagicTable::SHIFTS[square_index];
                let index = Mask(blockers.wrapping_mul(magic.0) >> (128 - shift));
                magic_table.moves[square_index][&index] & !self.bitboard.all()
            }
            None => Bitboard::legal_moves(start_square, blockers),
        };

        if !moves & end_square.mask() > Mask(0) {
            return Err(format!("Invalid move.").into());
        }

        self.bitboard[piece] &= !(start_square.mask());
        self.bitboard[piece] |= end_square.mask();

        self.zobrist_hash ^= self.zobrist_table[(piece, start_square)];
        self.zobrist_hash ^= self.zobrist_table[(piece, end_square)];
        self.toggle_turn();

        Ok(())
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.bitboard)
    }
}
