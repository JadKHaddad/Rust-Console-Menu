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
use std::io::{stdout, Write};
use std::{collections::HashSet, process};

pub enum Direction {
    Up,
    Down,
}

pub trait MenuLike {
    fn get_menu_mut(&mut self) -> &mut Menu;

    fn get_menu(&self) -> &Menu;

    fn title(&mut self, title: String) {
        let mut_menu = self.get_menu_mut();
        mut_menu.title = title;
        mut_menu.new_line_count = mut_menu.title.matches('\n').count();
    }

    fn options(&mut self, options: Vec<String>) {
        let mut_menu = self.get_menu_mut();
        mut_menu.options = options;
    }

    fn selected_options(&mut self, selected_options: HashSet<usize>) {
        let mut_menu = self.get_menu_mut();
        let options_len = mut_menu.options.len().clone();
        mut_menu.selected_options = selected_options;
        mut_menu
            .selected_options
            .retain(|index| *index < options_len);
    }

    fn selected_index(&mut self, selected_index: usize) {
        let mut_menu = self.get_menu_mut();
        if selected_index >= mut_menu.options.len() {
            mut_menu.selected_index = 0;
        } else {
            mut_menu.selected_index = selected_index;
        }
    }

    fn selector(&mut self, selector: String) {
        let mut_menu = self.get_menu_mut();
        mut_menu.selector = selector;
    }

    fn selected_foreground_color(&mut self, color: Color) {
        let mut_menu = self.get_menu_mut();
        mut_menu.selected_foreground_color = color;
    }

    fn selected_background_color(&mut self, color: Color) {
        let mut_menu = self.get_menu_mut();
        mut_menu.selected_background_color = color;
    }

    fn get_title(&self) -> &String {
        let menu = self.get_menu();
        &menu.title
    }

    fn get_options(&self) -> &Vec<String> {
        let menu = self.get_menu();
        &menu.options
    }

    fn get_selected_options(&self) -> &HashSet<usize> {
        let menu = self.get_menu();
        &menu.selected_options
    }

    fn get_selected_index(&self) -> usize {
        let menu = self.get_menu();
        menu.selected_index
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
        execute!(mut_menu.stdout, cursor::MoveToNextLine(dist), cursor::Show)?;
        Ok(())
    }

    fn setup_console(&mut self) -> Result<(), Box<dyn StdError>> {
        let mut_menu = self.get_menu_mut();
        enable_raw_mode()?;
        execute!(mut_menu.stdout, cursor::Hide)?;
        Ok(())
    }

    fn move_with_direction(
        &mut self,
        direction: Direction,
        current_line_out: String,
        next_line_out: String,
    ) -> Result<(), Box<dyn StdError>> {
        let mut_menu = self.get_menu_mut();
        let selector = &mut_menu.selector;
        let selected_foreground_color = mut_menu.selected_foreground_color;
        let selected_background_color = mut_menu.selected_background_color;
        let dist = mut_menu.selector.len() as u16;
        queue!(
            mut_menu.stdout,
            Clear(ClearType::CurrentLine),
            cursor::MoveRight(dist),
            Print(current_line_out),
        )?;
        match direction {
            Direction::Up => {
                queue!(mut_menu.stdout, cursor::MoveToPreviousLine(1))?;
            }
            Direction::Down => {
                queue!(mut_menu.stdout, cursor::MoveToNextLine(1))?;
            }
        }
        queue!(
            mut_menu.stdout,
            Clear(ClearType::CurrentLine),
            SetForegroundColor(selected_foreground_color),
            SetBackgroundColor(selected_background_color),
            Print(selector),
            Print(next_line_out),
            cursor::MoveToColumn(1),
            ResetColor
        )?;
        Ok(())
    }

    fn on_up_key(&mut self) -> Result<(), Box<dyn StdError>> {
        let mut_menu = self.get_menu_mut();
        if mut_menu.selected_index > 0 {
            let current_line_out = mut_menu.format_option(mut_menu.selected_index);
            mut_menu.selected_index -= 1;
            let next_line_out = mut_menu.format_option(mut_menu.selected_index);
            self.move_with_direction(Direction::Up, current_line_out, next_line_out)?;
        }
        Ok(())
    }

