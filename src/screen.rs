use std::{io::{self, Stdout}, thread, sync::mpsc};

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
}

fn truncate_output(str : String, i: u16) -> String {
    let mut vec = str.lines().rev().take(i as usize).collect::<Vec<_>>();
    vec.reverse();
    vec.join("\n") + &"\n"
}

fn draw(terminal: &mut Term, state: &mut ScreenState) {
    terminal.draw(|f| {
        let full_rect = f.size();

        let input_rect = Rect::new(0, full_rect.height - 1, full_rect.width, 1);
        let input_zone = Paragraph::new("> ".to_string() + &state.input_buffer);
        f.render_widget(input_zone, input_rect);

        if let Some(board) = &state.board {
            let board_rect = Rect::new(2, 2, 27, 13);
            let board_zone = Block::default()
                .title("Board")
                .borders(Borders::ALL);
            f.render_widget(board_zone, board_rect.clone());

            let board_paragraph = Paragraph::new(board.display());
            f.render_widget(board_paragraph, board_rect.inner(&Margin {vertical: 2, horizontal: 2}));
        }

        let output_rect = Rect::new(2, 17, full_rect.width - 2, full_rect.height - 17 - 3);
        state.output_buffer = truncate_output(state.output_buffer.clone(), output_rect.height);
        let output_zone = Paragraph::new(state.output_buffer.clone());
        f.render_widget(output_zone, output_rect);

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
        let mut state = ScreenState { input_buffer: String::new(), output_buffer: String::new(), board: None };

        draw(&mut terminal, &mut state);

        loop {
            let update = receiver.recv().expect("no hangup");
            
            match update {
                ScreenUpdate::Close => break,
                ScreenUpdate::UpdateBoard(board) => state.board = Some(board),
                ScreenUpdate::UpdateOutput(output) => state.output_buffer += &output,
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
                    state.output_buffer += &state.input_buffer;
                    input_sender.send(state.input_buffer.clone()).expect("sends");
                    state.input_buffer.clear();
                }
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
            // eprintln!("{:?}", event);
            sender.send(ScreenUpdate::CrosstermEvent(event)).expect("message sends");
        }
    })
}

pub enum ScreenUpdate {
    Close, // Closes the screen
    UpdateBoard (Board),
    UpdateOutput (String),
    CrosstermEvent (Event),
}

pub struct ScreenManager {
    tui_thread: Option<thread::JoinHandle<()>>,
    _listener_thread: Option<thread::JoinHandle<()>>,
    sender: mpsc::Sender<ScreenUpdate>,
    input_receiver: mpsc::Receiver<String>,
}

impl ScreenManager {
    pub fn new() -> ScreenManager {
        let (sender, receiver) = mpsc::channel();
        let (input_sender, input_receiver) = mpsc::channel();

        let tui_thread = spawn_tui_thread(receiver, input_sender);
        let event_listener_thread = spawn_listener_thread(sender.clone());
        ScreenManager { tui_thread: Some(tui_thread), _listener_thread: Some(event_listener_thread), sender, input_receiver }
    }

    pub fn update_board(&self, board: Board) {
        self.sender.send(ScreenUpdate::UpdateBoard(board)).expect("message sends");
    }

    pub fn output_line(&self, mut msg: String) {
        msg.push('\n');
        self.sender.send(ScreenUpdate::UpdateOutput(msg)).expect("message sends");
    }

    pub fn read_line(&self) -> String {
        self.input_receiver.recv().expect("received")
    }
}

impl Drop for ScreenManager {
    fn drop(&mut self) {
        self.sender.send(ScreenUpdate::Close).expect("messages sends");
        self.tui_thread.take().expect("thread").join().expect("cleanup succeeds");
        // self.listener_thread.take().expect("thread").join().expect("cleanup succeeds");
    }
}
