use menu::Menu;
use menu::MultiMenu;

fn main() {
    let mut multimenu = MultiMenu::new()
        .title("How are you today?".to_owned())
        .selected_selector(">>".to_owned())
        .inner_spacing(1)
        .outer_spacing(0)
        .selected_foreground_color(menu::Color::Yellow)
        .selected_background_color(menu::Color::Black)
        .selected_option_foreground_color(menu::Color::White)
        .selected_option_background_color(menu::Color::Black)
        .selected_selected_option_foreground_color(menu::Color::Green)
        .selected_selected_option_background_color(menu::Color::Grey)
        .options(vec![":>", ":D", ":/", ":(", ":)"])
        .selected_index(1)
        .unwrap();
    let res = multimenu.run().unwrap();
    if let Some(indecies) = res {
        println!(
            "You selected: {:?}",
            indecies
                .iter()
                .map(|i| multimenu.get_options()[*i].clone())
                .collect::<Vec<&str>>()
        );
    } else {
        println!("You didn't select anything");
    }

    let mut menu = Menu::new()
        .title("How are you today?".to_owned())
        .inner_spacing(1)
        .outer_spacing(0)
        .selected_foreground_color(menu::Color::Yellow)
        .selected_background_color(menu::Color::Black)
        .options(vec![":>", ":D", ":/", ":(", ":)"])
        .selected_index(1)
        .unwrap();
    let res = menu.run().unwrap();
    if let Some(index) = res {
        println!("You selected: {}", menu.get_options()[index]);
    } else {
        println!("You didn't select anything");
    }
}
