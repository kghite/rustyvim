use crate::Terminal;
use termion::event::Key;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Position {
	pub x: usize,
	pub y: usize,
}

pub struct Editor {
	should_quit: bool,
	terminal: Terminal,
	cursor_position: Position,
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
		Self {
			should_quit: false,
			terminal: Terminal::default().expect("Failed to init terminal"), 
			cursor_position: Position { x: 0, y: 0 },
		}
	}

	/*
		Refresh with redraw or exit message
	*/
	fn refresh_screen(&self) -> Result<(), std::io::Error> {
		Terminal::cursor_hide();
		Terminal::cursor_position(&Position { x: 0, y: 0 });

		if self.should_quit {
			println!("Goodbye.\r");
			Terminal::clear_screen();
		} else {
			self.draw_rows();
			Terminal::cursor_position(&self.cursor_position);
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

		Ok(())
	}

	/*
		Handle cursor navigation
	*/
	fn move_cursor(&mut self, key: Key) {
		let Position { mut y, mut x } = self.cursor_position;
		let size = self.terminal.size();
		let height = size.height.saturating_sub(1) as usize;
		let width = size.width.saturating_sub(1) as usize;
		
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
		Draw margin features
	*/
	fn draw_rows(&self) {
		let height = self.terminal.size().height;
		for row in 0..height - 1 {	
			Terminal::clear_current_line();
			if row == height / 3 {
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
