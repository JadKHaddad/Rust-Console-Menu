use menu::{Color, MultiMenu};
use std::collections::HashSet;

fn main() {
    let options = vec![
        "Option 1", "Option 2", "Option 3", "Option 4", "Option 5", "Option 6",
    ];
    let selected_options = HashSet::from([0, 2, 3]);
    let mut multi_menu = match MultiMenu::new(
        "Main Menu!\n",
        &options,
        0,
        " > ",
        selected_options,
        " * ",
        false,
    ) {
        Ok(mut menu) => {
            menu.set_selected_foreground_color(Color::Blue);
            menu.set_selected_background_color(Color::Black);
            menu.set_selected_selected_option_foreground_color(Color::Green);
            menu.set_selected_option_foreground_color(Color::Yellow);
            menu
        }
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    let selected_options = multi_menu.run();
    println!("Selected options: {:?}", selected_options);
}