    fn on_down_key(&mut self) -> Result<(), Box<dyn StdError>> {
        let mut_menu = self.get_menu_mut();
        if mut_menu.selected_index < mut_menu.options.len() - 1 {
            let current_line_out = mut_menu.format_option(mut_menu.get_menu().selected_index);
            mut_menu.selected_index += 1;
            let next_line_out = mut_menu.format_option(mut_menu.get_menu().selected_index);
            self.move_with_direction(Direction::Down, current_line_out, next_line_out)?;
        }
        Ok(())
    }

    fn on_space_key(&mut self) -> Result<(), Box<dyn StdError>> {
        Ok(())
    }

    fn on_break(&mut self) -> Result<Option<HashSet<usize>>, Box<dyn StdError>> {
        let mut selected = HashSet::new();
        selected.insert(self.get_selected_index());
        Ok(Some(selected))
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
                queue!(
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
            queue!(
                mut_menu.stdout,
                cursor::MoveRight(dist),
                Print(option),
                Print("\n"),
            )?;
        }
        let dist = (mut_menu.options.len() - mut_menu.selected_index) as u16;
        queue!(mut_menu.stdout, cursor::MoveToPreviousLine(dist))?;
        mut_menu.stdout.flush()?;
        Ok(())
    }

    fn run(&mut self) -> Result<Option<HashSet<usize>>, Box<dyn StdError>> {
        self.setup_console()?;
        self.display()?;

        loop {
            match read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Up,
                    modifiers: KeyModifiers::NONE,
                }) => {
                    self.on_up_key()?;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::NONE,
                }) => {
                    self.on_down_key()?;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char(' '),
                    modifiers: KeyModifiers::NONE,
                }) => {
                    self.on_space_key()?;
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
            self.get_menu_mut().stdout.flush()?;
        }
        self.restore_console()?;
        self.on_break()
    }
}

impl MenuLike for Menu {
    fn get_menu_mut(&mut self) -> &mut Menu {
        self
    }

    fn get_menu(&self) -> &Menu {
        self
    }
}
pub struct Menu {
    title: String,
    options: Vec<String>,
    selected_options: HashSet<usize>,
    selected_index: usize,
    stdout: std::io::Stdout,
    new_line_count: usize,
    selector: String,
    selected_foreground_color: Color,
    selected_background_color: Color,
}

impl Menu {
    pub fn new(
        title: String,
        options: Vec<String>,
        selected_options: HashSet<usize>,
        selected_index: usize,
        selector: String,
        selected_foreground_color: Color,
        selected_background_color: Color,
    ) -> Menu {
        let mut menu = Menu::default();
        menu.title(title);
        menu.options(options);
        menu.selected_options(selected_options);
        menu.selected_index(selected_index);
        menu.selector(selector);
        menu.selected_foreground_color(selected_foreground_color);
        menu.selected_background_color(selected_background_color);
        menu
    }
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            stdout: stdout(),
            title: String::from("Single Select Menu"),
            options: vec![
                String::from("Option 1"),
                String::from("Option 2"),
                String::from("Option 3"),
            ],
            selected_options: HashSet::new(),
            selected_index: 0,
            new_line_count: 0,
            selector: String::from("=> "),
            selected_foreground_color: Color::Reset,
            selected_background_color: Color::Reset,
        }
    }
}

pub struct MultiMenu {
    menu: Menu,
    selected_selector: String,
    selected_option_foreground_color: Color,
    selected_option_background_color: Color,
    selected_selected_option_foreground_color: Color,
    selected_selected_option_background_color: Color,
}

