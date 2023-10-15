
mod board;

use board::Board;

use std::io::stdin;

fn main() {
    let mut board = Board::new();

    while let Some(player) = board.next_to_move() {
        println!("\n{}\n", board.display());

        println!("{:?} to move", player);

        let mut buf = String::new();
        stdin().read_line(&mut buf).expect("Good io");

        let i = match buf.trim().parse::<i32>() {
            Ok(i) => i,
            Err(_) => {
                println!("Bad input, try again");
                continue;
            }
        };

        board = match board.play(i - 1, player) { // Subtract 1 to get to board coordinates
            Ok(next) => next,
            Err(msg) => {
                println!("{msg}");
                continue;
            }
        }
    }

    println!("\n{}\n", board.display());
    match board.winner() {
        Some(player) => println!("Game Over.\n{player:?} WINS!"),
        None => println!("Game Over.\nIt's a draw."),
    }
}
