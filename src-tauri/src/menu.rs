use tauri::{
    menu::{
        AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu, HELP_SUBMENU_ID,
        WINDOW_SUBMENU_ID,
    },
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Runtime,
};

/// Creates a menu filled with default menu items and submenus.
#[cfg(not(target_os = "android"))]
pub fn create_menu<R: Runtime>(app_handle: &AppHandle<R>) -> tauri::Result<()> {
    // app_handle.remove_menu();
    let pkg_info = app_handle.package_info();
    let config = app_handle.config();
    let app_name = pkg_info.name.clone();
    let about_metadata = AboutMetadata {
        name: Some(app_name.clone()),
        version: Some(pkg_info.version.to_string()),
        copyright: config.bundle.copyright.clone(),
        authors: config.bundle.publisher.clone().map(|p| vec![p]),
        icon: Some(app_handle.default_window_icon().unwrap().clone()),
        ..Default::default()
    };

    let window_menu = Submenu::with_id_and_items(
        app_handle,
        WINDOW_SUBMENU_ID,
        "Window",
        true,
        &[
            &PredefinedMenuItem::minimize(app_handle, None)?,
            &PredefinedMenuItem::maximize(app_handle, None)?,
            #[cfg(target_os = "macos")]
            &PredefinedMenuItem::separator(app_handle)?,
            &PredefinedMenuItem::close_window(app_handle, None)?,
        ],
    )?;

    let help_menu = Submenu::with_id_and_items(
        app_handle,
        HELP_SUBMENU_ID,
        "Help",
        true,
        &[
            #[cfg(not(target_os = "macos"))]
            &PredefinedMenuItem::about(app_handle, None, Some(about_metadata))?,
            #[cfg(debug_assertions)]
            &MenuItem::with_id(
                app_handle,
                "inspect",
                "Devtools Inspect",
                true,
                None::<String>,
            )?,
        ],
    )?;

    let menu = Menu::with_items(
        app_handle,
        &[
            #[cfg(target_os = "macos")]
            &Submenu::with_items(
                app_handle,
                app_name,
                true,
                &[
                    &PredefinedMenuItem::about(app_handle, None, Some(about_metadata))?,
                    &PredefinedMenuItem::separator(app_handle)?,
                    &PredefinedMenuItem::hide(app_handle, None)?,
                    &PredefinedMenuItem::separator(app_handle)?,
                    &PredefinedMenuItem::quit(app_handle, None)?,
                ],
            )?,
            #[cfg(target_os = "macos")]
            &Submenu::with_items(
                app_handle,
                "View",
                true,
                &[&PredefinedMenuItem::fullscreen(app_handle, None)?],
            )?,
            &window_menu,
            &help_menu,
        ],
    )?;
    app_handle.set_menu(menu)?;

    Ok({})
}
#[cfg(not(target_os = "android"))]
pub fn create_tray<R: Runtime>(app_handle: &AppHandle<R>) -> tauri::Result<()> {
    use crate::invoke;

    let branding = MenuItem::with_id(
        app_handle,
        "name",
        app_handle.package_info().name.clone(),
        false,
        None::<String>,
    )?;
    let _quit = MenuItem::with_id(app_handle, "quit", "Quit", true, None::<String>)?;
    let _inspect = MenuItem::with_id(
        app_handle,
        "inspect",
        "Devtools Inspect",
        true,
        None::<String>,
    )?;
    let _menu = Menu::with_items(app_handle, &[&branding, &_inspect, &_quit])?;

    TrayIconBuilder::with_id("main")
        .tooltip(app_handle.package_info().name.clone())
        .icon(app_handle.default_window_icon().unwrap().clone())
        .menu(&_menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            }
            "inspect" => {
                #[cfg(debug_assertions)]
                let _ = invoke::toggle_devtools(app);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                invoke::toggle_window_visibility(app);
            }
        })
        .icon_as_template(false)
        .build(app_handle)?;
    Ok({})
}
