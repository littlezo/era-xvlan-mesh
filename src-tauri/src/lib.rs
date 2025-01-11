#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Emitter;
use tauri_plugin_autostart::MacosLauncher;

mod invoke;
mod menu;

use crate::invoke::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt::init();

    if !invoke::check_sudo() {
        std::process::exit(0);
    }

    let mut builder = tauri::Builder::default();
    // .plugin(tauri_plugin_single_instance::init());

    builder = builder
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // 在这里写代码 ……
            invoke::focus_window(app)
        }))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![invoke::AUTOSTART_ARG]),
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_positioner::init());

    builder
        .setup(|app| {
            let app_handle = app.handle();
            #[cfg(not(target_os = "android"))]
            let _ = menu::create_menu(app_handle);
            #[cfg(not(target_os = "android"))]
            let _ = menu::create_tray(app_handle);
            return Ok(());
        })
        .invoke_handler(tauri::generate_handler![
            invoke::parse_network_config,
            invoke::start_network_instance,
            invoke::stop_network_instance,
            invoke::collect_network_infos,
            invoke::get_os_hostname,
            invoke::set_logging_level,
            invoke::set_tun_fd,
            invoke::is_autostart,
            invoke::test_config,
            invoke::era_xvlan_version
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
