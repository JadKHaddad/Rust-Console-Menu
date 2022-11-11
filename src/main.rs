use menu::Menu;
fn main() {
    let mut menu = Menu::new()
        .title("How would you like to do this?\n".to_owned())
        .inner_spacing(1)
        .outer_spacing(0)
        .options(vec![
            "Option 1".to_owned(),
            "Option 2".to_owned(),
            "Option 3".to_owned(),
        ]);
    let res = menu.run().unwrap();
    if let Some(index) = res {
        println!("You selected: {}", menu.get_options()[index]);
    } else {
        println!("You didn't select anything");
    }
}
