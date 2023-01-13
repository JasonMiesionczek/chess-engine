use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub const FILES: [&'static str; 8] = ["a", "b", "c", "d", "e", "f", "g", "h"];

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, Hash, Eq)]
pub struct PieceLocation {
    rank: u32,
    file: String,
}

impl Display for PieceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.file, self.rank)
    }
}

impl PieceLocation {
    pub fn new(file: String, rank: u32) -> PieceLocation {
        PieceLocation { rank, file }
    }

    pub fn new_from_x_y(x: i32, y: i32) -> PieceLocation {
        let file = FILES.get(x as usize).unwrap();
        let rank = y;

        PieceLocation {
            rank: rank as u32,
            file: file.to_string(),
        }
    }

    pub fn copy(&self) -> PieceLocation {
        PieceLocation {
            rank: self.rank.clone(),
            file: self.file.to_string(),
        }
    }

    pub fn new_from_string(location: &str) -> Result<PieceLocation, &str> {
        let mut chars = location.chars();

        if chars.clone().count() != 2 {
            return Err("Invalid length");
        }

        let file = match chars.next() {
            Some(f) => f.to_string(),
            None => "".to_string(),
        };
        let rank = match chars.next() {
            Some(r) => r.to_digit(10).unwrap(),
            None => 0,
        };

        if rank < 1 || rank > 8 {
            return Err("Rank out of bounds");
        }

        match FILES.iter().position(|&r| r == file) {
            None => return Err("File out of bounds"),
            _ => {}
        }

        Ok(PieceLocation { rank, file })
    }

    pub fn get_next_file(&self) -> Option<String> {
        let index = FILES.iter().position(|&r| r == self.file).unwrap();
        if index + 1 < FILES.len() {
            Some(FILES.get(index + 1).unwrap().to_string())
        } else {
            None
        }
    }

    pub fn get_previous_file(&self) -> Option<String> {
        let index: i32 = FILES.iter().position(|&r| r == self.file).unwrap() as i32;
        if index - 1 >= 0 {
            Some(FILES.get((index as usize) - 1).unwrap().to_string())
        } else {
            None
        }
    }

    pub fn move_east(&self) -> Option<PieceLocation> {
        match self.get_next_file() {
            Some(f) => Some(PieceLocation {
                rank: self.rank,
                file: f,
            }),
            None => None,
        }
    }

    pub fn move_west(&self) -> Option<PieceLocation> {
        match self.get_previous_file() {
            Some(f) => Some(PieceLocation {
                rank: self.rank,
                file: f,
            }),
            None => None,
        }
    }

    pub fn move_north(&self) -> Option<PieceLocation> {
        if self.rank == 8 {
            None
        } else {
            Some(PieceLocation {
                rank: self.rank + 1,
                file: self.file.clone(),
            })
        }
    }

    pub fn move_south(&self) -> Option<PieceLocation> {
        if self.rank == 1 {
            None
        } else {
            Some(PieceLocation {
                rank: self.rank - 1,
                file: self.file.clone(),
            })
        }
    }

    pub fn move_north_east(&self) -> Option<PieceLocation> {
        let move_east = self.move_east();
        let move_north = self.move_north();

        if move_east.is_some() && move_north.is_some() {
            Some(PieceLocation {
                rank: move_north.unwrap().rank,
                file: move_east.unwrap().file,
            })
        } else {
            None
        }
    }

    pub fn move_south_east(&self) -> Option<PieceLocation> {
        let move_east = self.move_east();
        let move_south = self.move_south();

        if move_east.is_some() && move_south.is_some() {
            Some(PieceLocation {
                rank: move_south.unwrap().rank,
                file: move_east.unwrap().file,
            })
        } else {
            None
        }
    }

    pub fn move_north_west(&self) -> Option<PieceLocation> {
        let move_west = self.move_west();
        let move_north = self.move_north();

        if move_west.is_some() && move_north.is_some() {
            Some(PieceLocation {
                rank: move_north.unwrap().rank,
                file: move_west.unwrap().file,
            })
        } else {
            None
        }
    }

    pub fn move_south_west(&self) -> Option<PieceLocation> {
        let move_west = self.move_west();
        let move_south = self.move_south();

        if move_west.is_some() && move_south.is_some() {
            Some(PieceLocation {
                rank: move_south.unwrap().rank,
                file: move_west.unwrap().file,
            })
        } else {
            None
        }
    }

