use std::{cell::Cell, collections::HashMap};

use chrono::{DateTime, Utc};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use uuid::Uuid;

use crate::{
    move_resolver::MoveResolver,
    movement_log::{MovementLogEntry, MovementLogger},
    piece_base::{ChessPiece, PieceColor, PieceType},
    piece_location::{PieceLocation, FILES},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CastleSide {
    KingSide,
    QueenSide,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KingCastleData {
    pub king_id: Uuid,
    pub king_target_location: PieceLocation,
    pub rook_id: Uuid,
    pub rook_target_location: PieceLocation,
    pub side: CastleSide,
}
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Copy)]
pub enum KingState {
    InCheck,
    InCheckMate,
    InStaleMate,
    NotInCheck,
    NotInCheckMate,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChessMatch {
    id: Uuid,
    white_player: Uuid,
    black_player: Uuid,
    status: u32,
    result: u32,
    winner: Option<Uuid>,
    started: Option<DateTime<Utc>>,
    completed: Option<DateTime<Utc>>,
    current_turn: Cell<u32>,
    pub pieces: Vec<ChessPiece>,
    white_king_state: KingState,
    black_king_state: KingState,
    pub white_king_castle: Vec<KingCastleData>,
    pub black_king_castle: Vec<KingCastleData>,
    movement_log: Vec<MovementLogEntry>,
}

impl ChessMatch {
    pub fn new(white_player: Uuid, black_player: Uuid) -> ChessMatch {
        let pieces = ChessMatch::generate_pieces();

        ChessMatch {
            id: Uuid::new_v4(),
            white_player,
            black_player,
            status: 0,
            result: 0,
            winner: None,
            started: None,
            completed: None,
            current_turn: Cell::new(0),
            pieces,
            white_king_state: KingState::NotInCheck,
            black_king_state: KingState::NotInCheck,
            white_king_castle: Vec::new(),
            black_king_castle: Vec::new(),
            movement_log: Vec::new(),
        }
    }

    pub fn copy(&self) -> ChessMatch {
        ChessMatch {
            id: self.id.clone(),
            white_player: self.white_player.clone(),
            black_player: self.black_player.clone(),
            status: self.status,
            result: self.result,
            winner: self.winner,
            started: self.started,
            completed: self.completed,
            current_turn: self.current_turn.clone(),
            pieces: self.pieces.clone(),
            white_king_state: self.white_king_state.clone(),
            black_king_state: self.black_king_state.clone(),
            white_king_castle: self.white_king_castle.clone(),
            black_king_castle: self.black_king_castle.clone(),
            movement_log: self.movement_log.clone(),
        }
    }

    pub fn new_from_json(data: String) -> ChessMatch {
        serde_json::from_str(data.as_str()).expect("Error reading JSON match data")
    }

    pub fn get_match_id(&self) -> Uuid {
        self.id
    }

    pub fn get_white_player_id(&self) -> Uuid {
        self.white_player
    }

    pub fn get_black_player_id(&self) -> Uuid {
        self.black_player
    }

    pub fn get_json_string(&self) -> String {
        serde_json::to_string(self).expect("Error generating JSON output")
    }

    pub fn get_current_turn_and_color(&self) -> (u32, PieceColor) {
        (
            self.current_turn.get(),
            if self.current_turn.get() == 0 {
                PieceColor::White
            } else {
                PieceColor::Black
            },
        )
    }

    pub fn get_white_king_castle_data(&mut self) -> Vec<KingCastleData> {
        self.white_king_castle.clone()
    }

    pub fn get_black_king_castle_data(&mut self) -> Vec<KingCastleData> {
        self.black_king_castle.clone()
    }

    pub fn get_white_king_state(&self) -> KingState {
        self.white_king_state
    }

    pub fn set_white_king_state(&mut self, state: KingState) {
        self.white_king_state = state;
    }

    pub fn get_black_king_state(&self) -> KingState {
        self.black_king_state
    }

    pub fn set_black_king_state(&mut self, state: KingState) {
        self.black_king_state = state;
    }

