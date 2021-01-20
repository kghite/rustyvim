use crate::Document;
use crate::Row;
use crate::Terminal;
use std::env;
use termion::event::Key;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Position {
	pub x: usize,
	pub y: usize,
}

pub struct Editor {
	should_quit: bool,
	terminal: Terminal,
	cursor_position: Position,
	offset: Position,
	document: Document,
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
		let document = if args.len() > 1 {
			let filename = &args[1];
			Document::open(&filename).unwrap_or_default()
		} else {
			Document::default()
		}; // ; here and not inside ensures doc is never undefined

		Self {
			should_quit: false,
			terminal: Terminal::default().expect("Failed to init terminal"), 
			document,	
			cursor_position: Position::default(),
			offset: Position::default(),
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
			Key::Left => x = x.saturating_sub(1),
			Key::Right => {
				if x < width {
					x = x.saturating_add(1);
				}
			}
			Key::PageUp => y = 0,
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
		for terminal_row in 0..height - 1 {	
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
}

fn die(e: std::io::Error) {
	Terminal::clear_screen();
	panic!(e);
} 