impl MultiMenu {
    pub fn new(
        title: String,
        options: Vec<String>,
        selected_options: HashSet<usize>,
        selected_index: usize,
        selector: String,
        selected_selector: String,
        selected_foreground_color: Color,
        selected_background_color: Color,
        selected_option_foreground_color: Color,
        selected_option_background_color: Color,
        selected_selected_option_foreground_color: Color,
        selected_selected_option_background_color: Color,
    ) -> MultiMenu {
        let mut menu = MultiMenu::default();
        menu.title(title);
        menu.options(options);
        menu.selected_options(selected_options);
        menu.selected_index(selected_index);
        menu.selector(selector);
        menu.selected_selector(selected_selector);
        menu.selected_foreground_color(selected_foreground_color);
        menu.selected_background_color(selected_background_color);
        menu.selected_option_foreground_color(selected_option_foreground_color);
        menu.selected_option_background_color(selected_option_background_color);
        menu.selected_selected_option_foreground_color(selected_selected_option_foreground_color);
        menu.selected_selected_option_background_color(selected_selected_option_background_color);
        menu
    }
}

impl Default for MultiMenu {
    fn default() -> Self {
        let mut menu = Menu::default();
        menu.title(String::from("Multi Select Menu"));
        Self {
            menu,
            selected_selector: String::from("-> "),
            selected_option_foreground_color: Color::Reset,
            selected_option_background_color: Color::Reset,
            selected_selected_option_foreground_color: Color::Reset,
            selected_selected_option_background_color: Color::Reset,
        }
    }
}

impl MenuLike for MultiMenu {
    fn get_menu_mut(&mut self) -> &mut Menu {
        &mut self.menu
    }

    fn get_menu(&self) -> &Menu {
        self.menu.get_menu()
    }

    fn move_with_direction(
        &mut self,
        direction: Direction,
        current_line_out: String,
        next_line_out: String,
    ) -> Result<(), Box<dyn StdError>> {
        let selected_selector = &self.selected_selector.clone();

        let selected_option_foreground_color = self.selected_option_foreground_color;
        let selected_option_background_color = self.selected_option_background_color;
        let selected_selected_option_foreground_color =
            self.selected_selected_option_foreground_color;
        let selected_selected_option_background_color =
            self.selected_selected_option_background_color;

        let mut_menu = self.get_menu_mut();
        let selector = &mut_menu.selector;
        let selected_foreground_color = mut_menu.selected_foreground_color;
        let selected_background_color = mut_menu.selected_background_color;
        let dist = mut_menu.selector.len() as u16;

        queue!(mut_menu.stdout, Clear(ClearType::CurrentLine))?;
        let cond = match direction {
            Direction::Up => (mut_menu.selected_index + 1),
            Direction::Down => (mut_menu.selected_index - 1),
        };
        if mut_menu.selected_options.contains(&cond) {
            queue!(
                mut_menu.stdout,
                SetForegroundColor(selected_option_foreground_color),
                SetBackgroundColor(selected_option_background_color),
                Print(selected_selector),
                Print(current_line_out),
                ResetColor,
            )?;
        } else {
            queue!(
                mut_menu.stdout,
                cursor::MoveRight(dist),
                Print(current_line_out),
            )?;
        }
        match direction {
            Direction::Up => {
                queue!(
                    mut_menu.stdout,
                    cursor::MoveToPreviousLine(1),
                    Clear(ClearType::CurrentLine)
                )?;
            }
            Direction::Down => {
                queue!(
                    mut_menu.stdout,
                    cursor::MoveToNextLine(1),
                    Clear(ClearType::CurrentLine)
                )?;
            }
        }
        if mut_menu.selected_options.contains(&mut_menu.selected_index) {
            queue!(
                mut_menu.stdout,
                SetForegroundColor(selected_selected_option_foreground_color),
                SetBackgroundColor(selected_selected_option_background_color),
            )?;
        } else {
            queue!(
                mut_menu.stdout,
                SetForegroundColor(selected_foreground_color),
                SetBackgroundColor(selected_background_color),
            )?;
        }
        queue!(
            mut_menu.stdout,
            Print(selector),
            Print(next_line_out),
            ResetColor,
            cursor::MoveToColumn(1)
        )?;

        Ok(())
    }

