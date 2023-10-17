
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum AnalyzedTile {
    Empty,
    You,
    Enemy
}

impl Board {
    pub fn new() -> Board {
        Board {tiles: [[Tile::Empty; 7]; 6]}
    }

    fn display_row(&self, row: usize) -> String {
        let mut string  = "|".to_string();

        for i in 0..7 {
            string += match self.tiles[row][i] {
                Tile::Empty => " . ",
                Tile::Piece(Player::Red) => " R ",
                Tile::Piece(Player::Yellow) => " Y ",
            };
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

    fn win_chains() -> [(i32, i32, i32, i32); 25] {
        [ (0, 0, 0, 1), (1, 0, 0, 1), (2, 0, 0, 1), (3, 0, 0, 1), (4, 0, 0, 1), (5, 0, 0, 1)  // rows
        , (0, 0, 1, 0),  (0, 1, 1, 0),  (0, 2, 1, 0),  (0, 3, 1, 0),  (0, 4, 1, 0),  (0, 5, 1, 0), (0, 6, 1, 0)  // columns 
        , (3, 0, -1, 1), (4, 0, -1, 1), (5, 0, -1, 1), (5, 1, -1, 1), (5, 2, -1, 1), (5, 3, -1, 1)  // downward right
        , (3, 6, -1, -1), (4, 6, -1, -1), (5, 6, -1, -1), (5, 5, -1, -1), (5, 4, -1, -1), (5, 3, -1, -1)  // downward left
        ]
    }

    // None if no winner (or game ongoing)
    pub fn winner(&self) -> Option<Player> {
        for (row, col, d_r, d_c) in Self::win_chains() {
            if let Some(p) = self.win_on_chain(row, col, d_r, d_c) {
                return Some(p)
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

    pub fn next_boards(&self) -> Vec<Board> {
        let mut boards = vec![];
        let Some(player) = self.next_to_move()
            else { return boards };

        for i in 0..7 {
            if let Ok(next) = self.play(i, player) {
                boards.push(next);
            }
        }

        boards
    }

    fn score_window(window: &[AnalyzedTile]) -> i32 {
        assert!(window.len() == 4);

        let mut yours = 0;
        let mut enemies = 0;

        for tile in window {
            if tile == &AnalyzedTile::You {
                yours += 1;
            }
            else if tile == &AnalyzedTile::Enemy {
                enemies += 1;
            }
        }

        if enemies > 0 {
            0
        }
        else {
            match yours {
                1 => 5,
                2 => 50,
                3 => 500,
                _ => 0,
            }
        }
    }

    // Score for the player along a given chain.
    fn player_score_on_chain(&self, chain: (i32, i32, i32, i32), player: Player) -> i32 {
        let (mut row, mut col, d_r, d_c) = chain;
        let mut chain = vec![];

        let mut score = 0;

        while Self::in_bounds(row, col) {
            chain.push(match self.tiles[row as usize][col as usize] {
                Tile::Empty => AnalyzedTile::Empty,
                Tile::Piece(p) if p == player => AnalyzedTile::You,
                _ => AnalyzedTile::Enemy,
            });

            row += d_r;
            col += d_c;
        }

        // sliding window
        for i in 0..chain.len() -3 {
            score += Self::score_window(&chain[i..i+4]);
        } 

        score
    }

    fn pieces_played(&self) -> i32 {
        let mut count = 0;
        for row in self.tiles {
            for tile in row {
                if tile != Tile::Empty {
                    count += 1;
                }
            }
        }
        count
    }

    // Subjective score. Positive / High means win better for Red (player 1). 
    // Negative / Low means better for Yellow.
    pub fn get_score(&self) -> i32 {
        match &self.winner() {
            Some(Player::Red) => return 1000000000,
            Some(Player::Yellow) => return -1000000000,
            _ => (),
        }

        if self.next_to_move() == None {
            return 0;
        }

        let mut score = 0;
        for chain in Self::win_chains() {
            score += self.player_score_on_chain(chain, Player::Red);
            score -= self.player_score_on_chain(chain, Player::Yellow);
        }

        score
    }
}
