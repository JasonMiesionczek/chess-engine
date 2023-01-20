use std::collections::HashMap;

use log::debug;
use uuid::Uuid;

use crate::{
    chess_match::{CastleSide, ChessMatch, KingCastleData, KingState},
    match_helpers::MatchHelpers,
    piece_base::{ChessPiece, LocationState, MoveDirection, PeekResult, PieceColor, PieceType},
    piece_location::PieceLocation,
};

pub enum SimulateType {
    Move,
    Capture,
}
pub struct PieceValidMove {
    piece_id: Uuid,
    location: PieceLocation,
}

pub struct CheckMateResult {
    pub king_state: KingState,
    pub new_valid_moves: Vec<PieceValidMove>,
    pub new_valid_captures: Vec<PieceValidMove>,
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
                    // skip kings here, they need to be calculated last due to checking if their
                    // valid moves/captures would put them into check
                }
            }
        }

        let mut kings = chess_match.get_kings();
        kings.iter_mut().for_each(|k| {
            self.calculate_king_moves(k, chess_match);
            self.calculate_king_can_castle(k, chess_match);
        });

        chess_match.set_pieces(pieces.clone());
    }

    pub fn override_valid_moves(
        &self,
        chess_match: &mut ChessMatch,
        new_valid_moves: Vec<PieceValidMove>,
        new_valid_captures: Vec<PieceValidMove>,
    ) {
        let mut pieces = chess_match.get_pieces_in_play_mut();
        pieces.iter_mut().for_each(|p| p.clear_all_moves());

        for m in new_valid_moves {
            let piece = chess_match.get_piece_by_id(&m.piece_id);
            piece.add_valid_move(&m.location.clone());
        }

        for c in new_valid_captures {
            let piece = chess_match.get_piece_by_id(&c.piece_id);
            piece.add_valid_capture(&c.location.clone());
        }
    }

    pub fn is_king_in_check(&self, king: &ChessPiece, chess_match: &ChessMatch) -> KingState {
        let location = king.location.clone();
        let attacking_color = if king.get_color() == PieceColor::White {
            PieceColor::Black
        } else {
            PieceColor::White
        };

        // detect if king is in check
        let attacking_pieces =
            MatchHelpers::get_pieces_with_valid_captures(chess_match, &location, &attacking_color);

        if attacking_pieces.len() > 0 {
            return KingState::InCheck;
        }

        KingState::NotInCheck
    }

    pub fn is_king_in_check_or_stale_mate(
        &self,
        king: &ChessPiece,
        chess_match: &ChessMatch,
    ) -> CheckMateResult {
        let mut new_valid_moves: Vec<PieceValidMove> = Vec::new();
        let mut new_valid_captures: Vec<PieceValidMove> = Vec::new();
        let king_state = self.is_king_in_check(king, chess_match);

        // iterate through all pieceses moves and captures, checking if each one results in the
        // king still being in check
        let pieces = chess_match.get_pieces_in_play();
        for p in pieces {
            p.get_valid_moves().iter().for_each(|m| {
                let mut sim_result =
                    self.simulate_move_or_capture(SimulateType::Move, chess_match, &p, m.clone());
                self.calculate_valid_moves(&mut sim_result);
                let sim_king = sim_result.get_piece_by_id_copy(&king.id);
                let sim_king_state = self.is_king_in_check(&sim_king, &sim_result);
                if sim_king_state == KingState::NotInCheck {
                    new_valid_moves.push(PieceValidMove {
                        piece_id: p.id.clone(),
                        location: m.clone(),
                    });
                }
            });

            p.get_valid_captures().iter().for_each(|c| {
                let mut sim_result = self.simulate_move_or_capture(
                    SimulateType::Capture,
                    chess_match,
                    &p,
                    c.clone(),
                );
                self.calculate_valid_moves(&mut sim_result);
                let sim_kings = sim_result.get_kings();
                let sim_king = sim_kings.iter().find(|k| k.get_color() == king.get_color());
                if sim_king.is_none() {
                    return;
                }
                let sim_king = sim_king.unwrap();
                //let sim_king = sim_result.get_piece_by_id_copy(&king.id);
                let sim_king_state = self.is_king_in_check(&sim_king, &sim_result);
                if sim_king_state == KingState::NotInCheck {
                    new_valid_captures.push(PieceValidMove {
                        piece_id: p.id.clone(),
                        location: c.clone(),
                    });
                }
            })
        }

        let new_king_state = if new_valid_moves.len() == 0 && new_valid_captures.len() == 0 {
            if king_state == KingState::InCheck {
                KingState::InCheckMate
            } else {
                KingState::InStaleMate
            }
        } else {
            king_state
        };

        CheckMateResult {
            king_state: new_king_state,
            new_valid_moves,
            new_valid_captures,
        }
    }

    pub fn simulate_move_or_capture(
        &self,
        sim_type: SimulateType,
        chess_match: &ChessMatch,
        piece: &ChessPiece,
        location: PieceLocation,
    ) -> ChessMatch {
        let mut match_copy = chess_match.copy();

        match sim_type {
            SimulateType::Move => {
                let piece_copy = match_copy.get_piece_by_id(&piece.id);
                piece_copy.location = location.clone()
            }
            SimulateType::Capture => {
                let piece_to_capture = match_copy
                    .get_piece_at_location_mut(location.clone())
                    .unwrap();
                piece_to_capture.set_captured();
                let piece_copy = match_copy.get_piece_by_id(&piece.id);
                piece_copy.location = location.clone();
            }
        }

        match_copy
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
                let location = peek.location.clone().unwrap();
                let can_be_attacked =
                    MatchHelpers::locations_can_be_attacked(vec![location.clone()], chess_match);
                if can_be_attacked.len() == 0 {
                    piece.add_valid_move(&location);
                    continue;
                }
            }

            if peek.state == LocationState::Capture {
                let location = peek.location.clone().unwrap();
                let can_be_attacked =
                    MatchHelpers::locations_can_be_attacked(vec![location.clone()], chess_match);
                if can_be_attacked.len() == 0 {
                    piece.add_valid_capture(&location);
                }
            }
        }
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

            let (file, rank) = rook.location.get_x_y();
            let color = piece.get_color();
            let rank = rank + 1f64;

            if file == 0f64 {
                // queen side
                let file_b = PieceLocation::new_from_string(format!("b{}", rank).as_str()).unwrap();
                let file_c = PieceLocation::new_from_string(format!("c{}", rank).as_str()).unwrap();
                let file_d = PieceLocation::new_from_string(format!("d{}", rank).as_str()).unwrap();

                let locations_can_be_attacked = MatchHelpers::locations_can_be_attacked(
                    vec![file_b.clone(), file_c.clone(), file_d.clone()],
                    chess_match,
                );

                let file_b_state = rook.peek_location(&file_b, chess_match);
                let file_c_state = rook.peek_location(&file_c, chess_match);
                let file_d_state = rook.peek_location(&file_d, chess_match);

                if file_b_state == LocationState::Empty
                    && file_c_state == LocationState::Empty
                    && file_d_state == LocationState::Empty
                    && locations_can_be_attacked.len() == 0
                {
                    self.add_valid_castle(
                        piece,
                        file_c,
                        file_d,
                        rook.id,
                        color,
                        CastleSide::QueenSide,
                        chess_match,
                    );
                }
            } else {
                // king side
                let file_f = PieceLocation::new_from_string(format!("f{}", rank).as_str()).unwrap();
                let file_g = PieceLocation::new_from_string(format!("g{}", rank).as_str()).unwrap();
                let locations_can_be_attacked = MatchHelpers::locations_can_be_attacked(
                    vec![file_f.clone(), file_g.clone()],
                    chess_match,
                );
                let file_f_state = rook.peek_location(&file_f, chess_match);
                let file_g_state = rook.peek_location(&file_g, chess_match);

                if file_f_state == LocationState::Empty
                    && file_g_state == LocationState::Empty
                    && locations_can_be_attacked.len() == 0
                {
                    self.add_valid_castle(
                        piece,
                        file_g,
                        file_f,
                        rook.id,
                        color,
                        CastleSide::KingSide,
                        chess_match,
                    );
                }
            }
        }
    }

    fn add_valid_castle(
        &self,
        piece: &mut ChessPiece,
        king_loc: PieceLocation,
        rook_loc: PieceLocation,
        rook_id: Uuid,
        color: PieceColor,
        side: CastleSide,
        chess_match: &mut ChessMatch,
    ) {
        piece.add_valid_move(&king_loc);
        piece.add_valid_move(&rook_loc);
        let kcd = KingCastleData {
            king_id: piece.id.clone(),
            king_target_location: king_loc,
            rook_id,
            rook_target_location: rook_loc,
            side,
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
