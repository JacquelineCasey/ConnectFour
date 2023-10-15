
use color_print::cformat;

#[derive(Hash, Clone, Copy, Eq, PartialEq, Debug)]
pub enum Player {
    Red, // First
    Yellow,
}

#[derive(Hash, Clone, Copy, Eq, PartialEq)]
pub enum Tile {
    Empty,
    Piece(Player),
}

#[derive(Hash, Clone, Eq, PartialEq)]
pub struct Board {
    pub tiles: [[Tile; 7]; 6], // row 0 is bottom!
}

impl Board {
    pub fn new() -> Board {
        Board {tiles: [[Tile::Empty; 7]; 6]}
    }

    fn display_row(&self, row: usize) -> String {
        let mut string  = "|".to_string();

        for i in 0..7 {
            string += match self.tiles[row][i] {
                Tile::Empty => " . ".into(),
                Tile::Piece(Player::Red) => cformat!(" <red>R</> "),
                Tile::Piece(Player::Yellow) => cformat!(" <yellow>Y</> "),
            }.as_str();
        }

        string += "|\n";

        string
    }

    pub fn display(&self) -> String {
        let mut string = "".to_string();
        string += "+ -  -  -  -  -  -  - +\n";

        for i in (0..6).rev() {
            string += self.display_row(i).as_str();
        }

        string += "+ -  -  -  -  -  -  - +\n";
        string += "  1  2  3  4  5  6  7  "; // user facing labels

        string
    }

    fn in_bounds(row: i32, col: i32) -> bool {
        row >= 0 && col >= 0 && row < 6 && col < 7
    }

    // Errors on wrong player or illegal move.
    pub fn play(&self, col: i32, player: Player) -> Result<Board, String> {
        if Some(player) != self.next_to_move() {
            return Err("Bad player".into());
        }
        
        if !Self::in_bounds(0, col) {
            return Err("Illegal coordinate".into());
        }

        let mut i = 5;
        while Self::in_bounds(i, col) && self.tiles[i as usize][col as usize] == Tile::Empty {
            i -= 1;
        }

        i += 1;

        if i > 5 {
            return Err("Column full".into());
        }

        let mut new_board = self.clone();

        new_board.tiles[i as usize][col as usize] = Tile::Piece(player);

        Ok(new_board)
    }

    fn win_on_chain(&self, mut row: i32, mut col: i32, d_r: i32, d_c: i32) -> Option<Player> {
        let mut red = 0;
        let mut yellow = 0;
        while Self::in_bounds(row, col) {
            match self.tiles[row as usize][col as usize] {
                Tile::Empty => {
                    red = 0;
                    yellow = 0;
                }
                Tile::Piece(Player::Red) => {
                    red += 1;
                    yellow = 0;
                    if red == 4 {
                        return Some(Player::Red);
                    }
                }
                Tile::Piece(Player::Yellow) => {
                    red = 0;
                    yellow += 1;
                    if yellow == 4 {
                        return Some(Player::Yellow);
                    }
                }
            }

            row += d_r;
            col += d_c;
        }

        None
    }

    // None if no winner (or game ongoing)
    pub fn winner(&self) -> Option<Player> {
        // rows
        for i in 0..6 {
            if let Some(p) = self.win_on_chain(i, 0, 0, 1) {
                return Some(p);
            }
        }

        // cols
        for i in 0..7 {
            if let Some(p) = self.win_on_chain(0, i, 1, 0) {
                return Some(p);
            }
        }

        // downward right
        let starts = [(3, 0), (4, 0), (5, 0), (5, 1), (5, 2), (5, 3)];
        for (i, j) in starts {
            if let Some(p) = self.win_on_chain(i, j, -1, 1) {
                return Some(p);
            }
        }

        // downward left
        let starts = [(3, 6), (4, 6), (5, 6), (5, 5), (5, 4), (5, 3)];
        for (i, j) in starts {
            if let Some(p) = self.win_on_chain(i, j, -1, -1) {
                return Some(p);
            }
        }

        None
    }

    // May panic if the board is in a bad state. Returns None if the game is over.
    pub fn next_to_move(&self) -> Option<Player> {
        if let Some(_) = self.winner() {
            None
        }
        else {
            let mut red = 0;
            let mut yellow = 0;

            for row in self.tiles {
                for tile in row {
                    match tile {
                        Tile::Piece(Player::Red) => red += 1,
                        Tile::Piece(Player::Yellow) => yellow += 1,
                        _ => (),
                    }
                }
            }

            // Red assumed to go first.
            if red + yellow == 6 * 7 {
                None
            }
            else if red == yellow {
                Some(Player::Red)
            }
            else if red == yellow + 1 {
                Some(Player::Yellow)
            }
            else {
                panic!("Board in illegal state");
            }
        }
    }
}
