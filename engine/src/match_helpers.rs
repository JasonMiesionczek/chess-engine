use crate::{
    chess_match::ChessMatch,
    piece_base::{ChessPiece, PieceColor},
    piece_location::PieceLocation,
};

pub struct MatchHelpers {}

impl MatchHelpers {
    pub fn get_pieces_with_valid_captures(
        chess_match: &ChessMatch,
        location: &PieceLocation,
        color: &PieceColor,
    ) -> Vec<ChessPiece> {
        let pieces = chess_match.get_player_pieces_in_play(color);
        let matching_pieces = pieces
            .into_iter()
            .filter(|p| p.get_valid_captures().contains(location))
            .collect();

        matching_pieces
    }

    pub fn any_piece_has_valid_capture(
        chess_match: &ChessMatch,
        location: &PieceLocation,
        color: &PieceColor,
    ) -> bool {
        let pieces = chess_match.get_player_pieces_in_play(color);
        pieces
            .iter()
            .any(|p| p.get_valid_captures().contains(location))
    }

    pub fn locations_can_be_attacked(
        locations: Vec<PieceLocation>,
        chess_match: &ChessMatch,
    ) -> Vec<PieceLocation> {
        let mut result: Vec<PieceLocation> = Vec::new();
        let pieces = chess_match.get_pieces_in_play();
        locations.iter().for_each(|loc| {
            if pieces.iter().any(|p| p.get_valid_captures().contains(loc)) {
                result.push(loc.clone());
            }
        });

        result
    }
}
