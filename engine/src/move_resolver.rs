use std::collections::HashMap;

use log::debug;
use uuid::Uuid;

use crate::{
    chess_match::{CastleSide, ChessMatch, KingCastleData},
    piece_base::{ChessPiece, LocationState, MoveDirection, PeekResult, PieceColor, PieceType},
    piece_location::PieceLocation,
};
#[derive(Debug)]
struct PieceValidMove {
    id: Uuid,
    location: PieceLocation,
    piece_type: PieceType,
}

pub struct MoveResolver {}

impl MoveResolver {
    pub fn calculate_valid_moves(&self, chess_match: &mut ChessMatch) {
        debug!("Calculating valid moves");
        let mut pieces = chess_match.get_pieces_in_play();
        for mut p in &mut pieces {
            p.clear_all_moves();

            match p.get_type() {
                PieceType::Pawn => self.calculate_pawn_moves(&mut p, &chess_match),
                PieceType::Rook => self.calculate_rook_moves(&mut p, &chess_match),
                PieceType::Knight => self.calculate_knight_moves(&mut p, &chess_match),
                PieceType::Bishop => self.calculate_bishop_moves(&mut p, &chess_match),
                PieceType::Queen => self.calculate_queen_moves(&mut p, &chess_match),
                PieceType::King => {
                    self.calculate_king_moves(&mut p, &chess_match);
                    self.calculate_king_can_castle(&mut p, chess_match);

                    // for valid king moves/captures, have to simulate each one and if the king would end up in check,
                    // remove those moves/captures
                    //self.handle_king_cant_move_into_check(&mut p, chess_match);
                }
            }
        }

        chess_match.set_pieces(pieces.clone());
    }

    pub fn handle_king_cant_move_into_check(&self, chess_match: &mut ChessMatch) {
        let mut moves_to_remove: Vec<(Uuid, PieceLocation)> = Vec::new();
        let mut caps_to_remove: Vec<(Uuid, PieceLocation)> = Vec::new();

        {
            let kings = chess_match.get_pieces_by_type(PieceType::King);
            for king in kings {
                let color = king.get_color();
                //let mut moves_to_remove = Vec::new();
                //let mut captures_to_remove = Vec::new();
                for m in &king.get_valid_moves() {
                    let mut match_clone = chess_match.copy();
                    self.simulate_move(&mut match_clone, &king, m.clone());
                    if color == PieceColor::White && match_clone.get_white_king_in_check() {
                        //moves_to_remove.push(m.clone());
                        moves_to_remove.push((king.id.clone(), m.clone()));
                    }
                }

                for c in &king.get_valid_captures() {
                    let mut match_clone = chess_match.copy();
                    self.simulate_capture(&mut match_clone, &king, c.clone());
                    if color == PieceColor::White && match_clone.get_white_king_in_check() {
                        //captures_to_remove.push(c.clone());
                        caps_to_remove.push((king.id.clone(), c.clone()));
                    }
                }
            }
        }

        for (id, location) in moves_to_remove {
            let piece = chess_match.get_piece_by_id(&id);
            piece.remove_valid_move(&location);
        }

        for (id, location) in caps_to_remove {
            let piece = chess_match.get_piece_by_id(&id);
            piece.remove_valid_captures(&location);
        }
    }

    fn calculate_checkmate(&self, chess_match: &mut ChessMatch) {
        if chess_match.get_white_king_in_check() {
            let white_pieces = chess_match.get_player_pieces_in_play(&PieceColor::White);
            let mut white_is_checkmate = true;
            for p in white_pieces {
                if !p.get_valid_moves().is_empty() || !p.get_valid_captures().is_empty() {
                    white_is_checkmate = false;
                }
            }

            chess_match.set_white_king_checkmate(white_is_checkmate);
        } else if chess_match.get_black_king_in_check() {
            let black_pieces = chess_match.get_player_pieces_in_play(&PieceColor::Black);
            let mut black_is_checkmate = true;
            for p in black_pieces {
                if !p.get_valid_moves().is_empty() || !p.get_valid_captures().is_empty() {
                    black_is_checkmate = false;
                }
            }

            chess_match.set_black_king_checkmate(black_is_checkmate);
        }
    }

