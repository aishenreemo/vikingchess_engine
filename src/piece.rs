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
    pub fn opposite(&self) -> Piece {
        match self {
            Piece::King => Piece::Attacker,
            Piece::Defender => Piece::Attacker,
            Piece::Attacker => Piece::Defender,
            _ => panic!("Invalid piece!"),
        }
    }
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