    fn on_space_key(&mut self) -> Result<(), Box<dyn StdError>> {
        let selected_selected_option_foreground_color =
            self.selected_selected_option_foreground_color;
        let selected_selected_option_background_color =
            self.selected_selected_option_background_color;
        let mut_menu = self.get_menu_mut();
        let selector = &mut_menu.selector;
        let selected_foreground_color = mut_menu.selected_foreground_color;
        let selected_background_color = mut_menu.selected_background_color;
        let option = mut_menu.format_option(mut_menu.selected_index);
        queue!(mut_menu.stdout, Clear(ClearType::CurrentLine))?;
        if mut_menu.selected_options.contains(&mut_menu.selected_index) {
            queue!(
                mut_menu.stdout,
                SetForegroundColor(selected_foreground_color),
                SetBackgroundColor(selected_background_color),
            )?;
            mut_menu.selected_options.remove(&mut_menu.selected_index);
        } else {
            queue!(
                mut_menu.stdout,
                SetForegroundColor(selected_selected_option_foreground_color),
                SetBackgroundColor(selected_selected_option_background_color),
            )?;
            mut_menu.selected_options.insert(mut_menu.selected_index);
        }
        queue!(
            mut_menu.stdout,
            Print(selector),
            Print(option),
            ResetColor,
            cursor::MoveToColumn(1)
        )?;
        Ok(())
    }

    fn on_break(&mut self) -> Result<Option<HashSet<usize>>, Box<dyn StdError>> {
        if self.get_selected_options().is_empty() {
            return Ok(None);
        }
        Ok(Some(self.get_selected_options().clone()))
    }

    fn display(&mut self) -> Result<(), Box<dyn StdError>> {
        let selected_selector = &self.selected_selector.clone();
        let selected_option_foreground_color = self.selected_option_foreground_color;
        let selected_option_background_color = self.selected_option_background_color;
        let selected_selected_option_foreground_color =
            self.selected_selected_option_foreground_color;
        let selected_selected_option_background_color =
            self.selected_selected_option_background_color;

        let mut_menu = self.get_menu_mut();

        let title = mut_menu.format_title();
        print!("{}", title);
        for i in 0..mut_menu.options.len() {
            let option = mut_menu.format_option(i);
            if i == mut_menu.selected_index {
                let selector = &mut_menu.selector;
                let selected_foreground_color = mut_menu.selected_foreground_color;
                let selected_background_color = mut_menu.selected_background_color;
                if mut_menu.selected_options.contains(&i) {
                    queue!(
                        mut_menu.stdout,
                        SetForegroundColor(selected_selected_option_foreground_color),
                        SetBackgroundColor(selected_selected_option_background_color),
                        Print(selector),
                        Print(option),
                        ResetColor,
                        Print("\n")
                    )?;
                    continue;
                }
                queue!(
                    mut_menu.stdout,
                    SetForegroundColor(selected_foreground_color),
                    SetBackgroundColor(selected_background_color),
                    Print(selector),
                    Print(option),
                    ResetColor,
                    Print("\n")
                )?;
                continue;
            }
            if mut_menu.selected_options.contains(&i) {
                queue!(
                    mut_menu.stdout,
                    SetForegroundColor(selected_option_foreground_color),
                    SetBackgroundColor(selected_option_background_color),
                    Print(selected_selector),
                    Print(option),
                    ResetColor,
                    Print("\n")
                )?;
                continue;
            }
            let dist = mut_menu.selector.len() as u16;
            queue!(
                mut_menu.stdout,
                cursor::MoveRight(dist),
                Print(option),
                Print("\n"),
            )?;
        }
        let dist = (mut_menu.options.len() - mut_menu.selected_index) as u16;
        queue!(mut_menu.stdout, cursor::MoveToPreviousLine(dist))?;
        mut_menu.stdout.flush()?;
        Ok(())
    }
}

impl MultiMenu {
    //TODO selector and selected_selector must be the same length
    pub fn selected_selector(&mut self, selected_selector: String) {
        self.selected_selector = selected_selector;
    }

    pub fn selected_option_foreground_color(&mut self, color: Color) {
        self.selected_option_foreground_color = color;
    }

    pub fn selected_option_background_color(&mut self, color: Color) {
        self.selected_option_background_color = color;
    }

    pub fn selected_selected_option_foreground_color(&mut self, color: Color) {
        self.selected_selected_option_foreground_color = color;
    }

    pub fn selected_selected_option_background_color(&mut self, color: Color) {
        self.selected_selected_option_background_color = color;
    }
}