    pub fn handle_king_in_check(&self, chess_match: &mut ChessMatch) {
        // when a king is in check, the only valid moves are ones that result
        // in the king not being in check anymore.
        // this includes moving the king, moving another piece to block the king
        // capturing the opposing piece that is threatening the king
        let white_king_in_check = chess_match.get_white_king_in_check();
        let black_king_in_check = chess_match.get_black_king_in_check();

        if !black_king_in_check && !white_king_in_check {
            return;
        }

        let mut new_valid_moves: Vec<PieceValidMove> = Vec::new();
        let mut new_valid_captures: Vec<PieceValidMove> = Vec::new();

        for p in &chess_match.pieces {
            for m in p.get_valid_moves() {
                let mut match_clone = chess_match.clone();
                debug!(
                    "Simulating move for piece: {:?} {:?} at {:?} and move to {:?}",
                    p.get_color(),
                    p.get_type(),
                    p.location,
                    m
                );
                self.simulate_move(&mut match_clone, &p, m.clone());
                if (white_king_in_check && !match_clone.get_white_king_in_check())
                    || (black_king_in_check && !match_clone.get_black_king_in_check())
                {
                    debug!("New valid move found");
                    new_valid_moves.push(PieceValidMove {
                        id: p.id.clone(),
                        location: m.clone(),
                        piece_type: p.get_type(),
                    })
                }
            }

            for c in p.get_valid_captures() {
                let mut match_clone = chess_match.clone();
                self.simulate_capture(&mut match_clone, &p, c.clone());
                if (white_king_in_check && !match_clone.get_white_king_in_check())
                    || (black_king_in_check && !match_clone.get_black_king_in_check())
                {
                    new_valid_captures.push(PieceValidMove {
                        id: p.id.clone(),
                        location: c.clone(),
                        piece_type: p.get_type(),
                    })
                }
            }
        }

        if white_king_in_check {
            // clear all valid moves and captures for white pieces
            debug!("clear moves for white pieces, king in check");
            for p in &mut chess_match.pieces {
                if p.get_color() == PieceColor::Black {
                    continue;
                }
                p.clear_all_moves();
            }
        } else if black_king_in_check {
            debug!("clear moves for black pieces, king in check");
            for p in &mut chess_match.pieces {
                if p.get_color() == PieceColor::White {
                    continue;
                }
                p.clear_all_moves();
            }
        }

        if !new_valid_moves.is_empty() {
            debug!("found new valid moves {:?}", new_valid_moves);
            for valid_move in new_valid_moves {
                let piece_to_update = chess_match.get_piece_by_id(&valid_move.id);
                piece_to_update.add_valid_move(&valid_move.location.clone());
            }
        } else {
            debug!("no valid moves found");
        }

        if !new_valid_captures.is_empty() {
            debug!("found new valid captures {:?}", new_valid_captures);
            for valid_capture in new_valid_captures {
                let piece_to_update = chess_match.get_piece_by_id(&valid_capture.id);
                piece_to_update.add_valid_capture(&valid_capture.location.clone());
            }
        } else {
            debug!("no valid captures found");
        }

        self.calculate_checkmate(chess_match);
    }

    fn simulate_move(
        &self,
        chess_match: &mut ChessMatch,
        piece: &ChessPiece,
        location: PieceLocation,
    ) {
        let piece = chess_match.get_piece_by_id(&piece.id);
        piece.set_moved(location);
        let resolver = MoveResolver {};
        resolver.calculate_valid_moves(chess_match);
        resolver.calculate_king_in_check(chess_match);
        //resolver.handle_king_cant_move_into_check(chess_match);
    }

    fn simulate_capture(
        &self,
        chess_match: &mut ChessMatch,
        piece: &ChessPiece,
        target_location: PieceLocation,
    ) {
        let source_piece = chess_match.get_piece_by_id(&piece.id);
        source_piece.set_moved(target_location.clone());
        let target_piece = chess_match
            .get_piece_at_location_mut(target_location.clone())
            .unwrap();

        target_piece.set_captured();

        let resolver = MoveResolver {};
        resolver.calculate_valid_moves(chess_match);
        resolver.calculate_king_in_check(chess_match);
    }

    fn calculate_king_moves(&self, piece: &mut ChessPiece, chess_match: &ChessMatch) {
        let directions = [
            MoveDirection::NorthEast,
            MoveDirection::SouthEast,
            MoveDirection::NorthWest,
            MoveDirection::SouthWest,
            MoveDirection::East,
            MoveDirection::South,
            MoveDirection::West,
            MoveDirection::North,
        ];

        for d in directions {
            let peek = piece.peek_direction(chess_match, &d, None);
            if peek.state == LocationState::Empty {
                piece.add_valid_move(&peek.location.unwrap());
                continue;
            }

            if peek.state == LocationState::Capture {
                piece.add_valid_capture(&peek.location.unwrap());
            }
        }
    }