    pub fn has_king_castle_data(&mut self, color: PieceColor) -> bool {
        match color {
            PieceColor::White => !self.white_king_castle.is_empty(),
            PieceColor::Black => !self.black_king_castle.is_empty(),
        }
    }

    pub fn set_pieces(&mut self, pieces: Vec<ChessPiece>) {
        self.pieces = pieces;
    }

    pub fn get_pieces_in_play(&self) -> Vec<ChessPiece> {
        self.pieces
            .clone()
            .into_iter()
            .filter(|p| !p.is_captured())
            .collect()
    }

    pub fn get_pieces_in_play_mut(&mut self) -> Vec<&mut ChessPiece> {
        self.pieces
            .iter_mut()
            .filter(|p| !p.is_captured())
            .collect()
    }

    pub fn get_player_pieces_in_play(&self, player: &PieceColor) -> Vec<ChessPiece> {
        let pieces_in_play = self.get_pieces_in_play();
        pieces_in_play
            .into_iter()
            .filter(|p| p.color == *player)
            .collect()
    }

    pub fn get_piece_by_type_and_color_mut(
        &mut self,
        piece_type: &PieceType,
        color: &PieceColor,
    ) -> &mut ChessPiece {
        let piece = self
            .pieces
            .iter_mut()
            .find(|p| p.get_type() == *piece_type && p.get_color() == *color);
        piece.unwrap()
    }

    pub fn get_player_pieces_by_type(
        &self,
        player: &PieceColor,
        piece_type: &PieceType,
    ) -> Vec<ChessPiece> {
        let pieces_in_play = self.get_player_pieces_in_play(player);
        pieces_in_play
            .into_iter()
            .filter(|p| p.get_type() == *piece_type)
            .collect()
    }

    pub fn get_piece_at_location(&self, location: PieceLocation) -> Option<ChessPiece> {
        let pieces = self.get_pieces_in_play();
        let piece_at_location: Vec<&ChessPiece> =
            pieces.iter().filter(|p| p.location == location).collect();
        if piece_at_location.is_empty() {
            None
        } else {
            Some(piece_at_location[0].clone())
        }
    }

    pub fn get_piece_at_location_mut(
        &mut self,
        location: PieceLocation,
    ) -> Option<&mut ChessPiece> {
        let piece = self.pieces.iter_mut().find(|p| p.location == location);
        if piece.is_some() {
            Some(piece.unwrap())
        } else {
            None
        }
    }

    pub fn location_is_being_attacked(
        &self,
        location: &PieceLocation,
        defending_player: &PieceColor,
    ) -> bool {
        let pieces = self.get_player_pieces_in_play(if *defending_player == PieceColor::White {
            &PieceColor::Black
        } else {
            defending_player
        });

        pieces
            .iter()
            .any(|p| p.get_valid_captures().contains(&location.clone()))
    }

    pub fn locations_are_being_attacked(
        &self,
        locations: Vec<&PieceLocation>,
        defending_player: &PieceColor,
    ) -> bool {
        locations
            .iter()
            .any(|loc| self.location_is_being_attacked(loc, defending_player))
    }

    pub fn calculate_valid_moves(&mut self) {
        let resolver = MoveResolver {};

        resolver.calculate_valid_moves(self);
        let kings = self.get_kings();
        for king in kings {
            let color = king.get_color();
            let check_state = resolver.is_king_in_check_or_stale_mate(&king, self);
            match check_state.king_state {
                KingState::InCheck => {
                    match color {
                        PieceColor::White => {
                            self.set_white_king_state(check_state.king_state.clone())
                        }
                        PieceColor::Black => {
                            self.set_black_king_state(check_state.king_state.clone())
                        }
                    }

                    resolver.override_valid_moves(
                        self,
                        check_state.new_valid_moves,
                        check_state.new_valid_captures,
                    );
                }
                _ => match color {
                    PieceColor::White => self.set_white_king_state(check_state.king_state.clone()),
                    PieceColor::Black => self.set_black_king_state(check_state.king_state.clone()),
                },
            }
        }
    }

