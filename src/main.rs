#![allow(dead_code)]
#![allow(unused_variables)]
use std::{io::{stdout, Read, Stdout, Write}, thread::sleep, time::Duration, vec};

use crossterm::{
    cursor, 
    event::{self, KeyCode}, 
    queue, terminal::{self, WindowSize}, ExecutableCommand, QueueableCommand};

// struct PageBuffer {
//     original: Vec<u8>,
//     append: Vec<u8>,
//     pages: Vec<Page>,
//     rowlen: usize,
// }

// enum PageType {
//     Original,
//     Append
// }

// struct Page {
//     source: PageType,
//     source_start: usize,
//     source_end: usize,
//     start: usize,
//     end: usize
// }

// impl PageBuffer {
//     fn write_out(&self) -> String {
//         let output: Vec<u8> = Vec::with_capacity(2048);

//         for page in self.pages {
//             match page.source {
//                 PageType::Original => (),
//                 PageType::Append => ()
//             }
//         }

//         "asd".to_string()
//     }

//     fn write(&self, data: Vec<u8>, cursor_pos: (u16, u16)) {
//         // is cursor inside original buffer?
//         if cursor_pos.0 as usize + (cursor_pos.1 as usize * self.rowlen) < self.original.len() {

//         }

//         // is cursor at end of buffer?
//         let page = Page {
//             source: PageType::Append,
//             source_start: self.append.len(),
//             source_end: self.append.len() + data.len(),
//             start: cursor_pos.0 as usize,
//             end: (cursor_pos.0 as usize + data.len())
//         };
//         self.pages.push(page);

//         self.append.extend(data);
//     }
// }



struct Row {
    buffer: Vec<u8>,
    cursor_pos: usize,
    row_len: usize
}

impl Row {
    fn new(row_size: usize) -> Self {
        Row {
            buffer: vec![0; row_size],
            cursor_pos: 0,
            row_len: 0
        }
    }
}


struct TerminalSize {
    columns: u16,
    rows: u16
}

impl Into<(u16, u16)> for TerminalSize {
    fn into(self) -> (u16, u16) {
        (self.columns, self.rows)
    }
}

impl From<(u16, u16)> for TerminalSize {
    fn from(value: (u16, u16)) -> Self {
        TerminalSize {
            columns: value.0,
            rows: value.1 
        }    
    }
}

struct CursorPos {
    column: u16,
    row: u16
}

impl Into<(u16, u16)> for CursorPos {
    fn into(self) -> (u16, u16) {
        (self.column, self.row)
    }
}

impl From<(u16, u16)> for CursorPos {
    fn from(value: (u16, u16)) -> Self {
        CursorPos {
            column: value.0,
            row: value.1
        }
    }
}

struct TextEditor {
    cursor_pos: CursorPos,
    initial_pos: CursorPos,
    row_length: u16,
    window_size: TerminalSize,
    buffer: Vec<Row>,
    out: Stdout,
    mode: Mode
}
enum Mode {
    Edit,
    Action
}


impl TextEditor {
    fn default() -> Self {
        return TextEditor {
            cursor_pos: CursorPos {column: 0, row: 0},
            initial_pos: CursorPos {column: 0, row: 0},
            row_length: 35,
            window_size: terminal::size().unwrap().into(),
            buffer: vec![Row::new(35)],
            out: stdout(),
            mode: Mode::Edit,
        }
    }
    
    fn run(&mut self) {
    
        queue!(self.out, terminal::EnterAlternateScreen).unwrap();
        self.out.flush().unwrap();
        
        loop {
            self.out.queue(cursor::MoveTo(
                self.initial_pos.column, 
                self.initial_pos.row
            )).unwrap();
            
            self.out.queue(
                terminal::Clear(terminal::ClearType::FromCursorDown)
            ).unwrap();

            self.out.flush().unwrap();

            for row in self.buffer.as_slice() {
                self.out.write(&row.buffer).unwrap();
                self.out.write(&[b'\n']).unwrap();
            }

            self.out.flush().unwrap();
            
            
            self.out.queue(cursor::MoveTo(0, 48)).unwrap();
            
            let buf = match self.mode {
                Mode::Action => {b"Mode: Action"},
                Mode::Edit => {b"Mode: Edit  "}
            };

            self.out.write(buf).unwrap();

            self.out.queue(cursor::MoveTo(self.cursor_pos.column, self.cursor_pos.row)).unwrap();

            let _ = self.out.flush();

            match event::read().unwrap() {
                event::Event::FocusGained => {},
                event::Event::FocusLost => {},
                event::Event::Key(k) => {
                    match k.kind {
                        event::KeyEventKind::Press => {self.key_press(k.code)},
                        event::KeyEventKind::Release => {},
                        event::KeyEventKind::Repeat => {}
                    }
                },
                event::Event::Mouse(m) => {},
                event::Event::Paste(p) => {},
                event::Event::Resize(x, y) => {}
            }
            // println!("{:?}", self.buffer);            
        }
    }

