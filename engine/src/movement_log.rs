use std::fmt::Display;

use log::info;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{chess_match::ChessMatch, piece_base::PieceType, piece_location::PieceLocation};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MovementLogEntry {
    id: Uuid,
    time_span: u32, // time since previous entry
    player_id: Uuid,
    notation: String,
    piece_id: Uuid,
    start_location: PieceLocation,
    end_location: PieceLocation,
    piece_captured: bool,
    captured_piece_id: Option<Uuid>,
    opponent_king_in_check: bool,
    opponent_king_in_checkmate: bool,
    castled_king_side: bool,
    castled_queen_side: bool,
}
impl Display for MovementLogEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.notation)
    }
}

impl MovementLogEntry {
    pub fn new(
        player_id: Uuid,
        piece_id: Uuid,
        start_location: PieceLocation,
        end_location: PieceLocation,
    ) -> MovementLogEntry {
        MovementLogEntry {
            id: Uuid::new_v4(),
            player_id,
            notation: String::new(),
            piece_id,
            start_location,
            end_location,
            piece_captured: false,
            captured_piece_id: None,
            opponent_king_in_check: false,
            opponent_king_in_checkmate: false,
            castled_king_side: false,
            castled_queen_side: false,
            time_span: 0,
        }
    }

    pub fn time_span(&mut self, time_span: u32) -> &mut MovementLogEntry {
        self.time_span = time_span;
        self
    }

    pub fn captured(&mut self, captured_piece_id: Uuid) -> &mut MovementLogEntry {
        self.captured_piece_id = Some(captured_piece_id);
        self.piece_captured = true;
        self
    }

    pub fn opponent_king_in_check(&mut self) -> &mut MovementLogEntry {
        self.opponent_king_in_check = true;
        self
    }

    pub fn opponent_king_in_checkmate(&mut self) -> &mut MovementLogEntry {
        self.opponent_king_in_checkmate = true;
        self
    }

    pub fn castled_king_side(&mut self) -> &mut MovementLogEntry {
        self.castled_king_side = true;
        self
    }

    pub fn castled_queen_side(&mut self) -> &mut MovementLogEntry {
        self.castled_queen_side = true;
        self
    }

    pub fn notation(&mut self, notation: String) -> &mut MovementLogEntry {
        self.notation = notation;
        self
    }

    pub fn get_start_location(&self) -> PieceLocation {
        self.start_location.clone()
    }

    pub fn get_end_location(&self) -> PieceLocation {
        self.end_location.clone()
    }

    pub fn get_notation(&self) -> String {
        self.notation.clone()
    }
}

pub struct MovementLogger {}

impl MovementLogger {
    pub fn add_entry_to_match(
        chess_match: &mut ChessMatch,
        entry: MovementLogEntry,
    ) -> MovementLogEntry {
        let mut entry = entry.clone();
        let piece = chess_match.get_piece_by_id_copy(&entry.piece_id);
        let piece_text = piece.get_notation_text();
        let start_location_text = entry.get_start_location();
        let captured_text = if entry.piece_captured {
            if piece.get_type() == PieceType::Pawn {
                format!("{}x", start_location_text.get_file())
            } else {
                "x".to_string()
            }
        } else {
            "".to_string()
        };
        let end_location_text = entry.get_end_location().to_string();
        if entry.castled_king_side {
            return entry.notation("O-O".to_string()).clone();
        }
        if entry.castled_queen_side {
            return entry.notation("O-O-O".to_string()).clone();
        }
        if piece.got_promoted() {
            return entry
                .notation(format!("{}={}", end_location_text, piece_text))
                .clone();
        }
        let check_suffix = if entry.opponent_king_in_check {
            "+".to_string()
        } else {
            "".to_string()
        };
        let checkmate_suffix = if entry.opponent_king_in_checkmate {
            "#".to_string()
        } else {
            "".to_string()
        };

        let final_notation = format!(
            "{}{}{}{}{}",
            piece_text, captured_text, end_location_text, check_suffix, checkmate_suffix
        );

        let result = entry.notation(final_notation).clone();
        info!("Log entry added: {:?}", result.clone());
        chess_match.add_log_entry(result.clone());
        result
    }

    pub fn get_formatted_entries(chess_match: &ChessMatch) -> String {
        let mut current_turn = 1;
        let mut result = String::new();
        let mut entry_text = String::new();
        let mut first_move = true;

        for entry in &chess_match.get_log_entries() {
            if first_move {
                let space = if current_turn > 1 { " " } else { "" };
                entry_text = format!("{}{}.{}", space, current_turn, entry.get_notation());
                first_move = false;
                continue;
            } else {
                first_move = true;
                entry_text = format!("{} {}", entry_text, entry.get_notation());
                result.push_str(entry_text.as_str());
                current_turn += 1;
            }
        }

        result
    }
}
