use tauri::Menu;

pub trait AddDefaultSubmenus {
  fn add_default_app_submenu_if_macos(self) -> Self;
}

impl AddDefaultSubmenus for Menu {
  fn add_default_app_submenu_if_macos(self) -> Menu {
    use tauri::MenuItem;
    #[cfg(target_os = "macos")]
    use tauri::MenuItemSubmenu;
    use tauri::Submenu;

    return self.add_submenu(Submenu::new(
      "Menu",
      Menu::new()
        .add_native_item(MenuItem::Copy)
        .add_native_item(MenuItem::Paste)
        .add_native_item(MenuItem::Hide)
        .add_native_item(MenuItem::HideOthers)
        .add_native_item(MenuItem::ShowAll)
        .add_native_item(MenuItem::Quit),
    ));
  }
}