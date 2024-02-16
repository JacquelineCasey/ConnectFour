
mod board;
mod screen;
mod analysis;

use std::sync::mpsc;

use board::Board;
use screen::ScreenManager;
use analysis::spawn_analysis_thread;

fn main() {
    let screen = ScreenManager::new();
    let mut board = Board::new();

    screen.update_board(board.clone());

    let (analysis_send, analysis_receive) = mpsc::channel();
    let _analysis_thread = spawn_analysis_thread(screen.clone(), board.clone(), analysis_receive);

    while let Some(player) = board.next_to_move() {
        screen.update_board(board.clone());
        screen.output_line(format!("{:?} to move. Input [1-7].", player));

        let buf = screen.read_line();

        let i = match buf.trim().parse::<i32>() {
            Ok(i) => i,
            Err(_) => {
                screen.output_line(format!("Bad input, try again"));
                continue;
            }
        };

        board = match board.play(i - 1, player, true) { // Subtract 1 to get to board coordinates
            Ok(next) => next,
            Err(msg) => {
                screen.output_line(format!("{msg}"));
                continue;
            }
        };

        analysis_send.send(board.clone()).expect("Sends");
    }

    screen.update_board(board.clone());
    match board.winner() {
        Some(player) => screen.output_line(format!("Game Over.\n{player:?} WINS!")),
        None => screen.output_line(format!("Game Over.\nIt's a draw.")),
    }

    screen.output_line("Press [ENTER] to leave".into());
    screen.read_line();
}