    pub fn get_kings(&self) -> Vec<ChessPiece> {
        let kings = self
            .get_pieces_in_play()
            .into_iter()
            .filter(|p| p.get_type() == PieceType::King)
            .collect();
        kings
    }

    pub fn get_piece_by_id(&mut self, piece_id: &Uuid) -> &mut ChessPiece {
        let piece = self.pieces.iter_mut().find(|p| p.id == *piece_id).unwrap();
        piece
    }

    pub fn get_piece_by_id_copy(&self, piece_id: &Uuid) -> ChessPiece {
        let piece = self
            .pieces
            .iter()
            .find(|p| p.id == *piece_id)
            .expect(format!("Could not find piece with id: {}", piece_id).as_str());
        piece.to_owned()
    }

    pub fn get_pieces_by_type(&self, piece_type: PieceType) -> Vec<ChessPiece> {
        self.pieces
            .clone()
            .into_iter()
            .filter(|p| p.get_type() == piece_type)
            .collect()
    }

    pub fn handle_king_castle(
        &mut self,
        piece_id: &Uuid,
        target_location: &PieceLocation,
        movement_entry: &mut MovementLogEntry,
    ) {
        let piece = self.get_piece_by_id(piece_id);
        let color = piece.get_color();

        match color {
            PieceColor::White => {
                if self.has_king_castle_data(color) {
                    for wkc in self.get_white_king_castle_data() {
                        debug!("found king castle data: {:?}", wkc);
                        if wkc.king_target_location == *target_location {
                            // piece still moves to target location,
                            // we just also move the rook to its target location
                            let rook = self.get_piece_by_id(&wkc.rook_id);
                            rook.set_moved(wkc.rook_target_location);
                            match wkc.side {
                                CastleSide::KingSide => movement_entry.castled_king_side(),
                                CastleSide::QueenSide => movement_entry.castled_queen_side(),
                            };
                        }
                    }
                }
            }
            PieceColor::Black => {
                if self.has_king_castle_data(color) {
                    for bkc in self.get_black_king_castle_data() {
                        if bkc.king_target_location == *target_location {
                            let rook = self.get_piece_by_id(&bkc.rook_id);
                            rook.set_moved(bkc.rook_target_location);
                            match bkc.side {
                                CastleSide::KingSide => movement_entry.castled_king_side(),
                                CastleSide::QueenSide => movement_entry.castled_queen_side(),
                            };
                        }
                    }
                }
            }
        }
    }

    pub fn move_piece(&mut self, piece_id: &Uuid, location: &PieceLocation) {
        debug!("move_piece called with {:?} at {:?}", piece_id, location);
        let piece = self.get_piece_by_id_copy(piece_id);
        debug!("valid moves: {:?}", piece.get_valid_moves());

        let player_id = if piece.get_color() == PieceColor::White {
            self.get_white_player_id()
        } else {
            self.get_black_player_id()
        };
        let mut movement_entry = MovementLogEntry::new(
            player_id,
            piece_id.clone(),
            piece.location.clone(),
            location.clone(),
        );
        let can_move = piece.get_valid_moves().contains(location);
        let can_capture = piece.get_valid_captures().contains(location);
        let is_king = piece.get_type() == PieceType::King;
        if can_capture {
            self.handle_capture(location.clone(), &mut movement_entry);
        }

        if can_move || can_capture {
            self.handle_move(&piece.id, location.clone());
        }

        if is_king {
            self.handle_king_castle(piece_id, &location.clone(), &mut movement_entry);
        }

        self.change_turn();
        self.calculate_valid_moves();

        if (piece.get_color() == PieceColor::Black
            && self.get_white_king_state() == KingState::InCheck)
            || (piece.get_color() == PieceColor::White
                && self.get_black_king_state() == KingState::InCheck)
        {
            movement_entry.opponent_king_in_check();
        }

        let final_entry = MovementLogger::add_entry_to_match(self, movement_entry);
        info!("Entry logged: {}", final_entry);
    }