    pub fn calculate_king_in_check(&self, chess_match: &mut ChessMatch) {
        let mut white_king_in_check = false;
        let mut black_king_in_check = false;

        let white_king_location = chess_match
            .get_piece_by_type_and_color_mut(&PieceType::King, &PieceColor::White)
            .location
            .clone();
        let black_king_location = chess_match
            .get_piece_by_type_and_color_mut(&PieceType::King, &PieceColor::Black)
            .location
            .clone();

        for p in chess_match.pieces.iter() {
            if p.is_captured() {
                continue;
            }

            if p.get_valid_captures().contains(&white_king_location) {
                white_king_in_check = true;
            }

            if p.get_valid_captures().contains(&black_king_location) {
                black_king_in_check = true;
            }
        }
        debug!(
            "king check results: white: {:?} black: {:?}",
            white_king_in_check, black_king_in_check
        );
        chess_match.set_white_king_in_check(white_king_in_check);
        chess_match.set_black_king_in_check(black_king_in_check);
    }

    fn calculate_king_can_castle(&self, piece: &mut ChessPiece, chess_match: &mut ChessMatch) {
        if piece.get_type() != PieceType::King || !piece.is_first_move() {
            return;
        }

        let rooks = chess_match.get_player_pieces_by_type(&piece.get_color(), &PieceType::Rook);

        // two ways to castle, king-side and queen-side.
        // for king-side, check the two tiles to the east of the king, if they are empty, and rook is still first move
        // then the king can castle that way, same with queen-size, except need to check 3 tiles
        for rook in &rooks {
            if !rook.is_first_move() {
                continue;
            }

            let (file, _) = rook.location.get_x_y();
            let color = piece.get_color();

            let direction = if file == 0f64 {
                MoveDirection::West
            } else {
                MoveDirection::East
            };

            // first step
            //let mut king_castle_data: Option<KingCastleData> = None;
            let peek_result = piece.peek_direction(chess_match, &direction, Some(&piece.location));
            if peek_result.state == LocationState::Empty {
                // second step
                let peek_result2 = piece.peek_direction(
                    chess_match,
                    &direction,
                    Some(&peek_result.location.as_ref().unwrap()),
                );
                if peek_result2.state == LocationState::Empty {
                    // third step, this is either the rook's location, or needs to be empty
                    let peek_result3 = piece.peek_direction(
                        chess_match,
                        &direction,
                        Some(&peek_result2.location.as_ref().unwrap()),
                    );
                    let p3_loc = peek_result3.location.as_ref().unwrap();
                    if *p3_loc == rook.location {
                        // we found the rook, so add the previous two results as valid moves for the king
                        let p_loc = peek_result.location.clone().unwrap();
                        let p2_loc = peek_result2.location.clone().unwrap();

                        // king can only castle if he does not pass through or land in check
                        if !chess_match.locations_are_being_attacked(vec![&p_loc, &p2_loc], &color)
                        {
                            self.add_valid_castle(piece, p_loc, p2_loc, rook, color, chess_match);
                        }
                    } else if peek_result3.state == LocationState::Empty {
                        // if we find another empty space, check once more for the rook
                        let peek_result4 = piece.peek_direction(
                            chess_match,
                            &direction,
                            Some(&peek_result3.location.as_ref().unwrap()),
                        );

                        if *peek_result4.location.as_ref().unwrap() == rook.location {
                            let p_loc = peek_result.location.clone().unwrap();
                            let p2_loc = peek_result2.location.clone().unwrap();
                            // king can only castle if he does not pass through or land in check
                            if !chess_match
                                .locations_are_being_attacked(vec![&p_loc, &p2_loc], &color)
                            {
                                self.add_valid_castle(
                                    piece,
                                    p_loc,
                                    p2_loc,
                                    rook,
                                    color,
                                    chess_match,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    fn add_valid_castle(
        &self,
        piece: &mut ChessPiece,
        p_loc: PieceLocation,
        p2_loc: PieceLocation,
        rook: &ChessPiece,
        color: PieceColor,
        chess_match: &mut ChessMatch,
    ) {
        piece.add_valid_move(&p_loc);
        piece.add_valid_move(&p2_loc);
        let rook_location = rook.location.get_x_y();
        let kcd = KingCastleData {
            king_id: piece.id.clone(),
            king_target_location: p2_loc,
            rook_id: rook.id.clone(),
            rook_target_location: p_loc,
            side: if rook_location.0 == 0f64 {
                CastleSide::QueenSide
            } else {
                CastleSide::KingSide
            },
        };
        match color {
            PieceColor::White => chess_match.white_king_castle.push(kcd),
            PieceColor::Black => chess_match.black_king_castle.push(kcd),
        }
    }

    fn calculate_queen_moves(&self, piece: &mut ChessPiece, chess_match: &ChessMatch) {
        let directions = [
            MoveDirection::NorthEast,
            MoveDirection::SouthEast,
            MoveDirection::NorthWest,
            MoveDirection::SouthWest,
            MoveDirection::East,
            MoveDirection::South,
            MoveDirection::West,
            MoveDirection::North,
        ];

        for d in directions {
            let peek = piece.peek_direction(chess_match, &d, None);
            piece.walk_direction(&d, peek.location, chess_match, None, None);
        }
    }

    fn calculate_bishop_moves(&self, piece: &mut ChessPiece, chess_match: &ChessMatch) {
        let directions = [
            MoveDirection::NorthEast,
            MoveDirection::SouthEast,
            MoveDirection::NorthWest,
            MoveDirection::SouthWest,
        ];

        for d in directions {
            let peek = piece.peek_direction(chess_match, &d, None);
            piece.walk_direction(&d, peek.location, chess_match, None, None);
        }
    }

    fn calculate_knight_moves(&self, piece: &mut ChessPiece, chess_match: &ChessMatch) {
        let directions = [
            MoveDirection::East,
            MoveDirection::South,
            MoveDirection::West,
            MoveDirection::North,
        ];

        let secondary_directions: HashMap<MoveDirection, Vec<MoveDirection>> = HashMap::from([
            (
                MoveDirection::North,
                vec![MoveDirection::NorthEast, MoveDirection::NorthWest],
            ),
            (
                MoveDirection::East,
                vec![MoveDirection::NorthEast, MoveDirection::SouthEast],
            ),
            (
                MoveDirection::South,
                vec![MoveDirection::SouthEast, MoveDirection::SouthWest],
            ),
            (
                MoveDirection::West,
                vec![MoveDirection::NorthWest, MoveDirection::SouthWest],
            ),
        ]);

        for d in directions {
            let peek = piece.peek_direction(chess_match, &d, None);
            if peek.state == LocationState::OutOfBounds {
                continue;
            }

            let secondary_direction = secondary_directions.get(&d).unwrap();
            for dd in secondary_direction {
                let loc = peek.clone().location.unwrap();
                let result = piece.peek_direction(chess_match, dd, Some(&loc));
                if result.state == LocationState::Empty {
                    piece.add_valid_move(&result.location.unwrap());
                    continue;
                }

                if result.state == LocationState::Capture {
                    piece.add_valid_capture(&result.location.unwrap());
                }
            }
        }
    }

    fn calculate_rook_moves(&self, piece: &mut ChessPiece, chess_match: &ChessMatch) {
        let directions = [
            MoveDirection::East,
            MoveDirection::South,
            MoveDirection::West,
            MoveDirection::North,
        ];

        for d in directions {
            let peek = piece.peek_direction(chess_match, &d, None);
            piece.walk_direction(&d, peek.location, chess_match, None, None);
        }
    }

    fn calculate_pawn_moves(&self, piece: &mut ChessPiece, chess_match: &ChessMatch) {
        //debug!("Calculating pawn moves");
        let forward_results = piece.peek_forward(chess_match);
        //debug!("Found {} Forward moves", forward_results.len());
        //debug!("Pawn location: {:?}", piece.location);
        for r in forward_results {
            if r.state == LocationState::Empty {
                piece.add_valid_move(&r.location.unwrap());
            }
        }

        let directions = match piece.color {
            PieceColor::White => [MoveDirection::NorthEast, MoveDirection::NorthWest],
            PieceColor::Black => [MoveDirection::SouthEast, MoveDirection::SouthWest],
        };

        for d in directions {
            let direction_result = piece.peek_direction(chess_match, &d, None);
            if direction_result.state == LocationState::Capture {
                piece.add_valid_capture(&direction_result.location.unwrap());
            }
        }
    }
}

#[cfg(test)]
mod test {
    use uuid::Uuid;

    use super::*;

    #[test]
    fn test_calculate_moves() {
        let mut chess_match = ChessMatch::new(Uuid::new_v4(), Uuid::new_v4());
        chess_match.calculate_valid_moves();

        //println!("{:?}", chess_match);
    }
}
