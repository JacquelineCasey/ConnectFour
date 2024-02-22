
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
                match receiver.try_recv() {
                    Ok(board) => {
                        root_board = board;
                        
                        send_root_info(&mut evaluated_boards, &root_board, &screen);

                        boundary = VecDeque::new();
    
                        // if !evaluated_boards.contains_key(&root_board) {
                        //     evaluated_boards.insert(root_board.clone(), root_board.get_score());
                        // }

                        boundary.push_back(root_board.clone());
                    }
                    Err(TryRecvError::Disconnected) => {
                        panic!("They hung up!")
                    }
                    Err(_) => (),
                }

                last_update = time::Instant::now();
                screen.update_analysis_count(evaluated_boards.len() as i32);
            }

            let Some(curr_board) = boundary.pop_front()
                else { continue };

            if !evaluated_boards.contains_key(&curr_board) {
                evaluated_boards.insert(curr_board.clone(), curr_board.get_score());

                update_parents(&mut evaluated_boards, &curr_board, &root_board, &screen);
            }


            /* Can this be made better with killer move optimization? */
            if let None = curr_board.winner() {
                boundary.extend(curr_board.next_boards());
            }
        }
    })
}

fn send_root_info(evaluated_boards: &mut HashMap<Board, i32>, root_board: &Board, screen: &ScreenManager) {
    screen.update_root_score(evaluated_boards[root_board]);

    let Some(player) = root_board.next_to_move()
        else {return};

    let mut next_move = -1;
    for col in 0..7 {
        let Ok(child) = root_board.play(col, player, false)
            else {continue};

        match evaluated_boards.get(&child) {
            Some(val) if val == &evaluated_boards[root_board] => {
                next_move = col;
            },
            _ => (),
        }
    }
    
    screen.update_recomended_move(next_move);
}

fn update_parents(evaluated_boards: &mut HashMap<Board, i32>, curr_board: &Board, root_board: &Board, screen: &ScreenManager) {
    if curr_board == root_board {
        send_root_info(evaluated_boards, root_board, screen);
        return;
    }

    let parent_boards = curr_board.prev_boards();

    for parent_board in parent_boards {
        let parent_player = parent_board.next_to_move().unwrap();  // Is this repeated? Yes.
        // But there's a chance of getting none (on win) if we do it above...

        if !evaluated_boards.contains_key(&parent_board) {
            continue;
        }

        let siblings = parent_board.next_boards();

        let scores = siblings.iter()
            .map(|b| evaluated_boards.get(b))
            .flatten()
            .map(|i| i.clone());

        let score = match parent_player {
            crate::board::Player::Red => scores.max(),
            crate::board::Player::Yellow => scores.min(),
        }.unwrap();

        if evaluated_boards[&parent_board] != score {
            evaluated_boards.insert(parent_board.clone(), score);

            update_parents(evaluated_boards, &parent_board, root_board, screen)
        }
    }
}
