use menu::Menu;
fn main() {
    let mut menu = Menu::new()
        .title("How are you today?".to_owned())
        .inner_spacing(1)
        .outer_spacing(0)
        .selected_foreground_color(menu::Color::Yellow)
        .selected_background_color(menu::Color::Black)
        .options(vec![
            ":>",
            ":D",
            ":/",
            ":(",
            ":)",
        ])
        .selected_index(1).unwrap();
    let res = menu.run().unwrap();
    if let Some(index) = res {
        println!("You selected: {}", menu.get_options()[index]);
    } else {
        println!("You didn't select anything");
    }
}