    fn handle_capture(&mut self, location: PieceLocation, movement_entry: &mut MovementLogEntry) {
        let piece = self.get_piece_at_location_mut(location).unwrap();
        piece.set_captured();
        movement_entry.captured(piece.id.clone());
    }

    fn handle_move(&mut self, piece_id: &Uuid, location: PieceLocation) {
        let piece = self.get_piece_by_id(piece_id);
        piece.set_moved(location);
    }

    pub fn change_turn(&mut self) -> u32 {
        if self.current_turn.get() == 0 {
            self.current_turn.set(1);
        } else {
            self.current_turn.set(0);
        }

        debug!("changed turn to: {:?}", self.current_turn);

        self.current_turn.get()
    }

    pub fn add_log_entry(&mut self, entry: MovementLogEntry) {
        self.movement_log.push(entry);
    }

    pub fn get_log_entries(&self) -> Vec<MovementLogEntry> {
        self.movement_log.clone()
    }

    fn generate_pieces() -> Vec<ChessPiece> {
        let mut result = Vec::new();
        let pawn_ranks: HashMap<PieceColor, u32> =
            HashMap::from([(PieceColor::White, 2), (PieceColor::Black, 7)]);
        let other_ranks: HashMap<PieceColor, u32> =
            HashMap::from([(PieceColor::White, 1), (PieceColor::Black, 8)]);

        fn get_location(file: usize, rank: u32) -> PieceLocation {
            PieceLocation::new(FILES.get(file).unwrap().to_string(), rank)
        }

        for color in PieceColor::iter() {
            // generate pawns
            let mut rank = pawn_ranks.get(&color).unwrap();
            for f in FILES {
                let location =
                    PieceLocation::new_from_string(format!("{}{}", f, rank).as_str()).unwrap();
                let piece = ChessPiece::new(PieceType::Pawn, color.clone(), location, 1);
                result.push(piece);
            }

            // generate rooks
            rank = other_ranks.get(&color).unwrap();
            let rook_positions = vec![0, 7];
            for p in rook_positions {
                let location = get_location(p, *rank);
                let rook = ChessPiece::new(PieceType::Rook, color.clone(), location, 5);
                result.push(rook);
            }

            // generate knights
            let knight_positions = vec![1, 6];
            for p in knight_positions {
                let location = get_location(p, *rank);
                let knight = ChessPiece::new(PieceType::Knight, color.clone(), location, 3);
                result.push(knight);
            }

            // generate bishops
            let bishop_positions = vec![2, 5];
            for p in bishop_positions {
                let location = get_location(p, *rank);
                let bishop = ChessPiece::new(PieceType::Bishop, color.clone(), location, 3);
                result.push(bishop);
            }

            // generate queen
            let queen_position = 3;
            let queen_location = get_location(queen_position, *rank);
            let queen = ChessPiece::new(PieceType::Queen, color.clone(), queen_location, 9);

            // generate king
            let king_position = 4;
            let king_location = get_location(king_position, *rank);
            let king = ChessPiece::new(PieceType::King, color.clone(), king_location, 0);

            result.push(queen);
            result.push(king);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pieces_generate() {
        let chess_match = ChessMatch::new(Uuid::new_v4(), Uuid::new_v4());

        assert_eq!(32, chess_match.pieces.len());
    }

    #[test]
    fn test_move_piece_and_update_valid_moves() {
        env_logger::init();
        let mut chess_match = ChessMatch::new(Uuid::new_v4(), Uuid::new_v4());
        chess_match.calculate_valid_moves();

        let piece = chess_match
            .get_piece_at_location(PieceLocation::new_from_string("a2").unwrap())
            .unwrap();
        chess_match.move_piece(&piece.id, &PieceLocation::new_from_string("a4").unwrap());
        let current_turn = chess_match.change_turn();
        chess_match.calculate_valid_moves();
        let piece = chess_match
            .get_piece_at_location(PieceLocation::new_from_string("a4").unwrap())
            .unwrap();
        assert_eq!(0, current_turn);
        assert_eq!(1, piece.get_valid_moves().len());
    }
}
