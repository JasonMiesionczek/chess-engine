use chess_engine::{
    chess_match::{ChessMatch, KingState},
    movement_log::MovementLogger,
    piece_base::{MoveDirection, PieceColor, PieceType},
    piece_location::PieceLocation,
};
use log::{debug, info};
use uuid::Uuid;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    env,
    error::Error,
    fs, io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{
        canvas::{Canvas, Context, Rectangle},
        Block, Borders, Clear, Paragraph,
    },
    Frame, Terminal,
};

struct App {
    pub chess_match: ChessMatch,
    current_tile: (i32, i32),
    selected_tile: Option<(i32, i32)>,
    show_saved_popup: bool,
    game_over_text: Option<String>,
}

impl App {
    fn new(chess_match: ChessMatch) -> App {
        App {
            chess_match,
            current_tile: (0, 0),
            selected_tile: None,
            show_saved_popup: false,
            game_over_text: None,
        }
    }

    fn on_tick(&mut self) {}

    fn set_current_tile(&mut self, direction: MoveDirection) {
        match direction {
            MoveDirection::East => {
                let current_x = self.current_tile.0;
                if current_x + 1 <= 7 {
                    self.current_tile = (current_x + 1, self.current_tile.1);
                }
            }
            MoveDirection::South => {
                let current_y = self.current_tile.1;
                if current_y - 1 >= 0 {
                    self.current_tile = (self.current_tile.0, current_y - 1);
                }
            }
            MoveDirection::West => {
                let current_x = self.current_tile.0;
                if current_x - 1 >= 0 {
                    self.current_tile = (current_x - 1, self.current_tile.1);
                }
            }
            MoveDirection::North => {
                let current_y = self.current_tile.1;
                if current_y + 1 <= 7 {
                    self.current_tile = (self.current_tile.0, current_y + 1);
                }
            }
            _ => {}
        }
    }

    fn handle_game_over(&mut self) {
        if self.chess_match.get_white_king_state() == KingState::InCheckMate {
            self.game_over_text = Some("Game Over! Black Wins!".to_string());
        } else if self.chess_match.get_black_king_state() == KingState::InCheckMate {
            self.game_over_text = Some("Game Over! White Wins!".to_string());
        }
    }

    fn print_match_log(&self) {
        let formatted_log = MovementLogger::get_formatted_entries(&self.chess_match);
        info!("{}", formatted_log);
    }

