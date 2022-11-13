use menu::Menu;
use menu::MenuLike;
use menu::MultiMenu;

fn main() {
    let mut selected_options = std::collections::HashSet::new();
    selected_options.insert(1);
    let mut multimenu = MultiMenu::default();
    multimenu.selected_background_color(menu::Color::Black);
    multimenu.selected_foreground_color(menu::Color::Yellow);
    multimenu.selected_options(selected_options);

    multimenu.selected_selected_option_background_color(menu::Color::White);
    multimenu.selected_selected_option_foreground_color(menu::Color::Red);

    let mut menu = Menu::default();
    menu.selected_background_color(menu::Color::Black);
    menu.selected_foreground_color(menu::Color::Yellow);


    let menus: Vec<Box<dyn MenuLike>>  = vec![Box::new(multimenu), Box::new(menu)];
    for mut menu in menus {
        let res = menu.run().unwrap();
        if let Some(indecies) = res {
            println!(
                "You selected: {:?}",
                indecies
                    .iter()
                    .map(|i| menu.get_options()[*i].clone())
                    .collect::<Vec<String>>()
            );
        } else {
            println!("You didn't select anything");
        }
    }
}
