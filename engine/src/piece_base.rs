use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use uuid::Uuid;

use crate::{chess_match::ChessMatch, piece_location::PieceLocation};

#[derive(Clone, Debug)]
pub struct PeekResult {
    pub location: Option<PieceLocation>,
    pub state: LocationState,
}

#[derive(Clone, Debug)]
pub struct WalkTargetResult {
    pub peek_result: PeekResult,
    pub is_being_attacked: bool,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum LocationState {
    Empty,
    Capture,
    Blocked,
    OutOfBounds,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize, Hash, Eq)]
pub enum PieceType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub enum MoveDirection {
    North,
    East,
    South,
    West,
    NorthEast,
    SouthEast,
    NorthWest,
    SouthWest,
}

#[derive(Debug, PartialEq, Clone, EnumIter, Eq, Hash, Copy, Serialize, Deserialize)]
pub enum PieceColor {
    White,
    Black,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Hash, Eq)]
pub struct ChessPiece {
    pub id: Uuid,
    piece_type: PieceType,
    pub color: PieceColor,
    pub location: PieceLocation,
    captured: bool,
    first_move: bool,
    promoted: bool,
    original_piece_type: Option<PieceType>,
    valid_moves: Vec<PieceLocation>,
    valid_captures: Vec<PieceLocation>,
    points: u32,
}

impl ChessPiece {
    pub fn new(
        piece_type: PieceType,
        color: PieceColor,
        location: PieceLocation,
        points: u32,
    ) -> ChessPiece {
        ChessPiece {
            id: Uuid::new_v4(),
            piece_type,
            color,
            location,
            captured: false,
            first_move: true,
            promoted: false,
            original_piece_type: None,
            valid_moves: Vec::new(),
            valid_captures: Vec::new(),
            points,
        }
    }

    pub fn set_moved(&mut self, location: PieceLocation) {
        self.first_move = false;
        self.location = location;
    }

    pub fn set_captured(&mut self) {
        self.captured = true;
    }

    pub fn has_any_valid_moves_or_captures(&self) -> bool {
        !self.valid_moves.is_empty() || !self.valid_captures.is_empty()
    }

    pub fn peek_location(
        &self,
        location: &PieceLocation,
        chess_match: &ChessMatch,
    ) -> LocationState {
        let pieces = chess_match.get_pieces_in_play();
        let piece_at_location: Vec<ChessPiece> = pieces
            .into_iter()
            .filter(|p| p.location == *location)
            .collect();
        if !piece_at_location.is_empty() {
            let piece = &piece_at_location[0];
            if piece.color == self.color {
                return LocationState::Blocked;
            }

            return LocationState::Capture;
        }
        LocationState::Empty
    }

    pub fn peek_direction(
        &self,
        chess_match: &ChessMatch,
        direction: &MoveDirection,
        location: Option<&PieceLocation>,
    ) -> PeekResult {
        let location = if location.is_some() {
            let loc = location.unwrap();
            loc.copy()
        } else {
            self.location.clone()
        };

        let direction_location = match direction {
            MoveDirection::East => location.move_east(),
            MoveDirection::North => location.move_north(),
            MoveDirection::South => location.move_south(),
            MoveDirection::West => location.move_west(),
            MoveDirection::NorthEast => location.move_north_east(),
            MoveDirection::NorthWest => location.move_north_west(),
            MoveDirection::SouthEast => location.move_south_east(),
            MoveDirection::SouthWest => location.move_south_west(),
        };

        if direction_location.is_none() {
            return PeekResult {
                location: None,
                state: LocationState::OutOfBounds,
            };
        }
        PeekResult {
            location: direction_location.clone(),
            state: self.peek_location(&direction_location.unwrap(), chess_match),
        }
    }

    pub fn walk_to_target(
        &self,
        source_piece: &ChessPiece,
        current_location: Option<PieceLocation>,
        target_location: &PieceLocation,
        results: Vec<WalkTargetResult>,
    ) -> Vec<WalkTargetResult> {
        if current_location.is_none() {
            return results;
        }

        results
    }

    pub fn walk_direction(
        &mut self,
        direction: &MoveDirection,
        location: Option<PieceLocation>,
        chess_match: &ChessMatch,
        num_steps: Option<u32>,
        current_step: Option<u32>,
    ) {
        let num_steps = num_steps.unwrap_or(0);
        let mut current_step = current_step.unwrap_or(1);

        if location.is_none() {
            return;
        }

        let location = location.unwrap();

        if num_steps > 0 && current_step == num_steps {
            return;
        }

        current_step += 1;
        match self.peek_location(&location, chess_match) {
            LocationState::OutOfBounds | LocationState::Blocked => return,
            LocationState::Capture => {
                self.valid_captures.push(location.clone());
                return;
            }
            LocationState::Empty => {
                self.valid_moves.push(location.clone());
                let peek_result = self.peek_direction(chess_match, &direction, Some(&location));
                self.walk_direction(
                    &direction,
                    peek_result.location,
                    chess_match,
                    Some(num_steps),
                    Some(current_step),
                )
            }
        }
    }