    fn key_press(&mut self, k: KeyCode) {
        match k {
            KeyCode::BackTab => {},
            KeyCode::Backspace => {
                                
                if (self.cursor_pos.column > 0) && (self.buffer[self.cursor_pos.row as usize].row_len > 0) {
                    self.cursor_pos.column -= 1;
                    self.buffer[self.cursor_pos.row as usize].buffer[self.cursor_pos.column as usize] = 0;
                    self.buffer[self.cursor_pos.row as usize].cursor_pos -= 1;
                    self.buffer[self.cursor_pos.row as usize].row_len -= 1;
                }

                if self.cursor_pos.column == 0 && (self.cursor_pos.row - self.initial_pos.row) > 0 {
                    self.buffer[self.cursor_pos.row as usize].buffer[0] = 0;
                    self.buffer[self.cursor_pos.row as usize].buffer[1] = 0;
                    self.cursor_pos.row -= 1;
                    self.buffer[self.cursor_pos.row as usize].row_len -= 1;
                    self.cursor_pos.column = self.buffer[self.cursor_pos.row as usize].cursor_pos as u16;
                }
            },
            KeyCode::CapsLock => {},
            KeyCode::Char(c) => {
                match self.mode {
                    Mode::Edit => {
                        println!("{}", c);

                        if self.buffer[self.cursor_pos.row as usize].row_len < self.row_length as usize{

                            // split and insert
                            if self.cursor_pos.column < self.buffer[self.cursor_pos.row as usize].buffer.len() as u16 {
                                let (part1, part2) = self.buffer[self.cursor_pos.row as usize].buffer.split_at(self.cursor_pos.column as usize);

                                let mut new_row_buffer = part1.to_vec();
                                new_row_buffer.push(c as u8);
                                new_row_buffer.extend(part2.to_vec());

                                self.buffer[self.cursor_pos.row as usize].buffer = new_row_buffer;
                            } else {
                                self.buffer[self.cursor_pos.row as usize].buffer[self.cursor_pos.column as usize] = c.try_into().unwrap();
                            }
                            
                            self.cursor_pos.column += 1;
                            self.buffer[self.cursor_pos.row as usize].cursor_pos += 1;
                            self.buffer[self.cursor_pos.row as usize].row_len += 1;
                        }
                    }
                    Mode::Action => {
                        match c {
                            'q' => {
                                self.out.execute(terminal::LeaveAlternateScreen).unwrap();
                            }
                            'w' => {}
                            _ => {
                                self.out.write(&[c as u8]).unwrap();
                                self.out.flush().unwrap();
                                sleep(Duration::from_secs(1));
                            }
                        }
                    }
               }
            }
            KeyCode::Delete => {},
            KeyCode::Down => {
                if (self.cursor_pos.row as usize) < self.buffer.len() -1 {
                    self.cursor_pos.row += 1;
                    self.cursor_pos.column = self.buffer[self.cursor_pos.row as usize].cursor_pos as u16;
                }
            },
            KeyCode::Left => {
                if self.cursor_pos.column > 0 {
                    self.cursor_pos.column -= 1;
                }
            },
            KeyCode::Up => {
                if self.cursor_pos.row > 0 {
                    self.cursor_pos.row -= 1;
                    if self.cursor_pos.column > self.buffer[self.cursor_pos.row as usize].cursor_pos as u16 {
                        self.cursor_pos.column = self.buffer[self.cursor_pos.row as usize].cursor_pos as u16
                    }
                }
            },
            KeyCode::Right => {
                if self.cursor_pos.column < self.buffer[self.cursor_pos.row as usize].cursor_pos as u16
                {
                    self.cursor_pos.column += 1;
                }
            },
            KeyCode::End => {},
            KeyCode::Enter => {
                let buffer_pos = self.cursor_buffer_pos();
                // self.buffer[(self.cursor_pos.row)  as usize][self.cursor_pos.column as usize] = b'\n';
                
                // if self.cursor_pos.row == self.buffer.len() as u16 {
                self.buffer.push(Row::new(self.row_length as usize));
                self.cursor_pos.row += 1;
                // }
                // self.cursor_pos.row -= 1;
                // self.initial_pos.row -= 1;
                self.cursor_pos.column = 0;
            },
            KeyCode::Esc => {
                self.mode = match self.mode {
                    Mode::Edit => Mode::Action,
                    Mode::Action => Mode::Edit 
                }
            },
            KeyCode::F(f) => {},
            KeyCode::Home => {},
            KeyCode::Insert => {},
            KeyCode::KeypadBegin => {},
            KeyCode::Media(m) => {},
            KeyCode::Menu => {},
            KeyCode::Modifier(m) => {},
            KeyCode::Null => {},
            KeyCode::NumLock => {},
            KeyCode::PageDown => {},
            KeyCode::PageUp => {},
            KeyCode::Pause => {},
            KeyCode::PrintScreen => {},
            KeyCode::ScrollLock => {},
            KeyCode::Tab => {},
        }
    }

    fn cursor_buffer_pos(&self) -> usize {
        let delta = CursorPos {
            column: self.cursor_pos.column - self.initial_pos.column, 
            row: self.cursor_pos.row - self.initial_pos.row
        };
        // println!("{:?}", delta);
        (delta.column + delta.row*self.row_length) as usize
    }
}

fn main() {
    let mut text_editor = TextEditor::default();

    text_editor.run();

    // let mut stdout = stdout();

    // queue!(stdout, terminal::EnterAlternateScreen).unwrap();
    // stdout.flush().unwrap();
    // stdout.write(b"this is a test").unwrap();
    // stdout.flush().unwrap();
    // sleep(Duration::from_secs(4));

    // queue!(stdout, terminal::LeaveAlternateScreen).unwrap();
    // stdout.flush().unwrap();
    // let mut buffer = [0;1024];
    // let mut stdin = stdin();

    // for k in stdin().bytes() {
    //     stdout.write(&[k.unwrap()]).unwrap();
    // }
}