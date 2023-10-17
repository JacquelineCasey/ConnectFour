
use std::{thread::{spawn, JoinHandle}, sync::mpsc::{self, TryRecvError}, collections::{HashMap, VecDeque}, time};
use crate::{ScreenManager, board::Board};

pub fn spawn_analysis_thread(screen: ScreenManager, 
        mut root_board: Board,
        receiver: mpsc::Receiver<Board>) -> JoinHandle<()> {

    let mut last_update = time::Instant::now();    
    
    return spawn(move || {
        let mut evaluated_boards: HashMap<Board, i32> = HashMap::new(); // Maps to value
        let mut boundary: VecDeque<Board> = VecDeque::new();  // Boards to analyze soon. BFS style queue.

        evaluated_boards.insert(root_board.clone(), root_board.get_score());
        boundary.extend(root_board.next_boards());

        loop {
            if time::Instant::now() - last_update > time::Duration::from_millis(200) {
                last_update = time::Instant::now();
                screen.update_analysis_count(evaluated_boards.len() as i32);
            }

            match receiver.try_recv() {
                Ok(board) => {
                    root_board = board;
                    boundary = VecDeque::new();

                    if !evaluated_boards.contains_key(&root_board) {
                        evaluated_boards.insert(root_board.clone(), root_board.get_score());
                    }
                }
                Err(TryRecvError::Disconnected) => {
                    panic!("They hung up!")
                }
                Err(_) => (),
            }

            let Some(curr_board) = boundary.pop_front()
                else { continue };

            if !evaluated_boards.contains_key(&curr_board) {
                evaluated_boards.insert(curr_board.clone(), curr_board.get_score());

                update_parents(&evaluated_boards, &curr_board, &root_board)
            }

            boundary.extend(curr_board.next_boards());
        }
    })
}

fn update_parents(evaluated_boards: &HashMap<Board, i32>, curr_board: &Board, root_board: &Board) {
    // pass
    
    // Examine parents, updating their values as necessary. You can stop early if the existing value
    // dominates.

    // Inform the other threads if the root is changed. Calculate next best move.

    // Do not go past the turn of root_board. Ideally, we could prune unreachable states? Maybe mark them?
}
