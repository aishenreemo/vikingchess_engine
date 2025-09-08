use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use crate::VikingChessResult;
use crate::action::Action;
use crate::bitboard::Bitboard;
use crate::bitboard::BitboardIter;
use crate::magics::MagicTable;
use crate::mask::Mask;
use crate::piece::Piece;
use crate::square::Square;
use crate::state::State;
use crate::zobrist::ZobristTable;

pub struct Board {
    bitboard: Bitboard,
    zobrist_table: ZobristTable,
    history: Vec<State>,
    pub state: State,
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

        let state = State {
            zobrist_hash: initial_hash,
            turn,
            action: None,
        };

        let history = vec![state];

        Ok(Self {
            bitboard,
            zobrist_table,
            state,
            history,
        })
    }

    pub fn iter_bitboard<'a>(&'a self) -> BitboardIter<'a> {
        self.bitboard.iter()
    }

    pub fn turn_mask(&self) -> Mask {
        match self.state.turn {
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

    fn moves(&self, square: Square, magic_table: Option<&MagicTable>) -> Mask {
        let blockers = Bitboard::moves(square) & self.bitboard.all();
        match magic_table {
            Some(magic_table) => {
                let blockers = blockers & Bitboard::blockers(square);
                let square_index = square.index();
                let magic = magic_table.magics[square_index];
                let shift = MagicTable::SHIFTS[square_index];
                let index = Mask(blockers.wrapping_mul(magic.0) >> (128 - shift));
                magic_table.moves[square_index][&index] & !self.bitboard.all()
            }
            None => Bitboard::legal_moves(square, blockers),
        }
    }

    pub fn move_piece(&mut self, action: Action, magic_table: Option<&MagicTable>) -> VikingChessResult<()> {
        if !action.valid(&self.bitboard) {
            panic!("There is no {:?} in start_square {:?}", action.piece, action.from);
        }

        if !action.turn_valid(self.turn_mask()) {
            return Err(format!("{:?} does not have the current turn yet.", action.piece).into());
        } else if (action.piece != Piece::King) && ((action.to.mask() & Mask::CORNER_MASK) > Mask(0)) {
            return Err("Pieces can't move to the corner besides the king.".to_string().into());
        } else if action.to.mask() & Mask::THRONE_MASK > Mask(0) {
            return Err("No one can go to the throne.".to_string().into());
        }

        let moves = self.moves(action.from, magic_table);
        if !moves & action.to.mask() > Mask(0) {
            return Err("Invalid move.".to_string().into());
        }

        self.remove_piece(action.piece, action.from);
        self.add_piece(action.piece, action.to);
        self.state.action = Some(action);
        Ok(())
    }

    pub fn remove_piece(&mut self, piece: Piece, square: Square) {
        self.bitboard[piece] &= !square.mask();
        self.state.zobrist_hash ^= self.zobrist_table[(piece, square)];
    }

    pub fn add_piece(&mut self, piece: Piece, square: Square) {
        self.bitboard[piece] |= square.mask();
        self.state.zobrist_hash ^= self.zobrist_table[(piece, square)];
    }

    pub fn eliminated_pieces_iter(&self) -> EliminatedPiecesIter {
        let Some(action) = self.state.action else {
            panic!("Move a piece first.");
        };

        EliminatedPiecesIter {
            counter: 0,
            bitboard: &self.bitboard,
            killer: (action.piece, action.to),
        }
    }

    pub fn toggle_turn(&mut self) {
        self.state.turn = match self.state.turn {
            Piece::Attacker => Piece::Defender,
            Piece::Defender => Piece::Attacker,
            _ => panic!("Invalid current turn."),
        }
    }

    pub fn save(&mut self) {
        self.history.push(self.state);
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.bitboard)
    }
}

pub struct EliminatedPiecesIter<'a> {
    counter: usize,
    killer: (Piece, Square),
    bitboard: &'a Bitboard,
}

impl<'a> Iterator for EliminatedPiecesIter<'a> {
    type Item = (Piece, Square);

    fn next(&mut self) -> Option<Self::Item> {
        const OFFSETS: [(i8, i8); 4] = [(2, 7), (10, 11), (14, 13), (22, 17)];

        use Piece::*;
        let opposite = self.killer.0.opposite();
        let [ally_mask, enemy_mask] = match opposite {
            Piece::Defender => [self.bitboard[Attacker], self.bitboard[Defender] | self.bitboard[King]],
            Piece::Attacker => [self.bitboard[Defender] | self.bitboard[King], self.bitboard[Attacker]],
            _ => unreachable!(),
        };

        while self.counter < OFFSETS.len() {
            let (ally_offset, enemy_offset) = OFFSETS[self.counter];
            let ally_pos = self.killer.1.try_from_offset(ally_offset);
            let enemy_pos = self.killer.1.try_from_offset(enemy_offset);

            if ally_pos.is_err() || enemy_pos.is_err() {
                self.counter += 1;
                continue;
            }

            let ally_pos = ally_pos.unwrap();
            let enemy_pos = enemy_pos.unwrap();

            let is_ally_present = ally_pos.mask() & ally_mask > Mask(0);
            let is_enemy_present = enemy_pos.mask() & enemy_mask > Mask(0);
            let is_enemy_not_king = self.bitboard[Defender] & enemy_pos.mask() > Mask(0);

            if !is_ally_present || !is_enemy_present {
                self.counter += 1;
                continue;
            }

            let enemy_piece = match [opposite == Attacker, is_enemy_not_king] {
                [true, false] => Attacker,
                [false, true] => Defender,
                _ => King
            };

            self.counter += 1;
            return Some((enemy_piece, enemy_pos));
        }

        None
    }
}
