#[macro_use]
extern crate crossterm;
extern crate termsize;
use crossterm::cursor;
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
pub use crossterm::style::Color;
use crossterm::style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{enable_raw_mode, Clear, ClearType};
use std::{collections::HashSet, io::stdout, process};

pub fn wait_for_input() {
    read().unwrap();
}

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
        ingnore_waringns: bool,
    ) -> Result<Self, String> {
        let new_line_count = title.matches('\n').count();
        if !ingnore_waringns {
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
            if 2 + new_line_count + options.len() > console_size.rows as usize {
                return Err(String::from("Menu will not fit on screen"));
            }
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
        format!("{}", self.options[index])
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
                    cursor::MoveToNextLine(1),
                    ResetColor
                )
                .unwrap();
                continue;
            }
            execute!(
                self.stdout,
                cursor::MoveRight(self.selector.len() as u16),
                Print(out),
                cursor::MoveToNextLine(1),
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
                            cursor::MoveToPreviousLine(1),
                            Clear(ClearType::CurrentLine),
                            SetForegroundColor(self.selected_foreground_color),
                            SetBackgroundColor(self.selected_background_color),
                            Print(self.selector),
                            Print(next_line_out),
                            ResetColor,
                            cursor::MoveToColumn(1)
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
                            cursor::MoveToNextLine(1),
                            Clear(ClearType::CurrentLine),
                            SetForegroundColor(self.selected_foreground_color),
                            SetBackgroundColor(self.selected_background_color),
                            Print(self.selector),
                            Print(next_line_out),
                            cursor::MoveToColumn(1),
                            ResetColor,
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

pub struct MultiMenu<'a> {
    title: &'a str,
    options: &'a Vec<&'a str>,
    selected_index: usize,
    stdout: std::io::Stdout,
    new_line_count: usize,
    selector: &'a str,
    selected_foreground_color: Color,
    selected_background_color: Color,
    selected_options: HashSet<usize>,
    selected_selector: &'a str,
    selected_option_foreground_color: Color,
    selected_option_background_color: Color,
    selected_selected_option_foreground_color: Color,
    selected_selected_option_background_color: Color,
}