    pub fn get_x_y(&self) -> (f64, f64) {
        let x = FILES.iter().position(|&r| r == self.file).unwrap();
        let y = self.rank - 1;

        (x as f64, y as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_piece_location_from_string() {
        let loc = PieceLocation::new_from_string("a1").unwrap();
        assert_eq!(loc.rank, 1);
        assert_eq!(loc.file, "a");

        let bad_rank = PieceLocation::new_from_string("a9");
        assert_eq!(Err("Rank out of bounds"), bad_rank);

        let bad_file = PieceLocation::new_from_string("t1");
        assert_eq!(Err("File out of bounds"), bad_file);

        let bad_length = PieceLocation::new_from_string("test");
        assert_eq!(Err("Invalid length"), bad_length);
    }

    #[test]
    fn test_get_next_file() {
        let loc = PieceLocation::new_from_string("a1").unwrap();
        let next_file = loc.get_next_file();
        assert_eq!(Some("b".to_string()), next_file);
    }

    #[test]
    fn test_get_prev_file() {
        let loc = PieceLocation::new_from_string("a1").unwrap();
        let prev_file = loc.get_previous_file();
        assert_eq!(None, prev_file);
    }

    #[test]
    fn test_locations_are_equal() {
        let loc1 = PieceLocation::new_from_string("a1").unwrap();
        let loc2 = PieceLocation::new_from_string("a1").unwrap();
        let loc3 = PieceLocation::new_from_string("b1").unwrap();
        let are_equal = loc1 == loc2;
        assert_eq!(are_equal, true);

        let are_not_equal = loc2 == loc3;
        assert_eq!(are_not_equal, false);
    }

    #[test]
    fn test_move_east_and_west() {
        let loc1 = PieceLocation::new_from_string("a1").unwrap();
        let loc2 = PieceLocation::new_from_string("h1").unwrap();
        let moved_east = loc1.move_east().unwrap();
        let moved_west = moved_east.move_west().unwrap();
        let cant_move_west = loc1.move_west();
        let cant_move_east = loc2.move_east();

        assert_eq!("b".to_string(), moved_east.file);
        assert_eq!("a".to_string(), moved_west.file);
        assert_eq!(None, cant_move_west);
        assert_eq!(None, cant_move_east);
    }

    #[test]
    fn test_move_north_and_south() {
        let loc1 = PieceLocation::new_from_string("a1").unwrap();
        let loc2 = PieceLocation::new_from_string("a8").unwrap();

        let moved_north = loc1.move_north().unwrap();
        let moved_south = loc2.move_south().unwrap();
        let cant_move_north = loc2.move_north();
        let cant_move_south = loc1.move_south();

        assert_eq!(2, moved_north.rank);
        assert_eq!(7, moved_south.rank);
        assert_eq!(None, cant_move_north);
        assert_eq!(None, cant_move_south);
    }

    #[test]
    fn test_move_ne_se_nw_sw() {
        let loc1 = PieceLocation::new_from_string("a1").unwrap();
        let loc2 = PieceLocation::new_from_string("a8").unwrap();
        let loc3 = PieceLocation::new_from_string("h1").unwrap();
        let loc4 = PieceLocation::new_from_string("h8").unwrap();

        let moved_north_east = loc1.move_north_east().unwrap();
        let moved_south_east = loc2.move_south_east().unwrap();
        let moved_north_west = loc3.move_north_west().unwrap();
        let moved_south_west = loc4.move_south_west().unwrap();

        assert_eq!("b".to_string(), moved_north_east.file);
        assert_eq!(2, moved_north_east.rank);

        assert_eq!("b".to_string(), moved_south_east.file);
        assert_eq!(7, moved_south_east.rank);

        assert_eq!("g".to_string(), moved_north_west.file);
        assert_eq!(2, moved_north_west.rank);

        assert_eq!("g".to_string(), moved_south_west.file);
        assert_eq!(7, moved_south_west.rank);

        let cant_move_north_east = loc4.move_north_east();
        let cant_move_south_east = loc3.move_south_east();
        let cant_move_north_west = loc2.move_north_west();
        let cant_move_south_west = loc1.move_south_west();

        assert_eq!(None, cant_move_north_east);
        assert_eq!(None, cant_move_south_east);
        assert_eq!(None, cant_move_north_west);
        assert_eq!(None, cant_move_south_west);
    }
}
