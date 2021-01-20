use crate::Document;
use crate::Row;
use crate::Terminal;
use std::env;
use std::time::Duration;
use std::time::Instant;
use termion::color;
use termion::event::Key;

const STATUS_FG_COLOR: color::Rgb = color::Rgb(63, 63, 63);
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239);
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Position {
	pub x: usize,
	pub y: usize,
}

struct StatusMessage {
	text: String,
	time: Instant,
}

impl StatusMessage {
	fn from(message: String) -> Self {
		Self {
			time: Instant::now(),
			text: message,
		}
	}
}

pub struct Editor {
	should_quit: bool,
	terminal: Terminal,
	cursor_position: Position,
	offset: Position,
	document: Document,
	status_message: StatusMessage,
}

impl Editor {
	pub fn run(&mut self) {
		loop {
			if let Err(error) = self.refresh_screen() {
				die(error);
			}
			if self.should_quit {
				break;
			}
			if let Err(error) = self.process_keypress() {
				die(error);
			}
		}
	}

	pub fn default() -> Self {
		// Get input filename
		let args: Vec<String> = env::args().collect();
		let mut initial_status = String::from("Ctrl-Q to quit");
		let document = if args.len() > 1 {
			let file_name = &args[1];
			let doc = Document::open(&file_name);
			if doc.is_ok() {
				doc.unwrap()
			} else {
				initial_status = format!("ERROR: Can't open {}", file_name);
				Document::default()
			}
		} else {
			Document::default()
		}; // ; here and not inside ensures doc is never undefined

		Self {
			should_quit: false,
			terminal: Terminal::default().expect("Failed to init terminal"), 
			document,	
			cursor_position: Position::default(),
			offset: Position::default(),
			status_message: StatusMessage::from(initial_status),
		}
	}

	/*
		Refresh with redraw or exit message
	*/
	fn refresh_screen(&self) -> Result<(), std::io::Error> {
		Terminal::cursor_hide();
		Terminal::cursor_position(&Position::default());

		if self.should_quit {
			println!("Goodbye.\r");
			Terminal::clear_screen();
		} else {
			self.draw_rows();
			self.draw_status_bar();
			self.draw_message_bar();
			Terminal::cursor_position(&Position {
				x: self.cursor_position.x.saturating_sub(self.offset.x),
				y: self.cursor_position.y.saturating_sub(self.offset.y),
			});
		}

		Terminal::cursor_show();
		Terminal::flush()
	}

	/*
		Read optional key input
	*/
	fn process_keypress(&mut self) -> Result<(), std::io::Error> {
		let pressed_key = Terminal::read_key()?;
		match pressed_key {
			Key::Ctrl('q') => self.should_quit = true,
			Key::Up 
			| Key:: Down
			| Key::Left 
			| Key::Right 
			| Key::PageUp
			| Key::PageDown
			| Key:: End
			| Key::Home => self.move_cursor(pressed_key),
			_ => (),
		}

		self.scroll();
		Ok(())
	}

	/* 
		Add scroll bump to position
	*/
	fn scroll(&mut self) {
		let Position { x, y } = self.cursor_position;
		let width = self.terminal.size().width as usize;
		let height = self.terminal.size().height as usize;
		let mut offset = &mut self.offset;

		if y < offset.y {
			offset.y = y;
		} else if y >= offset.y.saturating_add(height) {
			offset.y = y.saturating_sub(height).saturating_add(1);
		}
		if x < offset.x {
			offset.x = x;
		} else if x >= offset.x.saturating_add(width) {
			offset.x = x.saturating_sub(width).saturating_add(1);
		}
	}

	/*
		Handle cursor navigation
	*/
	fn move_cursor(&mut self, key: Key) {
		let Position { mut y, mut x } = self.cursor_position;
		let height = self.document.len();
		let mut width = if let Some(row) = self.document.row(y) {
			row.len()
		} else {
			0
		};
		
		match key {
			Key::Up => y = y.saturating_sub(1),
			Key::Down => {
				if y < height {
					y = y.saturating_add(1);
				}
			}
			Key::Left => {
				if x > 0 {
					x -= 1;
				} else if y > 0 {
					y -= 1;
					if let Some(row) = self.document.row(y) {
						x = row.len()
					} else {
						x = 0;
					}
				}
			}
			Key::Right => {
				if x < width {
					x += 1;
				} else if y < height {
					y += 1;
					x = 0;
				}
			}
			Key::PageUp => y = 0, // TODO: Sep paging and doc start/end
			Key::PageDown => y = height,
			Key::Home => x = 0,
			Key::End => x = width,
			_ => (),
		}

		// Snap scrolling to line ends
		width = if let Some(row) = self.document.row(y) {
			row.len()
		} else {
			0
		};
		if x > width {
			x = width;
		}
		
		self.cursor_position = Position { x, y }
	}

	/*
		Print centered welcome message
	*/
	fn draw_welcome_msg(&self) {
		let mut msg = format!("RustyVim -- version {}", VERSION);
		let width = self.terminal.size().width as usize;
		let len = msg.len();
		let padding = width.saturating_sub(len) / 2;
		let spaces = " ".repeat(padding.saturating_sub(1));

		msg = format!("~{}{}", spaces, msg);
		msg.truncate(width);
		println!("{}\r", msg);
	}

	/* 
		Draw document rows
	*/
	pub fn draw_row(&self, row: &Row) {
		let width = self.terminal.size().width as usize;
		let start = self.offset.x;
		let end = self.offset.x + width;
		let row = row.render(start, end);
		println!("{}\r", row)
	}
	
	/*
		Draw terminal row features
	*/
	fn draw_rows(&self) {
		let height = self.terminal.size().height;
		for terminal_row in 0..height {	
			Terminal::clear_current_line();
			let bump = self.offset.y;
			if let Some(row) = self.document.row(terminal_row as usize + bump) {
				self.draw_row(row);
			} else if self.document.is_empty() && terminal_row == height / 3 {
				self.draw_welcome_msg();
			} else {
				println!("~\r");
			}
		}
	}

	/*
		Draw colored status bar with info
	*/
	fn draw_status_bar(&self) {
		let mut status;
		let width = self.terminal.size().width as usize;
		let mut file_name = "[No Name]".to_string();

		// File status - name, len
		if let Some(name) = &self.document.file_name {
			file_name = name.clone();
			file_name.truncate(20);
		}
		status = format!("{} - {} lines", file_name, self.document.len());
		
		// Line indicator
		let line_indicator = format!(
			"{}/{}",
			self.cursor_position.y.saturating_add(1),
			self.document.len()
		);

		let len = status.len() + line_indicator.len();
		if width > len {
			status.push_str(&" ".repeat(width - len));
		}		
		status = format!("{}{}", status, line_indicator);
		status.truncate(width);

		Terminal::set_bg_color(STATUS_BG_COLOR);
		Terminal::set_fg_color(STATUS_FG_COLOR);
		println!("{}\r", status);
		Terminal::reset_fg_color();
		Terminal::reset_bg_color();
	}

	/*
		Draw bottom status bar
	*/
	fn draw_message_bar(&self) {
		Terminal::clear_current_line();
		let message = &self.status_message;
		if Instant::now() - message.time < Duration::new(5, 0) {
			let mut text = message.text.clone();
			text.truncate(self.terminal.size().width as usize);
			print!("{}", text);
		}
	}
}

fn die(e: std::io::Error) {
	Terminal::clear_screen();
	panic!(e);
} 