    pub fn peek_forward(&self, chess_match: &ChessMatch) -> Vec<PeekResult> {
        let mut results: Vec<PeekResult> = Vec::new();

        let direction = match self.color {
            PieceColor::White => MoveDirection::North,
            PieceColor::Black => MoveDirection::South,
        };

        let result = self.peek_direction(chess_match, &direction, None);
        results.push(result.clone());
        if self.first_move && result.state == LocationState::Empty {
            let result =
                self.peek_direction(chess_match, &direction, Some(&result.location.unwrap()));
            results.push(result.clone());
        }

        results
    }

    pub fn is_captured(&self) -> bool {
        self.captured
    }

    pub fn is_first_move(&self) -> bool {
        self.first_move
    }

    pub fn get_color(&self) -> PieceColor {
        self.color
    }

    pub fn get_type(&self) -> PieceType {
        self.piece_type
    }

    pub fn got_promoted(&self) -> bool {
        self.promoted
    }

    pub fn add_valid_move(&mut self, location: &PieceLocation) {
        if !self.valid_moves.contains(location) {
            self.valid_moves.push(location.copy());
        }
    }

    pub fn add_valid_capture(&mut self, location: &PieceLocation) {
        if !self.valid_captures.contains(location) {
            self.valid_captures.push(location.copy());
        }
    }

    pub fn remove_valid_move(&mut self, location: &PieceLocation) {
        if self.valid_moves.contains(location) {
            let pos = self
                .valid_moves
                .iter()
                .position(|m| *m == *location)
                .unwrap();
            self.valid_moves.remove(pos);
        }
    }

    pub fn remove_valid_captures(&mut self, location: &PieceLocation) {
        if self.valid_captures.contains(location) {
            let pos = self
                .valid_captures
                .iter()
                .position(|m| *m == *location)
                .unwrap();
            self.valid_captures.remove(pos);
        }
    }

    pub fn get_valid_moves(&self) -> Vec<PieceLocation> {
        self.valid_moves.clone()
    }

    pub fn get_valid_captures(&self) -> Vec<PieceLocation> {
        self.valid_captures.clone()
    }

    pub fn clear_all_moves(&mut self) {
        self.valid_captures.clear();
        self.valid_moves.clear();
    }

    pub fn get_text(&self) -> String {
        match self.color {
            PieceColor::White => match self.piece_type {
                PieceType::Pawn => "♙".to_string(),
                PieceType::Rook => "♖".to_string(),
                PieceType::Knight => "♘".to_string(),
                PieceType::Bishop => "♗".to_string(),
                PieceType::Queen => "♕".to_string(),
                PieceType::King => "♔".to_string(),
            },
            PieceColor::Black => match self.piece_type {
                PieceType::Pawn => "♟︎".to_string(),
                PieceType::Rook => "♜".to_string(),
                PieceType::Knight => "♞".to_string(),
                PieceType::Bishop => "♝".to_string(),
                PieceType::Queen => "♛".to_string(),
                PieceType::King => "♚".to_string(),
            },
        }
    }

    pub fn get_notation_text(&self) -> String {
        if self.piece_type == PieceType::Pawn {
            "".to_string()
        } else {
            self.get_text()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peek_location() {
        let chess_match = ChessMatch::new(Uuid::new_v4(), Uuid::new_v4());
        let piece = chess_match
            .get_piece_at_location(PieceLocation::new_from_string("a1").unwrap())
            .unwrap();
        let result =
            piece.peek_location(&PieceLocation::new_from_string("a2").unwrap(), &chess_match);
        assert_eq!(LocationState::Blocked, result);

        let result =
            piece.peek_location(&PieceLocation::new_from_string("a3").unwrap(), &chess_match);
        assert_eq!(LocationState::Empty, result);

        let result =
            piece.peek_location(&PieceLocation::new_from_string("a8").unwrap(), &chess_match);
        assert_eq!(LocationState::Capture, result);
    }

    #[test]
    fn test_peek_east() {
        let chess_match = ChessMatch::new(Uuid::new_v4(), Uuid::new_v4());
        let piece = chess_match
            .get_piece_at_location(PieceLocation::new_from_string("a1").unwrap())
            .unwrap();
        let result = piece.peek_direction(&chess_match, &MoveDirection::East, None);
        assert_eq!(
            PieceLocation::new_from_string("b1").unwrap(),
            result.location.unwrap()
        );
        assert_eq!(LocationState::Blocked, result.state);

        let piece = chess_match
            .get_piece_at_location(PieceLocation::new_from_string("h1").unwrap())
            .unwrap();
        let result = piece.peek_direction(&chess_match, &MoveDirection::East, None);
        assert_eq!(None, result.location);
        assert_eq!(LocationState::OutOfBounds, result.state);
    }

    #[test]
    fn test_peek_forward() {
        let chess_match = ChessMatch::new(Uuid::new_v4(), Uuid::new_v4());
        let piece = chess_match
            .get_piece_at_location(PieceLocation::new_from_string("a2").unwrap())
            .unwrap();
        let results = piece.peek_forward(&chess_match);
        assert_eq!(2, results.len());
    }
}
