use menu::{Color, Menu};
use std::io::{stdin, Read};

fn main() {
    let options = vec![
        "Enter Nested Menu",
        "Option 2",
        "Option 3",
        "Option 4",
        "Option 5",
        "Option 6",
    ];
    let mut menu = match Menu::new("Main Menu!\n", &options, 0, " > ") {
        Ok(mut menu) => {
            menu.set_selected_foreground_color(Color::Blue);
            menu.set_selected_background_color(Color::Black);
            menu
        }
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    let nested_options = vec![
        "Nested Option 1",
        "Nested Option 2",
        "Nested Option 3",
        "Nested Option 4",
        "Nested Option 5",
        "Nested Option 6",
    ];
    let mut nested_menu = match Menu::new("Nested Menu!\n", &nested_options, 0, " > ") {
        Ok(mut menu) => {
            menu.set_selected_foreground_color(Color::Green);
            menu.set_selected_background_color(Color::Black);
            menu
        }
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    loop {
        let selected = menu.run();
        match selected {
            Some(index) => match index {
                0 => loop {
                    let selected = nested_menu.run();
                    match selected {
                        Some(index) => {
                            println!("You selected: {}", nested_options[index]);
                            stdin().read(&mut [0]).unwrap();
                        }
                        None => {
                            break;
                        }
                    }
                },
                _ => {
                    println!("You selected: {}", options[index]);
                    stdin().read(&mut [0]).unwrap();
                }
            },
            None => break,
        }
    }
    println!("Exiting...");
}