    fn set_selected_tile(&mut self) {
        if self.selected_tile.is_none() {
            // check if current player has a piece on selected tile
            let (_, current_color) = self.chess_match.get_current_turn_and_color();
            let (loc_x, loc_y) = self.current_tile;
            let piece = self
                .chess_match
                .get_piece_at_location(PieceLocation::new_from_x_y(loc_x, loc_y + 1));
            if piece.is_some() {
                let piece = piece.unwrap();
                debug!("Valid moves: {:?}", piece.get_valid_moves());
                if piece.color == current_color {
                    self.selected_tile = Some(self.current_tile);
                }
            }
        } else {
            if self.selected_tile.unwrap() == self.current_tile {
                self.selected_tile = None;
            } else {
                // perform the action

                // get piece at selected tile, set its location to current_tile
                let (loc_x, loc_y) = self.selected_tile.unwrap();
                let piece = self
                    .chess_match
                    .get_piece_at_location(PieceLocation::new_from_x_y(loc_x, loc_y + 1));

                if piece.is_some() {
                    let piece = piece.unwrap();
                    let (new_loc_x, new_loc_y) = self.current_tile;
                    let new_location = PieceLocation::new_from_x_y(new_loc_x, new_loc_y + 1);
                    self.chess_match.move_piece(&piece.id, &new_location);
                    if self.chess_match.get_white_king_state() == KingState::InCheckMate
                        || self.chess_match.get_black_king_state() == KingState::InCheckMate
                    {
                        self.handle_game_over();
                    }
                    self.selected_tile = None;
                } else {
                    self.selected_tile = Some(self.current_tile);
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let mut chess_match = if args.len() > 1 {
        let json_string =
            fs::read_to_string(args[1].clone()).expect("Unable to read specified file.");
        ChessMatch::new_from_json(json_string)
    } else {
        ChessMatch::new(Uuid::new_v4(), Uuid::new_v4())
    };
    chess_match.calculate_valid_moves();

    let mut show_ui = true;
    if args.len() > 2 && args[2] == "--headless" {
        show_ui = false;
    }
    if show_ui {
        // setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // create app and run it
        let tick_rate = Duration::from_millis(250);
        let mut app = App::new(chess_match);
        let res = run_app(&mut terminal, &mut app, tick_rate);

        // restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            println!("{:?}", err)
        }
        println!(
            "Log: {}",
            MovementLogger::get_formatted_entries(&app.chess_match)
        );
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Char('s') => {
                        let json_data = app.chess_match.get_json_string();
                        let filename = format!("{}.json", app.chess_match.get_match_id());
                        app.show_saved_popup = true;
                        std::fs::write(filename, json_data)
                            .expect("Error writing match data to disk");
                    }
                    KeyCode::Char('l') => {
                        app.print_match_log();
                    }
                    KeyCode::Esc => {
                        app.show_saved_popup = false;
                    }
                    KeyCode::Down => {
                        app.set_current_tile(MoveDirection::South);
                    }
                    KeyCode::Up => {
                        app.set_current_tile(MoveDirection::North);
                    }
                    KeyCode::Right => {
                        app.set_current_tile(MoveDirection::East);
                    }
                    KeyCode::Left => {
                        app.set_current_tile(MoveDirection::West);
                    }
                    KeyCode::Char(' ') => {
                        app.set_selected_tile();
                    }
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let rects = Layout::default()
        .constraints([Constraint::Ratio(1, 1)].as_ref())
        .margin(0)
        .split(f.size());
    let canvas = Canvas::default()
        .block(Block::default().borders(Borders::ALL).title("Chess"))
        .paint(|ctx| {
            draw_pieces(ctx, &app.chess_match);
            draw_board(ctx, &app.current_tile, &app.selected_tile, &app.chess_match);
        })
        .x_bounds([0.0, 17.0])
        .y_bounds([0.0, 17.0]);
    f.render_widget(canvas, rects[0]);

    let size = f.size();

    if app.show_saved_popup {
        let block = Block::default().title("Popup").borders(Borders::ALL);
        let area = centered_rect(60, 20, size);
        let text = Paragraph::new(Span::styled(
            "Match state saved successfully.",
            Style::default().fg(Color::LightGreen),
        ))
        .alignment(Alignment::Center);
        f.render_widget(Clear, area); //this clears out the background
        f.render_widget(block, area);
        f.render_widget(text, area);
    }

    if app.game_over_text.is_some() {
        let block = Block::default().title("Popup").borders(Borders::ALL);
        let area = centered_rect(60, 20, size);
        let text = Paragraph::new(Span::styled(
            app.game_over_text.as_ref().unwrap().as_str(),
            Style::default().fg(Color::LightGreen),
        ))
        .alignment(Alignment::Center);
        f.render_widget(Clear, area); //this clears out the background
        f.render_widget(block, area);
        f.render_widget(text, area);
    }
}

fn draw_pieces(ctx: &mut Context, chess_match: &ChessMatch) {
    let base_x = 2.1f64;
    let base_y = 2.25f64;
    let check_color = Color::Yellow;

    for piece in &chess_match.pieces {
        if piece.is_captured() {
            continue;
        }
        let mut color = Color::White;
        if piece.color == PieceColor::Black {
            if piece.get_type() == PieceType::King
                && chess_match.get_black_king_state() == KingState::InCheck
            {
                color = check_color;
            } else {
                color = Color::DarkGray;
            }
        } else {
            if piece.get_type() == PieceType::King
                && chess_match.get_white_king_state() == KingState::InCheck
            {
                color = check_color;
            }
        }
        let style = Style::default().fg(color);
        let spans = Spans::from(Span::styled(piece.get_text(), style));
        let location = piece.location.get_x_y();
        let x = (location.0 * base_x) + 1.0;
        let y = (location.1 * base_y) + 0.50;
        ctx.print(x, y, spans.clone());
    }
}

fn draw_board(
    ctx: &mut Context,
    current_tile: &(i32, i32),
    selected_tile: &Option<(i32, i32)>,
    chess_match: &ChessMatch,
) {
    let mut color = Color::DarkGray;
    let mut x_offset = 0f64;
    let mut y_offset = 0f64;

    let valid_moves: Vec<(i32, i32)> = if selected_tile.is_some() {
        let loc = selected_tile.unwrap();
        let piece = chess_match
            .get_piece_at_location(PieceLocation::new_from_x_y(loc.0, loc.1 + 1))
            .unwrap();
        piece
            .get_valid_moves()
            .iter()
            .map(|m| {
                let xy = m.get_x_y();
                (xy.0 as i32, xy.1 as i32)
            })
            .collect()
    } else {
        Vec::new()
    };

    let valid_captures: Vec<(i32, i32)> = if selected_tile.is_some() {
        let loc = selected_tile.unwrap();
        let piece = chess_match
            .get_piece_at_location(PieceLocation::new_from_x_y(loc.0, loc.1 + 1))
            .unwrap();
        piece
            .get_valid_captures()
            .iter()
            .map(|m| {
                let xy = m.get_x_y();
                (xy.0 as i32, xy.1 as i32)
            })
            .collect()
    } else {
        Vec::new()
    };

    for y in 0..=7 {
        if y % 2 == 0 {
            color = Color::DarkGray;
        } else {
            color = Color::White;
        }
        for x in 0..=7 {
            let is_valid_move = valid_moves.contains(&(x, y));
            let is_valid_capture = valid_captures.contains(&(x, y));
            let is_current = x == current_tile.0 && y == current_tile.1;
            let is_selected = if selected_tile.is_some() {
                let s_tile = selected_tile.unwrap();
                x == s_tile.0 && y == s_tile.1
            } else {
                false
            };
            if x > 0 {
                x_offset = (x as f64) * 1.125f64;
            } else {
                x_offset = 0f64;
            }
            if y > 0 {
                y_offset = (y as f64) * 1.125f64;
            } else {
                y_offset = 0f64;
            }
            let color_to_use = if is_selected { Color::Yellow } else { color };
            let color_to_use = if is_valid_move {
                Color::LightMagenta
            } else {
                color_to_use
            };
            let color_to_use = if is_valid_capture {
                Color::LightRed
            } else {
                color_to_use
            };
            let color_to_use = if is_current {
                Color::Green
            } else {
                color_to_use
            };
            let rect = Rectangle {
                x: (x as f64) + x_offset,
                y: (y as f64) + y_offset,
                width: 2f64,
                height: 2f64,
                color: color_to_use,
            };
            ctx.draw(&rect);
            if color == Color::DarkGray {
                color = Color::White;
            } else {
                color = Color::DarkGray;
            }
        }
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
