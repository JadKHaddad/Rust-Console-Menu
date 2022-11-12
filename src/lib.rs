#[macro_use]
extern crate crossterm;
pub use crossterm::style::Color;
use crossterm::{
    cursor,
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use std::error::Error as StdError;
use std::fmt::Display as StdDisplay;
use std::{collections::HashSet, io::stdout, process};

#[derive(Debug)]
pub enum MenuError {
    IndexOutOfBounds,
}

impl StdDisplay for MenuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MenuError::IndexOutOfBounds => write!(f, "Index out of bounds"),
        }
    }
}

impl StdError for MenuError {
    fn description(&self) -> &str {
        match self {
            MenuError::IndexOutOfBounds => "Index out of bounds",
        }
    }
}

pub trait MenuLike<'a, T>
where
    T: StdDisplay + 'a,
    Self: Sized + Default,
{
    fn get_menu_mut(&mut self) -> &mut Menu<T>;

    fn get_menu(&self) -> &Menu<T>;

    fn new() -> Self {
        Default::default()
    }

    fn title(mut self, title: String) -> Self {
        let mut_menu = self.get_menu_mut();
        mut_menu.title = title;
        mut_menu.new_line_count = mut_menu.title.matches('\n').count();
        self
    }

    fn options(mut self, options: Vec<T>) -> Self {
        let mut_menu = self.get_menu_mut();
        mut_menu.options = options;
        self
    }

    fn selected_index(mut self, selected_index: usize) -> Result<Self, MenuError> {
        let mut_menu = self.get_menu_mut();
        if selected_index >= mut_menu.options.len() {
            return Err(MenuError::IndexOutOfBounds);
        }
        mut_menu.selected_index = selected_index;
        Ok(self)
    }

    fn selector(mut self, selector: String) -> Self {
        let mut_menu = self.get_menu_mut();
        mut_menu.selector = selector;
        self
    }

    fn outer_spacing(mut self, outer_spacing: u16) -> Self {
        let mut_menu = self.get_menu_mut();
        mut_menu.outer_spacing = outer_spacing;
        let mut outer = " ".repeat(mut_menu.outer_spacing as usize);
        outer.push_str(&mut_menu.selector);
        mut_menu.selector = outer;
        self
    }

    fn inner_spacing(mut self, inner_spacing: u16) -> Self {
        let mut_menu = self.get_menu_mut();
        mut_menu.inner_spacing = inner_spacing;
        mut_menu
            .selector
            .push_str(&" ".repeat(inner_spacing as usize));
        self
    }

    fn selected_foreground_color(mut self, color: Color) -> Self {
        let mut_menu = self.get_menu_mut();
        mut_menu.selected_foreground_color = color;
        self
    }

    fn selected_background_color(mut self, color: Color) -> Self {
        let mut_menu = self.get_menu_mut();
        mut_menu.selected_background_color = color;
        self
    }

    fn get_title(&'a self) -> &String {
        let menu = self.get_menu();
        &menu.title
    }

    fn get_options(&self) -> &Vec<T> {
        let menu = self.get_menu();
        &menu.options
    }

    fn format_option(&self, index: usize) -> String {
        let menu = self.get_menu();
        format!("{}", menu.options[index])
    }

    fn format_title(&self) -> String {
        let menu = self.get_menu();
        format!("{}\n", menu.title)
    }

    fn restore_console(&mut self) -> Result<(), Box<dyn StdError>> {
        let mut_menu = self.get_menu_mut();
        disable_raw_mode()?;
        let dist = mut_menu.options.len() as u16 - mut_menu.selected_index as u16;
        execute!(
            mut_menu.stdout,
            cursor::MoveToNextLine(dist),
            cursor::Show,
        )?;
        Ok(())
    }

    fn setup_console(&mut self) -> Result<(), Box<dyn StdError>> {
        let mut_menu = self.get_menu_mut();
        enable_raw_mode()?;
        execute!(mut_menu.stdout, cursor::Hide)?;
        Ok(())
    }

    fn display(&mut self) -> Result<(), Box<dyn StdError>> {
        let mut_menu = self.get_menu_mut();

        let title = mut_menu.format_title();
        print!("{}", title);
        for i in 0..mut_menu.options.len() {
            let option = mut_menu.format_option(i);
            if i == mut_menu.selected_index {
                let selector = &mut_menu.selector;
                let selected_foreground_color = mut_menu.selected_foreground_color;
                let selected_background_color = mut_menu.selected_background_color;
                execute!(
                    mut_menu.stdout,
                    SetForegroundColor(selected_foreground_color),
                    SetBackgroundColor(selected_background_color),
                    Print(selector),
                    Print(option),
                    ResetColor,
                    Print("\n"),
                )?;
                continue;
            }
            let dist = mut_menu.selector.len() as u16;
            execute!(
                mut_menu.stdout,
                cursor::MoveRight(dist),
                Print(option),
                Print("\n"),
            )?;
        }
        let dist = (mut_menu.options.len() - mut_menu.selected_index) as u16;
        execute!(mut_menu.stdout, cursor::MoveToPreviousLine(dist),)?;
        Ok(())
    }

    fn run(&mut self) -> Result<Option<usize>, Box<dyn StdError>> {
        let mut_menu = self.get_menu_mut();

        mut_menu.setup_console()?;
        mut_menu.display()?;
        let selector = &mut_menu.selector;
        let selected_foreground_color = mut_menu.selected_foreground_color;
        let selected_background_color = mut_menu.selected_background_color;
        let dist = mut_menu.selector.len() as u16;
        loop {
            match read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Up,
                    modifiers: KeyModifiers::NONE,
                }) => {
                    if mut_menu.selected_index > 0 {
                        let current_line_out = mut_menu.format_option(mut_menu.selected_index);
                        mut_menu.selected_index -= 1;
                        let next_line_out = mut_menu.format_option(mut_menu.selected_index);
                        execute!(
                            mut_menu.stdout,
                            Clear(ClearType::CurrentLine),
                            cursor::MoveRight(dist),
                            Print(current_line_out),
                            cursor::MoveToPreviousLine(1),
                            Clear(ClearType::CurrentLine),
                            SetForegroundColor(selected_foreground_color),
                            SetBackgroundColor(selected_background_color),
                            Print(selector),
                            Print(next_line_out),
                            cursor::MoveToColumn(1),
                            ResetColor,
                        )?;
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::NONE,
                }) => {
                    if mut_menu.selected_index < mut_menu.options.len() - 1 {
                        let current_line_out =
                            mut_menu.format_option(mut_menu.get_menu().selected_index);
                        mut_menu.selected_index += 1;
                        let next_line_out =
                            mut_menu.format_option(mut_menu.get_menu().selected_index);
                        execute!(
                            mut_menu.stdout,
                            Clear(ClearType::CurrentLine),
                            cursor::MoveRight(dist),
                            Print(current_line_out),
                            cursor::MoveToNextLine(1),
                            Clear(ClearType::CurrentLine),
                            SetForegroundColor(selected_foreground_color),
                            SetBackgroundColor(selected_background_color),
                            Print(selector),
                            Print(next_line_out),
                            cursor::MoveToColumn(1),
                            ResetColor,
                        )?;
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
                    mut_menu.restore_console()?;
                    return Ok(None);
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                }) => {
                    mut_menu.restore_console()?;
                    process::exit(1);
                }
                _ => continue,
            }
        }
        mut_menu.restore_console()?;
        Ok(Some(mut_menu.get_menu().selected_index))
    }

    fn wait_for_input() -> Result<(), Box<dyn StdError>> {
        read()?;
        Ok(())
    }
}

