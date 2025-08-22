use std::collections::HashSet;

use super::*;
use crate::action::Action;
use crate::bitboard::Bitboard;
use crate::board::Board;
use crate::mask::Mask;
use crate::piece::Piece;
use crate::square::Square;
use crate::zobrist::ZobristTable;

#[test]
fn bitboard_test() -> VikingChessResult<()> {
    let mut board = Bitboard::default();

    assert_eq!(board[Piece::King], Mask(0));
    assert_eq!(board[Piece::Defender], Mask(0));
    assert_eq!(board[Piece::Attacker], Mask(0));

    board[Piece::King] |= Square::try_from((4, 4))?.mask();
    assert_eq!(board[Piece::King], Mask(1 << 40));
    Ok(())
}

#[test]
#[should_panic]
fn bitboard_index_panic() {
    let board = Bitboard::default();
    let _ = board[Piece::Length];
}

#[test]
fn zobrist_hash_update_test() -> VikingChessResult<()> {
    let mut board = Board::new();
    let initial_hash = board.state.zobrist_hash;

    let action = Action::new(Piece::Defender, 39.try_into()?, 30.try_into()?);
    board.state.turn = Piece::Defender;
    board.move_piece(action, None)?;
    assert_ne!(board.state.zobrist_hash, initial_hash);

    let action = Action::new(Piece::Defender, 30.try_into()?, 39.try_into()?);
    board.state.turn = Piece::Defender;
    board.move_piece(action, None)?;
    assert_eq!(board.state.zobrist_hash, initial_hash);
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
    assert_eq!(table[(piece, square)], (*table)[expected_index]);
}

#[test]
fn square_from_usize_test() -> VikingChessResult<()> {
    let square_index = 15;
    let square = Square::try_from(square_index)?;

    assert_eq!(square.row, 1);
    assert_eq!(square.col, 6);
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
fn bitboard_iter_test() -> VikingChessResult<()> {
    let mut bitboard = Bitboard::default();
    let squares = [
        Square::try_from((4, 4))?,
        Square::try_from((0, 0))?,
        Square::try_from((8, 8))?,
    ];

    bitboard[Piece::King] |= squares[0].mask();
    bitboard[Piece::Attacker] |= squares[1].mask();
    bitboard[Piece::Defender] |= squares[2].mask();

    let mut iter = bitboard.iter();
    assert_eq!(iter.next(), Some((Piece::Attacker, squares[1])));
    assert_eq!(iter.next(), Some((Piece::King, squares[0])));
    assert_eq!(iter.next(), Some((Piece::Defender, squares[2])));
    assert_eq!(iter.next(), None);
    Ok(())
}

#[test]
fn square_adjacent_test() -> VikingChessResult<()> {
    let squares = [
        Square::try_from((4, 4))?,
        Square::try_from((0, 0))?,
        Square::try_from((8, 8))?,
    ];

    #[rustfmt::skip]
    let adjacent_masks = [
        Mask(565700879974400),
        Mask(514),
        Mask(606824093048749409959936)
    ];

    #[rustfmt::skip]
    let interjacent_mask = [
        Mask(288235049080324096),
        Mask(262148),
        Mask(302236066589675721064448)
    ];

    let iter = squares.iter().zip(adjacent_masks.iter().zip(interjacent_mask.iter()));
    for (square, (adjacent_mask, interjacent_mask)) in iter {
        assert_eq!(&square.adjacent_mask(), adjacent_mask);
        assert_eq!(&square.interjacent_mask(), interjacent_mask);
    }

    Ok(())
}
