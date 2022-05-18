#[macro_use]
extern crate crossterm;
extern crate termsize;
pub use crossterm::style::Color;
use crossterm::cursor;
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{enable_raw_mode, Clear, ClearType};
use std::{io::stdout, process};


pub struct Menu<'a> {
    title: &'a str,
    options: &'a Vec<&'a str>,
    selected_index: usize,
    stdout: std::io::Stdout,
    new_line_count: usize,
    selector: &'a str,
    selected_foreground_color: Color,
    selected_background_color: Color,
}

impl<'a> Menu<'a> {
    pub fn new(
        title: &'a str,
        options: &'a Vec<&'a str>,
        selected_index: usize,
        selector: &'a str,
    ) -> Result<Self, String> {
        if selected_index >= options.len() {
            return Err(format!(
                "Selected option [{}] is out of range",
                selected_index
            ));
        }

        let console_size = termsize::get().unwrap();
        if title.len() > console_size.cols as usize {
            return Err(String::from("Title is too long"));
        }
        for (i, option) in options.iter().enumerate() {
            if option.len() + selector.len() > console_size.cols as usize {
                return Err(format!("Option [{}] is too long", i));
            }
            if option.contains('\n') {
                return Err(format!("Option [{}] contains new line", i));
            }
            if option.contains('\t') {
                return Err(format!("Option [{}] contains tab", i));
            }
        }
        let new_line_count = title.matches('\n').count();
        if 2 + new_line_count + options.len() > console_size.rows as usize {
            return Err(String::from("Menu will not fit on screen"));
        }
        Ok(Menu {
            title,
            options,
            selected_index: selected_index,
            stdout: stdout(),
            new_line_count,
            selector,
            selected_foreground_color: Color::White,
            selected_background_color: Color::Black,
        })
    }
    pub fn set_selected_foreground_color(&mut self, color: Color) {
        self.selected_foreground_color = color;
    }

    pub fn set_selected_background_color(&mut self, color: Color) {
        self.selected_background_color = color;
    }

    fn format_option(&self, index: usize) -> String {
        format!("{}\n", self.options[index])
    }

    fn format_title(&self) -> String {
        format!("{}\n", self.title)
    }

    fn display(&mut self) {
        let out = self.format_title();
        execute!(
            self.stdout,
            Clear(ClearType::All),
            cursor::MoveTo(0, 0),
            Print(out)
        )
        .unwrap();
        for i in 0..self.options.len() {
            let out = self.format_option(i);
            if i == self.selected_index {
                execute!(
                    self.stdout,
                    SetForegroundColor(self.selected_foreground_color),
                    SetBackgroundColor(self.selected_background_color),
                    Print(self.selector),
                    Print(out),
                    ResetColor
                )
                .unwrap();
                continue;
            }
            execute!(
                self.stdout,
                cursor::MoveRight(self.selector.len() as u16),
                Print(out)
            )
            .unwrap();
        }
        execute!(
            self.stdout,
            cursor::MoveTo(
                0,
                self.selected_index as u16 + self.new_line_count as u16 + 1
            )
        )
        .unwrap();
    }

    pub fn restore_console(&mut self) {
        execute!(
            self.stdout,
            Clear(ClearType::All),
            cursor::MoveTo(0, 0),
            cursor::Show,
        )
        .unwrap();
    }

    pub fn run(&mut self) -> Option<usize> {
        enable_raw_mode().unwrap();
        execute!(self.stdout, cursor::Hide).unwrap();
        self.display();
        loop {
            match read().unwrap() {
                Event::Key(KeyEvent {
                    code: KeyCode::Up,
                    modifiers: KeyModifiers::NONE,
                }) => {
                    if self.selected_index > 0 {
                        let current_line_out = self.format_option(self.selected_index);
                        self.selected_index -= 1;
                        let next_line_out = self.format_option(self.selected_index);
                        execute!(
                            self.stdout,
                            Clear(ClearType::CurrentLine),
                            cursor::MoveRight(self.selector.len() as u16),
                            Print(current_line_out),
                            cursor::MoveToPreviousLine(2),
                            Clear(ClearType::CurrentLine),
                            SetForegroundColor(self.selected_foreground_color),
                            SetBackgroundColor(self.selected_background_color),
                            Print(self.selector),
                            Print(next_line_out),
                            ResetColor,
                            cursor::MoveToPreviousLine(1)
                        )
                        .unwrap();
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::NONE,
                }) => {
                    if self.selected_index < self.options.len() - 1 {
                        let current_line_out = self.format_option(self.selected_index);
                        self.selected_index += 1;
                        let next_line_out = self.format_option(self.selected_index);
                        execute!(
                            self.stdout,
                            Clear(ClearType::CurrentLine),
                            cursor::MoveRight(self.selector.len() as u16),
                            Print(current_line_out),
                            Clear(ClearType::CurrentLine),
                            SetForegroundColor(self.selected_foreground_color),
                            SetBackgroundColor(self.selected_background_color),
                            Print(self.selector),
                            Print(next_line_out),
                            ResetColor,
                            cursor::MoveToPreviousLine(1)
                        )
                        .unwrap();
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                }) => break,
                Event::Key(KeyEvent {
                    code: KeyCode::Esc,
                    modifiers: KeyModifiers::NONE,
                }) => {
                    self.restore_console();
                    return None;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                }) => {
                    self.restore_console();
                    process::exit(0);
                }
                _ => continue,
            }
        }
        self.restore_console();
        Some(self.selected_index)
    }
}
