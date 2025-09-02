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

#[test]
fn test_legal_moves_no_blockers() {
    let blockers = Mask(0);
    let start_square = Square::try_from((4, 4)).unwrap();
    let legal_moves = Bitboard::legal_moves(start_square, blockers);

    let mut expected_moves = Mask(0);
    for r in 5..9 {
        expected_moves |= Square::try_from((4, r)).unwrap().mask();
    }
    for r in 0..4 {
        expected_moves |= Square::try_from((4, r)).unwrap().mask();
    }
    for f in 5..9 {
        expected_moves |= Square::try_from((f, 4)).unwrap().mask();
    }
    for f in 0..4 {
        expected_moves |= Square::try_from((f, 4)).unwrap().mask();
    }

    assert_eq!(legal_moves, expected_moves, "No blockers on a central square");
    assert_eq!(legal_moves.0.count_ones(), 16, "Expected 16 legal moves with no blockers");
}

#[test]
fn test_legal_moves_with_blocker_up() {
    let blocker_square = Square::try_from((4, 6)).unwrap();
    let blockers = blocker_square.mask();
    let start_square = Square::try_from((4, 4)).unwrap();
    let legal_moves = Bitboard::legal_moves(start_square, blockers);

    let mut expected_moves = Mask(0);
    for r in 5..6 {
        expected_moves |= Square::try_from((4, r)).unwrap().mask();
    }
    for r in 0..4 {
        expected_moves |= Square::try_from((4, r)).unwrap().mask();
    }
    for f in 5..9 {
        expected_moves |= Square::try_from((f, 4)).unwrap().mask();
    }
    for f in 0..4 {
        expected_moves |= Square::try_from((f, 4)).unwrap().mask();
    }

    assert_eq!(legal_moves, expected_moves, "Blocked by a piece on row 6");
    assert_eq!(legal_moves.0.count_ones(), 13, "Expected 13 legal moves");
}

#[test]
fn test_legal_moves_with_multiple_blockers() {
    let blocker1_square = Square::try_from((4, 6)).unwrap();
    let blocker2_square = Square::try_from((2, 4)).unwrap();
    let blockers = blocker1_square.mask() | blocker2_square.mask();
    let start_square = Square::try_from((4, 4)).unwrap();
    let legal_moves = Bitboard::legal_moves(start_square, blockers);

    let mut expected_moves = Mask(0);
    expected_moves |= Square::try_from((3, 4)).unwrap().mask();
    expected_moves |= Square::try_from((4, 5)).unwrap().mask();
    for r in 0..4 {
        expected_moves |= Square::try_from((4, r)).unwrap().mask();
    }
    for f in 5..9 {
        expected_moves |= Square::try_from((f, 4)).unwrap().mask();
    }

    assert_eq!(legal_moves, expected_moves, "Blocked by two pieces");
    assert_eq!(legal_moves.0.count_ones(), 10, "Expected 10 legal moves");
}

#[test]
fn test_legal_moves_edge_case_corner() {
    let blockers = Mask(0);
    let start_square = Square::try_from((0, 0)).unwrap();
    let legal_moves = Bitboard::legal_moves(start_square, blockers);

    let mut expected_moves = Mask(0);
    for f in 1..9 {
        expected_moves |= Square::try_from((f, 0)).unwrap().mask();
    }
    for r in 1..9 {
        expected_moves |= Square::try_from((0, r)).unwrap().mask();
    }


    assert_eq!(legal_moves, expected_moves, "No blockers on a corner square");
    assert_eq!(legal_moves.0.count_ones(), 16, "Expected 16 legal moves");
}

#[test]
fn test_legal_moves_edge_case_side() {
    let blockers = Mask(0);
    let start_square = Square::try_from((0, 4)).unwrap();
    let legal_moves = Bitboard::legal_moves(start_square, blockers);

    let mut expected_moves = Mask(0);
    for f in 1..9 {
        expected_moves |= Square::try_from((f, 4)).unwrap().mask();
    }
    for r in 0..4 {
        expected_moves |= Square::try_from((0, r)).unwrap().mask();
    }
    for r in 5..9 {
        expected_moves |= Square::try_from((0, r)).unwrap().mask();
    }

    assert_eq!(legal_moves, expected_moves, "No blockers on a side square");
    assert_eq!(legal_moves.0.count_ones(), 16, "Expected 16 legal moves");
}
