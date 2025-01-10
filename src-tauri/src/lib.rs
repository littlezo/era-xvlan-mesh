#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter,
    Manager, // Runtime,
};
use tauri_plugin_autostart::MacosLauncher;

pub mod invoke;

use crate::invoke::*;

pub const AUTOSTART_ARG: &str = "--autostart";

fn toggle_window_visibility<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or_default() {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt::init();

    if !check_sudo() {
        std::process::exit(0);
    }

    let mut builder = tauri::Builder::default();

    builder = builder
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![AUTOSTART_ARG]),
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_positioner::init());

    builder
        .setup(|app| {
            // for logging config
            // for tray icon, menu need to be built in js
            #[cfg(not(target_os = "android"))]
            {
                let branding = MenuItem::with_id(
                    app,
                    "name",
                    app.package_info().name.clone(),
                    false,
                    None::<String>,
                )?;
                let _quit = MenuItem::with_id(app, "quit", "Quit", true, None::<String>)?;
                let _inspect = MenuItem::with_id(app, "inspect", "Inspect", true, None::<String>)?;
                let _menu = Menu::with_items(app, &[&branding, &_inspect, &_quit])?;

                TrayIconBuilder::with_id("main")
                    .tooltip(app.package_info().name.clone())
                    .icon(app.default_window_icon().unwrap().clone())
                    .menu(&_menu)
                    .show_menu_on_left_click(false)
                    .on_menu_event(move |app, event| match event.id.as_ref() {
                        "quit" => {
                            app.exit(0);
                        }
                        "inspect" => {
                            if let Some(window) = app.get_webview_window("main") {
                                if window.is_devtools_open() {
                                    window.close_devtools();
                                } else {
                                    window.open_devtools();
                                }
                            };
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
                            toggle_window_visibility(app);
                        }
                    })
                    .icon_as_template(false)
                    .build(app)?;
                #[cfg(debug_assertions)]
                {
                    if let Some(window) = app.get_webview_window("main") {
                        if !window.is_devtools_open() {
                            window.open_devtools();
                        }
                    }
                }
            }

            return Ok(());
        })
        .invoke_handler(tauri::generate_handler![
            parse_network_config,
            start_network_instance,
            stop_network_instance,
            collect_network_infos,
            test_config,
            is_autostart,
        ])
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                let _ = window.emit("era://xvlan/mesh/window/close", ());
                api.prevent_close();
            }
            _ => {}
        })
        .build(tauri::generate_context!())
        .unwrap()
        .run(|_app, _event| {});
    // .expect("error while running tauri application");
}

fn check_sudo() -> bool {
    let is_elevated = privilege::user::privileged();
    if !is_elevated {
        let Ok(exe) = std::env::current_exe() else {
            return true;
        };
        let args: Vec<String> = std::env::args().collect();
        let mut elevated_cmd = privilege::runas::Command::new(exe);
        if args.contains(&AUTOSTART_ARG.to_owned()) {
            elevated_cmd.arg(AUTOSTART_ARG);
        }
        let _ = elevated_cmd.force_prompt(true).hide(true).gui(true).run();
    }
    is_elevated
}