impl<'a> MultiMenu<'a> {
    pub fn new(
        title: &'a str,
        options: &'a Vec<&'a str>,
        selected_index: usize,
        selector: &'a str,
        selected_options: HashSet<usize>,
        selected_selector: &'a str,
        ingnore_waringns: bool,
    ) -> Result<Self, String> {
        let new_line_count = title.matches('\n').count();
        if !ingnore_waringns {
            if selected_selector.len() != selector.len() {
                return Err(format!(
                    "selected_selector length must be equal to selector length"
                ));
            }
            if selected_index >= options.len() {
                return Err(format!(
                    "Selected option [{}] is out of range",
                    selected_index
                ));
            }
            for selected_option in selected_options.iter() {
                if *selected_option >= options.len() {
                    return Err(format!(
                        "Selected option [{}] is out of range",
                        selected_option
                    ));
                }
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
            if 2 + new_line_count + options.len() > console_size.rows as usize {
                return Err(String::from("Menu will not fit on screen"));
            }
        }
        Ok(MultiMenu {
            title,
            options,
            selected_index: selected_index,
            stdout: stdout(),
            new_line_count,
            selector,
            selected_foreground_color: Color::White,
            selected_background_color: Color::Black,
            selected_options,
            selected_selector,
            selected_option_foreground_color: Color::White,
            selected_option_background_color: Color::Black,
            selected_selected_option_foreground_color: Color::Black,
            selected_selected_option_background_color: Color::Black,
        })
    }
    pub fn set_selected_foreground_color(&mut self, color: Color) {
        self.selected_foreground_color = color;
    }

    pub fn set_selected_background_color(&mut self, color: Color) {
        self.selected_background_color = color;
    }

    pub fn set_selected_option_foreground_color(&mut self, color: Color) {
        self.selected_option_foreground_color = color;
    }

    pub fn set_selected_option_background_color(&mut self, color: Color) {
        self.selected_option_background_color = color;
    }

    pub fn set_selected_selected_option_foreground_color(&mut self, color: Color) {
        self.selected_selected_option_foreground_color = color;
    }

    pub fn set_selected_selected_option_background_color(&mut self, color: Color) {
        self.selected_selected_option_background_color = color;
    }

    fn format_option(&self, index: usize) -> String {
        format!("{}", self.options[index])
    }

    fn format_title(&self) -> String {
        format!("{}\n", self.title)
    }

    pub fn display(&mut self) {
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
                if self.selected_options.contains(&i) {
                    execute!(
                        self.stdout,
                        SetForegroundColor(self.selected_selected_option_foreground_color),
                        SetBackgroundColor(self.selected_selected_option_background_color),
                        Print(self.selector),
                        Print(out),
                        cursor::MoveToNextLine(1),
                        ResetColor
                    )
                    .unwrap();
                    continue;
                }
                execute!(
                    self.stdout,
                    SetForegroundColor(self.selected_foreground_color),
                    SetBackgroundColor(self.selected_background_color),
                    Print(self.selector),
                    Print(out),
                    cursor::MoveToNextLine(1),
                    ResetColor
                )
                .unwrap();
                continue;
            }
            if self.selected_options.contains(&i) {
                execute!(
                    self.stdout,
                    SetForegroundColor(self.selected_option_foreground_color),
                    SetBackgroundColor(self.selected_option_background_color),
                    Print(self.selected_selector),
                    Print(out),
                    cursor::MoveToNextLine(1),
                    ResetColor
                )
                .unwrap();
                continue;
            }
            execute!(
                self.stdout,
                cursor::MoveRight(self.selector.len() as u16),
                Print(out),
                cursor::MoveToNextLine(1),
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

    pub fn run(&mut self) -> Option<&HashSet<usize>> {
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

                        if self.selected_options.contains(&(self.selected_index + 1)) {
                            execute!(
                                self.stdout,
                                Clear(ClearType::CurrentLine),
                                SetForegroundColor(self.selected_option_foreground_color),
                                SetBackgroundColor(self.selected_option_background_color),
                                Print(self.selected_selector),
                                Print(current_line_out),
                                cursor::MoveToPreviousLine(1),
                                ResetColor
                            )
                            .unwrap();
                        } else {
                            execute!(
                                self.stdout,
                                Clear(ClearType::CurrentLine),
                                cursor::MoveRight(self.selector.len() as u16),
                                Print(current_line_out),
                                cursor::MoveToPreviousLine(1)
                            )
                            .unwrap();
                        }
                        if self.selected_options.contains(&self.selected_index) {
                            execute!(
                                self.stdout,
                                Clear(ClearType::CurrentLine),
                                SetForegroundColor(self.selected_selected_option_foreground_color),
                                SetBackgroundColor(self.selected_selected_option_background_color),
                                Print(self.selector),
                                Print(next_line_out),
                                ResetColor,
                                cursor::MoveToColumn(1)
                            )
                            .unwrap();
                        } else {
                            execute!(
                                self.stdout,
                                SetForegroundColor(self.selected_foreground_color),
                                SetBackgroundColor(self.selected_background_color),
                                Print(self.selector),
                                Print(next_line_out),
                                ResetColor,
                                cursor::MoveToColumn(1)
                            )
                            .unwrap();
                        }
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
                        if self.selected_options.contains(&(self.selected_index - 1)) {
                            execute!(
                                self.stdout,
                                Clear(ClearType::CurrentLine),
                                SetForegroundColor(self.selected_option_foreground_color),
                                SetBackgroundColor(self.selected_option_background_color),
                                Print(self.selected_selector),
                                Print(current_line_out),
                                cursor::MoveToNextLine(1),
                                ResetColor
                            )
                            .unwrap();
                        } else {
                            execute!(
                                self.stdout,
                                Clear(ClearType::CurrentLine),
                                cursor::MoveRight(self.selector.len() as u16),
                                Print(current_line_out),
                                cursor::MoveToNextLine(1)
                            )
                            .unwrap();
                        }
                        if self.selected_options.contains(&self.selected_index) {
                            execute!(
                                self.stdout,
                                Clear(ClearType::CurrentLine),
                                SetForegroundColor(self.selected_selected_option_foreground_color),
                                SetBackgroundColor(self.selected_selected_option_background_color),
                                Print(self.selector),
                                Print(next_line_out),
                                ResetColor,
                                cursor::MoveToColumn(1)
                            )
                            .unwrap();
                        } else {
                            execute!(
                                self.stdout,
                                SetForegroundColor(self.selected_foreground_color),
                                SetBackgroundColor(self.selected_background_color),
                                Print(self.selector),
                                Print(next_line_out),
                                ResetColor,
                                cursor::MoveToColumn(1)
                            )
                            .unwrap();
                        }
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char(' '),
                    modifiers: KeyModifiers::NONE,
                }) => {
                    let out = self.format_option(self.selected_index);
                    if self.selected_options.contains(&self.selected_index) {
                        execute!(
                            self.stdout,
                            Clear(ClearType::CurrentLine),
                            SetForegroundColor(self.selected_foreground_color),
                            SetBackgroundColor(self.selected_background_color),
                            Print(self.selector),
                            Print(out),
                            ResetColor,
                            cursor::MoveToColumn(1)
                        )
                        .unwrap();
                        self.selected_options.remove(&self.selected_index);
                    } else {
                        execute!(
                            self.stdout,
                            Clear(ClearType::CurrentLine),
                            SetForegroundColor(self.selected_selected_option_foreground_color),
                            SetBackgroundColor(self.selected_selected_option_background_color),
                            Print(self.selector),
                            Print(out),
                            ResetColor,
                            cursor::MoveToColumn(1)
                        )
                        .unwrap();
                        self.selected_options.insert(self.selected_index);
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
        Some(&self.selected_options)
    }
}
