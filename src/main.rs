use menu::Menu;
use menu::MenuLike;
use menu::MultiMenu;

fn main() {
    let multimenu = MultiMenu::default();
    let menu = Menu::default();

    let menus: Vec<Box<dyn MenuLike>>  = vec![Box::new(menu), Box::new(multimenu)];
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
