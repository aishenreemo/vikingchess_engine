use crate::action::Action;
use crate::piece::Piece;

#[derive(Clone, Copy)]
pub struct State {
    pub zobrist_hash: u64,
    pub turn: Piece,
    pub action: Option<Action>,
}