impl<'a, T> MenuLike<'a, T> for Menu<T>
where
    T: StdDisplay + 'a,
    Self: Sized,
{
    fn get_menu_mut(&mut self) -> &mut Menu<T> {
        self
    }

    fn get_menu(&self) -> &Menu<T> {
        self
    }
}
pub struct Menu<T>
where
    T: StdDisplay,
{
    title: String,
    options: Vec<T>,
    selected_index: usize,
    stdout: std::io::Stdout,
    new_line_count: usize,
    selector: String,
    outer_spacing: u16,
    inner_spacing: u16,
    selected_foreground_color: Color,
    selected_background_color: Color,
}

impl<T> Default for Menu<T>
where
    T: StdDisplay,
{
    fn default() -> Self {
        Self {
            stdout: stdout(),
            title: String::from("Title"),
            options: Vec::new(),
            selected_index: 0,
            new_line_count: 0,
            selector: String::from("=>"),
            outer_spacing: 0,
            inner_spacing: 0,
            selected_foreground_color: Color::Reset,
            selected_background_color: Color::Reset,
        }
    }
}

pub struct MultiMenu<T>
where
    T: StdDisplay,
{
    title: String,
    options: Vec<T>,
    selected_index: usize,
    selected_options: HashSet<usize>,
    stdout: std::io::Stdout,
    new_line_count: usize,
    selector: String,
    selected_selector: String,
    outer_spacing: u16,
    inner_spacing: u16,
    selected_foreground_color: Color,
    selected_background_color: Color,
    selected_option_foreground_color: Color,
    selected_option_background_color: Color,
    selected_selected_option_foreground_color: Color,
    selected_selected_option_background_color: Color,
}
impl<T> Default for MultiMenu<T>
where
    T: StdDisplay,
{
    fn default() -> Self {
        Self {
            stdout: stdout(),
            title: String::from("Title"),
            options: Vec::new(),
            selected_index: 0,
            selected_options: HashSet::new(),
            new_line_count: 0,
            selector: String::from("=>"),
            selected_selector: String::from("->"),
            outer_spacing: 0,
            inner_spacing: 0,
            selected_foreground_color: Color::Reset,
            selected_background_color: Color::Reset,
            selected_option_foreground_color: Color::Reset,
            selected_option_background_color: Color::Reset,
            selected_selected_option_foreground_color: Color::Reset,
            selected_selected_option_background_color: Color::Reset,
        }
    }
}
impl<T> MultiMenu<T>
where
    T: StdDisplay,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: String) -> Self {
        self.title = title;
        self.new_line_count = self.title.matches('\n').count();
        self
    }

    pub fn options(mut self, options: Vec<T>) -> Self {
        self.options = options;
        self
    }

    pub fn selected_index(mut self, selected_index: usize) -> Result<Self, MenuError> {
        if selected_index >= self.options.len() {
            return Err(MenuError::IndexOutOfBounds);
        }
        self.selected_index = selected_index;
        Ok(self)
    }

    //
    pub fn selected_options(mut self, selected_options: HashSet<usize>) -> Result<Self, MenuError> {
        for index in selected_options.iter() {
            if *index >= self.options.len() {
                return Err(MenuError::IndexOutOfBounds);
            }
        }
        self.selected_options = selected_options;
        Ok(self)
    }

    pub fn selector(mut self, selector: String) -> Self {
        self.selector = selector;
        self
    }

    // //TODO selector and selected_selector must be the same length
    pub fn selected_selector(mut self, selected_selector: String) -> Self {
        self.selected_selector = selected_selector;
        self
    }

    //
    pub fn outer_spacing(mut self, outer_spacing: u16) -> Self {
        self.outer_spacing = outer_spacing;
        let mut outer_selector = " ".repeat(self.outer_spacing as usize);
        let mut outer_selected_selector = outer_selector.clone();
        outer_selector.push_str(&self.selector);
        outer_selected_selector.push_str(&self.selected_selector);
        self.selector = outer_selector;
        self.selected_selector = outer_selected_selector;
        self
    }

    //
    pub fn inner_spacing(mut self, inner_spacing: u16) -> Self {
        self.inner_spacing = inner_spacing;
        self.selector
            .push_str(&" ".repeat(self.inner_spacing as usize));
        self.selected_selector
            .push_str(&" ".repeat(self.inner_spacing as usize));
        self
    }

    pub fn selected_foreground_color(mut self, color: Color) -> Self {
        self.selected_foreground_color = color;
        self
    }

    pub fn selected_background_color(mut self, color: Color) -> Self {
        self.selected_background_color = color;
        self
    }

    //
    pub fn selected_option_foreground_color(mut self, color: Color) -> Self {
        self.selected_option_foreground_color = color;
        self
    }
    //
    pub fn selected_option_background_color(mut self, color: Color) -> Self {
        self.selected_option_background_color = color;
        self
    }
    //
    pub fn selected_selected_option_foreground_color(mut self, color: Color) -> Self {
        self.selected_selected_option_foreground_color = color;
        self
    }
    //
    pub fn selected_selected_option_background_color(mut self, color: Color) -> Self {
        self.selected_selected_option_background_color = color;
        self
    }

    pub fn get_title(&self) -> &String {
        &self.title
    }

    pub fn get_options(&self) -> &Vec<T> {
        &self.options
    }

    fn format_option(&self, index: usize) -> String {
        format!("{}", self.options[index])
    }

    fn format_title(&self) -> String {
        format!("{}\n", self.title)
    }

    //
    pub fn display(&mut self) -> Result<(), Box<dyn StdError>> {
        let title = self.format_title();
        print!("{}", title);
        let selector = &self.selector;
        let selected_selector = &self.selected_selector;
        for i in 0..self.options.len() {
            let option = self.format_option(i);
            if i == self.selected_index {
                if self.selected_options.contains(&i) {
                    execute!(
                        self.stdout,
                        SetForegroundColor(self.selected_selected_option_foreground_color),
                        SetBackgroundColor(self.selected_selected_option_background_color),
                        Print(selector),
                        Print(option),
                        ResetColor,
                        Print("\n")
                    )?;
                    continue;
                }
                execute!(
                    self.stdout,
                    SetForegroundColor(self.selected_foreground_color),
                    SetBackgroundColor(self.selected_background_color),
                    Print(selector),
                    Print(option),
                    ResetColor,
                    Print("\n")
                )?;
                continue;
            }
            if self.selected_options.contains(&i) {
                execute!(
                    self.stdout,
                    SetForegroundColor(self.selected_option_foreground_color),
                    SetBackgroundColor(self.selected_option_background_color),
                    Print(selected_selector),
                    Print(option),
                    ResetColor,
                    Print("\n")
                )?;
                continue;
            }
            execute!(
                self.stdout,
                cursor::MoveRight(self.selector.len() as u16),
                Print(option),
                Print("\n"),
            )?;
        }
        execute!(
            self.stdout,
            cursor::MoveToPreviousLine((self.options.len() - self.selected_index) as u16),
        )?;
        Ok(())
    }

    fn restore_console(&mut self) -> Result<(), Box<dyn StdError>> {
        disable_raw_mode()?;
        execute!(
            self.stdout,
            cursor::MoveToNextLine(self.options.len() as u16 - self.selected_index as u16),
            cursor::Show,
        )?;
        Ok(())
    }

    fn setup_console(&mut self) -> Result<(), Box<dyn StdError>> {
        enable_raw_mode()?;
        execute!(self.stdout, cursor::Hide)?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<Option<HashSet<usize>>, Box<dyn StdError>> {
        self.setup_console()?;
        self.display()?;
        let selector = &self.selector;
        let selected_selector = &self.selected_selector;
        loop {
            match read()? {
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
                                Print(selected_selector),
                                Print(current_line_out),
                                cursor::MoveToPreviousLine(1),
                                ResetColor
                            )?;
                        } else {
                            execute!(
                                self.stdout,
                                Clear(ClearType::CurrentLine),
                                cursor::MoveRight(self.selector.len() as u16),
                                Print(current_line_out),
                                cursor::MoveToPreviousLine(1)
                            )?;
                        }
                        if self.selected_options.contains(&self.selected_index) {
                            execute!(
                                self.stdout,
                                Clear(ClearType::CurrentLine),
                                SetForegroundColor(self.selected_selected_option_foreground_color),
                                SetBackgroundColor(self.selected_selected_option_background_color),
                                Print(selector),
                                Print(next_line_out),
                                ResetColor,
                                cursor::MoveToColumn(1)
                            )?;
                        } else {
                            execute!(
                                self.stdout,
                                SetForegroundColor(self.selected_foreground_color),
                                SetBackgroundColor(self.selected_background_color),
                                Print(selector),
                                Print(next_line_out),
                                ResetColor,
                                cursor::MoveToColumn(1)
                            )?;
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
                                Print(selected_selector),
                                Print(current_line_out),
                                cursor::MoveToNextLine(1),
                                ResetColor
                            )?;
                        } else {
                            execute!(
                                self.stdout,
                                Clear(ClearType::CurrentLine),
                                cursor::MoveRight(self.selector.len() as u16),
                                Print(current_line_out),
                                cursor::MoveToNextLine(1)
                            )?;
                        }
                        if self.selected_options.contains(&self.selected_index) {
                            execute!(
                                self.stdout,
                                Clear(ClearType::CurrentLine),
                                SetForegroundColor(self.selected_selected_option_foreground_color),
                                SetBackgroundColor(self.selected_selected_option_background_color),
                                Print(selector),
                                Print(next_line_out),
                                ResetColor,
                                cursor::MoveToColumn(1)
                            )?;
                        } else {
                            execute!(
                                self.stdout,
                                SetForegroundColor(self.selected_foreground_color),
                                SetBackgroundColor(self.selected_background_color),
                                Print(selector),
                                Print(next_line_out),
                                ResetColor,
                                cursor::MoveToColumn(1)
                            )?;
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
                            Print(selector),
                            Print(out),
                            ResetColor,
                            cursor::MoveToColumn(1)
                        )?;
                        self.selected_options.remove(&self.selected_index);
                    } else {
                        execute!(
                            self.stdout,
                            Clear(ClearType::CurrentLine),
                            SetForegroundColor(self.selected_selected_option_foreground_color),
                            SetBackgroundColor(self.selected_selected_option_background_color),
                            Print(selector),
                            Print(out),
                            ResetColor,
                            cursor::MoveToColumn(1)
                        )?;
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
                    self.restore_console()?;
                    return Ok(None);
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                }) => {
                    self.restore_console()?;
                    process::exit(1);
                }
                _ => continue,
            }
        }
        self.restore_console()?;
        if self.selected_options.is_empty() {
            return Ok(None);
        }
        Ok(Some(self.selected_options.clone()))
    }
}
