use std::{io::{self, Stdout}, thread, sync::{mpsc, Arc, Mutex}};

use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Paragraph},
    layout::{Rect, Margin},
    Terminal
};

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, event::{Event, KeyEventKind, KeyCode, KeyEvent, KeyModifiers},
};

use crate::board::Board;

type Term = Terminal<CrosstermBackend<Stdout>>;


struct ScreenState {
    input_buffer: String,
    output_buffer: String,
    board: Option<Board>,
    analyzed_boards: i32,
}

fn truncate_output(str : String, i: u16) -> String {
    let mut vec = str.lines().rev().take(i as usize).collect::<Vec<_>>();
    vec.reverse();
    vec.join("\n") + &"\n"
}

fn analysis_paragraph(state: &ScreenState) -> String {
    let score = match &state.board {
        Some(board) => board.get_score().to_string(),
        None => "???".to_string(),
    };

    format!("current score (naive): {score}\nboards analyzed: {}", state.analyzed_boards)
}

fn draw(terminal: &mut Term, state: &mut ScreenState) {
    terminal.draw(|f| {
        let full_rect = f.size();

        let input_rect = Rect::new(2, full_rect.height - 3, full_rect.width - 4, 3);
        let input_zone = Block::default()
            .borders(Borders::ALL);
        f.render_widget(input_zone, input_rect);
        let input_paragraph = Paragraph::new("> ".to_string() + &state.input_buffer);
        f.render_widget(input_paragraph, input_rect.inner(&Margin { vertical: 1, horizontal: 2 }));

        if let Some(board) = &state.board {
            let board_rect = Rect::new(2, 1, 31, 13);
            let board_zone = Block::default()
                .title("Board")
                .borders(Borders::ALL);
            f.render_widget(board_zone, board_rect.clone());

            let board_paragraph = Paragraph::new(board.display());
            f.render_widget(board_paragraph, board_rect.inner(&Margin {vertical: 2, horizontal: 4}));
        }

        let analysis_rect = Rect::new(34, 1, input_rect.width -32, 13);
        let analysis_zone = Block::default()
            .title("Analysis")
            .borders(Borders::ALL);
        f.render_widget(analysis_zone, analysis_rect.clone());
        let analysis_paragraph = Paragraph::new(analysis_paragraph(&state));
        f.render_widget(analysis_paragraph, analysis_rect.inner(&Margin {vertical: 2, horizontal: 4}));


        let output_rect = Rect::new(2, 14, full_rect.width - 4, full_rect.height - 17);
        let output_zone = Block::default()
            .title("Messages")
            .borders(Borders::ALL);
        f.render_widget(output_zone, output_rect.clone());
        let output_paragraph_rect = output_rect.inner(&Margin {vertical: 1, horizontal: 4});
        state.output_buffer = truncate_output(state.output_buffer.clone(), output_paragraph_rect.height);
        let output_zone = Paragraph::new(state.output_buffer.clone());
        f.render_widget(output_zone, output_paragraph_rect);

    }).expect("no error on draw");
}

// fn clear_crossterm_events() {
//     while crossterm::event::poll(Duration::from_nanos(1)).expect("No kaboom") {
//         _ = crossterm::event::read().expect("No kaboom");
//     }
// }

