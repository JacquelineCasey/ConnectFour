
mod board;
mod screen;

use board::Board;
use screen::ScreenManager;

fn main() {
    let screen = ScreenManager::new();
    let mut board = Board::new();

    screen.update_board(board.clone());

    while let Some(player) = board.next_to_move() {
        screen.update_board(board.clone());
        screen.output_line(format!("{:?} to move", player));

        let buf = screen.read_line();

        let i = match buf.trim().parse::<i32>() {
            Ok(i) => i,
            Err(_) => {
                screen.output_line(format!("Bad input, try again"));
                continue;
            }
        };

        board = match board.play(i - 1, player) { // Subtract 1 to get to board coordinates
            Ok(next) => next,
            Err(msg) => {
                screen.output_line(format!("{msg}"));
                continue;
            }
        }
    }

    screen.update_board(board.clone());
    match board.winner() {
        Some(player) => screen.output_line(format!("Game Over.\n{player:?} WINS!")),
        None => screen.output_line(format!("Game Over.\nIt's a draw.")),
    }

    screen.output_line("Press [ENTER] to leave".into());
    screen.read_line();
}
