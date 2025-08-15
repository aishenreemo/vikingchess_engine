use std::collections::HashMap;

use ron::de::SpannedError;
use serde::Deserialize;
use serde::Serialize;

use crate::mask::Mask;
use crate::prelude::Bitboard;

#[derive(Debug, Serialize, Deserialize)]
pub struct MagicTable {
    pub magics: Vec<Mask>,
    pub moves: Vec<HashMap<Mask, Mask>>,
}

impl MagicTable {
    pub const MAGICS_PATH: &'static str = "./assets/magics.ron";

    #[rustfmt::skip]
    pub const SHIFTS: [u32; Bitboard::TOTAL_SQUARES] = [
        14, 13, 13, 13, 13, 13, 13, 13, 14,
        13, 12, 12, 12, 12, 12, 12, 12, 13,
        13, 12, 12, 12, 12, 12, 12, 12, 13,
        13, 12, 12, 12, 12, 12, 12, 12, 13,
        13, 12, 12, 12, 12, 12, 12, 12, 13,
        13, 12, 12, 12, 12, 12, 12, 12, 13,
        13, 12, 12, 12, 12, 12, 12, 12, 13,
        13, 12, 12, 12, 12, 12, 12, 12, 13,
        14, 13, 13, 13, 13, 13, 13, 13, 14,
    ];
}

impl From<Vec<(Mask, HashMap<Mask, Mask>)>> for MagicTable {
    fn from(item: Vec<(Mask, HashMap<Mask, Mask>)>) -> Self {
        let mut magics = Vec::with_capacity(item.len());
        let mut moves = Vec::with_capacity(item.len());

        for (magic, moves_map) in item {
            magics.push(magic);
            moves.push(moves_map);
        }

        MagicTable {
            magics,
            moves,
        }
    }
}

impl TryFrom<String> for MagicTable {
    type Error = SpannedError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ron::from_str(&value)
    }
}
