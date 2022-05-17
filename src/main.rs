use menu::Menu;

fn main() {
    let options = vec!["Option 1", "Option 2", "Option 3", "Option 4", "Option 5", "Option 6"];
    let mut menu = match Menu::new("Main Menu!", &options, 1, 0, |a| String::from(a) ){
        Ok(menu) => menu,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };
    menu.run();
}