fn spawn_tui_thread(receiver: mpsc::Receiver<ScreenUpdate>, input_sender: mpsc::Sender<String>) -> thread::JoinHandle<()> {
    return thread::spawn(move || {
        enable_raw_mode().expect("success");
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).expect("success");
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).expect("success");

        // State
        let mut state = ScreenState { input_buffer: String::new(), output_buffer: String::new(), board: None, analyzed_boards: 0 };

        draw(&mut terminal, &mut state);

        loop {
            let update = receiver.recv().expect("no hangup");
            
            match update {
                ScreenUpdate::Close => break,
                ScreenUpdate::UpdateBoard(board) => state.board = Some(board),
                ScreenUpdate::UpdateOutput(output) => state.output_buffer += &output,
                ScreenUpdate::AnalysisCount(count) => state.analyzed_boards = count,
                ScreenUpdate::CrosstermEvent(Event::Key(KeyEvent { 
                    code: KeyCode::Char(c), modifiers, kind: KeyEventKind::Press, .. 
                })) => {  
                    if c == 'c' && modifiers == KeyModifiers::CONTROL {
                        break;
                    }
                    state.input_buffer.push(c);      
                },
                ScreenUpdate::CrosstermEvent(Event::Key(KeyEvent { 
                    code: KeyCode::Enter, kind: KeyEventKind::Press, .. 
                })) => {
                    state.input_buffer += "\n";
                    state.output_buffer += &"> ";
                    state.output_buffer += &state.input_buffer;
                    input_sender.send(state.input_buffer.clone()).expect("sends");
                    state.input_buffer.clear();
                }
                ScreenUpdate::CrosstermEvent(Event::Key(KeyEvent { 
                    code: KeyCode::Backspace, kind: KeyEventKind::Press, .. 
                })) => {
                    if state.input_buffer.len() > 0 {
                        state.input_buffer.pop();
                    }
                },
                ScreenUpdate::CrosstermEvent(_) => continue,
            }

            draw(&mut terminal, &mut state);
        }

        // clear_crossterm_events();
        disable_raw_mode().expect("success");

        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
        ).expect("success");
        terminal.show_cursor().expect("success");
    })
}

fn spawn_listener_thread(sender: mpsc::Sender<ScreenUpdate>) -> thread::JoinHandle<()> {
    return thread::spawn(move || {
        loop {
            let event = crossterm::event::read().expect("success");
            sender.send(ScreenUpdate::CrosstermEvent(event)).expect("message sends");
        }
    })
}

pub enum ScreenUpdate {
    Close, // Closes the screen
    UpdateBoard (Board),
    UpdateOutput (String),
    CrosstermEvent (Event),
    AnalysisCount (i32),
}

#[derive(Clone)]
pub struct ScreenManager {
    tui_thread: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
    _listener_thread: Arc<Option<thread::JoinHandle<()>>>,
    sender: mpsc::Sender<ScreenUpdate>,
    input_receiver: Arc<Mutex<mpsc::Receiver<String>>>,
}

impl ScreenManager {
    pub fn new() -> ScreenManager {
        let (sender, receiver) = mpsc::channel();
        let (input_sender, input_receiver) = mpsc::channel();

        let tui_thread = spawn_tui_thread(receiver, input_sender);
        let event_listener_thread = spawn_listener_thread(sender.clone());
        ScreenManager { 
            tui_thread: Arc::new(Mutex::new(Some(tui_thread))),
            _listener_thread: Arc::new(Some(event_listener_thread)), 
            sender, 
            input_receiver: Arc::new(Mutex::new(input_receiver))
        }
    }

    pub fn update_board(&self, board: Board) {
        self.sender.send(ScreenUpdate::UpdateBoard(board)).expect("message sends");
    }

    pub fn output_line(&self, mut msg: String) {
        msg.push('\n');
        self.sender.send(ScreenUpdate::UpdateOutput(msg)).expect("message sends");
    }

    pub fn read_line(&self) -> String {
        self.input_receiver.lock().unwrap().recv().expect("received")
    }

    pub fn update_analysis_count(&self, count: i32) {
        self.sender.send(ScreenUpdate::AnalysisCount(count)).expect("sent");
    }
}

impl Drop for ScreenManager {
    fn drop(&mut self) {
        self.sender.send(ScreenUpdate::Close).expect("messages sends");
        self.tui_thread.lock().unwrap().take().expect("thread").join().expect("cleanup succeeds");
        // self.listener_thread.take().expect("thread").join().expect("cleanup succeeds");
    }
}
