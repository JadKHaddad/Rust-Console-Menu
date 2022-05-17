#[macro_use]
extern crate crossterm;
extern crate termsize;

use crossterm::cursor;
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use std::io::stdout;

pub struct Menu<'a> {
    title: &'a str,
    options: &'a Vec<&'a str>,
    selected_index: usize,
    stdout: std::io::Stdout,
    title_vertical_offset: usize,
    title_horizontal_offset: usize,
    new_line_count: usize,
    selector: &'a str,
    selector_offset_left: usize,
    selector_offset_right: usize,
}

impl<'a> Menu<'a> {
    pub fn new(
        title: &'a str,
        options: &'a Vec<&'a str>,
        title_vertical_offset: usize,
        title_horizontal_offset: usize,
        selected_index: usize,
        selector: &'a str,
        selector_offset_left: usize,
        selector_offset_right: usize,
    ) -> Result<Self, String> {
        if selected_index >= options.len() {
            return Err(format!(
                "Selected option [{}] is out of range",
                selected_index
            ));
        }

        let console_size = termsize::get().unwrap();
        if title.len() + title_horizontal_offset > console_size.cols as usize {
            return Err(String::from("Title is too long"));
        }
        for (i, option) in options.iter().enumerate() {
            if option.len() + selector.len() + selector_offset_left + selector_offset_right
                > console_size.cols as usize
            {
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
        if 2 + new_line_count + title_vertical_offset + options.len() > console_size.rows as usize {
            return Err(String::from("Menu will not fit on screen"));
        }
        Ok(Menu {
            title,
            options,
            selected_index: selected_index,
            stdout: stdout(),
            title_vertical_offset,
            title_horizontal_offset,
            new_line_count,
            selector,
            selector_offset_left,
            selector_offset_right,
        })
    }

    fn format_option(&self, index: usize) -> String {
        format!("{}\n", self.options[index])
    }

    fn format_title(&self) -> String {
        let mut title = format!("{}\n", self.title);
        for _ in 0..self.title_vertical_offset {
            title.push('\n');
        }
        title
    }

    pub fn display(&mut self) {
        let out = self.format_title();
        execute!(
            self.stdout,
            Clear(ClearType::All),
            cursor::MoveTo(self.title_horizontal_offset as u16, 0),
            Print(out)
        )
        .unwrap();
        for i in 0..self.options.len() {
            let out = self.format_option(i);
            if i == self.selected_index {
                execute!(
                    self.stdout,
                    cursor::MoveRight(self.selector_offset_left as u16),
                    SetForegroundColor(Color::Blue),
                    Print(self.selector),
                    cursor::MoveRight(self.selector_offset_right as u16),
                    Print(out),
                    ResetColor
                )
                .unwrap();
                continue;
            }
            execute!(
                self.stdout,
                cursor::MoveRight(
                    self.selector_offset_left as u16
                        + self.selector_offset_right as u16
                        + self.selector.len() as u16
                ),
                Print(out)
            )
            .unwrap();
        }
        execute!(
            self.stdout,
            cursor::MoveTo(
                0,
                self.selected_index as u16
                    + self.new_line_count as u16
                    + self.title_vertical_offset as u16
                    + 1
            )
        )
        .unwrap();
    }

    pub fn run(&mut self) {
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
                            cursor::MoveRight(
                                self.selector_offset_left as u16
                                    + self.selector_offset_right as u16
                                    + self.selector.len() as u16
                            ),
                            Print(current_line_out),
                            cursor::MoveToPreviousLine(2),
                            Clear(ClearType::CurrentLine),
                            cursor::MoveRight(self.selector_offset_left as u16),
                            SetForegroundColor(Color::Blue),
                            Print(self.selector),
                            cursor::MoveRight(self.selector_offset_right as u16),
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
                            cursor::MoveRight(
                                self.selector_offset_left as u16
                                    + self.selector_offset_right as u16
                                    + self.selector.len() as u16
                            ),
                            Print(current_line_out),
                            Clear(ClearType::CurrentLine),
                            cursor::MoveRight(self.selector_offset_left as u16),
                            SetForegroundColor(Color::Blue),
                            Print(self.selector),
                            cursor::MoveRight(self.selector_offset_right as u16),
                            Print(next_line_out),
                            ResetColor,
                            cursor::MoveToPreviousLine(1)
                        )
                        .unwrap();
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Esc,
                    modifiers: KeyModifiers::NONE,
                    //clearing the screen and printing our message
                }) => break,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                }) => break,
                _ => continue,
            }
        }
        execute!(
            self.stdout,
            Clear(ClearType::All),
            cursor::MoveTo(0, 0),
            cursor::Show,
        )
        .unwrap();
    }
}
