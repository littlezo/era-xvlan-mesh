use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
    sync::{Mutex, OnceLock},
};

use dashmap::DashMap;
use era_xvlan::{
    common::config::ConfigLoader,
    launcher::{NetworkConfig, NetworkInstance, NetworkInstanceRunningInfo},
    utils::NewFilterSender,
};
use tauri::{AppHandle, Emitter, Manager as _, Runtime};

pub const AUTOSTART_ARG: &str = "--autostart";

static INSTANCE_MAP: once_cell::sync::Lazy<DashMap<String, NetworkInstance>> =
    once_cell::sync::Lazy::new(DashMap::new);

static EMIT_INSTANCE_INFO: once_cell::sync::Lazy<AtomicBool> =
    once_cell::sync::Lazy::new(|| AtomicBool::new(false));
pub static LOGGER_LEVEL_SENDER: OnceLock<Mutex<Option<NewFilterSender>>> = OnceLock::new();

#[tauri::command]
pub fn era_xvlan_version() -> Result<String, String> {
    Ok(era_xvlan::VERSION.to_string())
}

#[tauri::command]
pub fn is_autostart() -> Result<bool, String> {
    let args: Vec<String> = std::env::args().collect();
    println!("{:?}", args);
    #[cfg(debug_assertions)]
    eprintln!("args: {:?}", args); // 使用 eprintln!
    Ok(args.contains(&crate::AUTOSTART_ARG.to_owned()))
}

#[tauri::command]
pub fn parse_network_config(cfg: NetworkConfig) -> Result<String, String> {
    println!("cfg: {:?}", cfg);
    eprintln!("cfg: {:?}", cfg); // 使用 eprintln!
    let toml = cfg
        .gen_config()
        .map_err(|e| format!("failed to parse peer uri: {} cfg: {:?}", e.to_string(), cfg))?;
    eprintln!("toml: {:?}", toml);
    Ok(toml.dump())
}

#[tauri::command]
pub async fn start_network_instance(app: AppHandle, cfg: NetworkConfig) -> Result<(), String> {
    if INSTANCE_MAP.contains_key(cfg.instance_id()) {
        return Err("instance already exists".to_string());
    }
    let id = cfg.instance_id().to_string();
    let cfg = cfg.gen_config().map_err(|e| e.to_string())?;
    let mut instance = NetworkInstance::new(cfg);
    instance.start().map_err(|e| e.to_string())?;

    if !EMIT_INSTANCE_INFO.load(Ordering::Relaxed) {
        EMIT_INSTANCE_INFO.store(true, Ordering::Relaxed);
        tracing::info!("instance info emit started");
        tokio::spawn(async move {
            let mut ret = vec![];
            let mut flag = 0;
            loop {
                for instance in INSTANCE_MAP.iter() {
                    if let Some(info) = instance.get_running_info() {
                        ret.push(info);
                    }
                }

                if ret.is_empty() {
                    flag += 1;
                    if flag > 5 {
                        EMIT_INSTANCE_INFO.store(false, Ordering::Relaxed);
                        tracing::info!("instance info emit stopped");
                        break;
                    }
                } else {
                    flag = 0;
                }

                let _ = app.emit("era://xvlan/mesh/info", &ret);
                ret.clear();
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    }

    INSTANCE_MAP.insert(id, instance);
    Ok(())
}

#[tauri::command]
pub fn stop_network_instance(id: String) -> Result<(), String> {
    let _ = INSTANCE_MAP.remove(&id);
    tracing::info!("instance {} stopped", id);
    Ok(())
}

#[tauri::command]
pub fn collect_network_infos() -> Result<BTreeMap<String, NetworkInstanceRunningInfo>, String> {
    let mut ret = BTreeMap::new();
    for instance in INSTANCE_MAP.iter() {
        if let Some(info) = instance.get_running_info() {
            ret.insert(instance.key().clone(), info);
        }
    }
    Ok(ret)
}

#[tauri::command]
pub fn get_os_hostname() -> Result<String, String> {
    Ok(gethostname::gethostname().to_string_lossy().to_string())
}

#[tauri::command]
pub fn set_logging_level(level: String) -> Result<(), String> {
    let lock = LOGGER_LEVEL_SENDER.get_or_init(|| Mutex::new(None));
    let sender = lock.lock().map_err(|e| e.to_string())?;

    if sender.is_none() {
        return Err("logger not initialized".to_string());
    }

    sender
        .as_ref()
        .unwrap()
        .send(level)
        .map_err(|e| e.to_string())?;
    Ok(())
    // let sender = unsafe { LOGGER_LEVEL_SENDER.as_ref().unwrap() };
    // sender.send(level).map_err(|e| e.to_string())?;
    // Ok(())
}

#[tauri::command]
pub fn set_tun_fd(instance_id: String, fd: i32) -> Result<(), String> {
    let mut instance = INSTANCE_MAP
        .get_mut(&instance_id)
        .ok_or("instance not found")?;
    instance.set_tun_fd(fd);
    Ok(())
}
#[tauri::command]
pub fn test_config(config: NetworkConfig) -> Result<NetworkConfig, String> {
    println!("{:?}", config);
    Ok(config)
}

pub fn toggle_window_visibility<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or_default() {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

pub fn check_sudo() -> bool {
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

#[cfg(debug_assertions)]
pub fn toggle_devtools<R: Runtime>(app_handle: &AppHandle<R>) -> tauri::Result<()> {
    println!("toggled devtools for app");
    #[cfg(debug_assertions)] // only include this code on debug builds
    {
        if let Some(window) = app_handle.get_webview_window("main") {
            if !window.is_devtools_open() {
                window.open_devtools();
            } else {
                window.close_devtools();
            }
        };
    }
    Ok({})
}

pub fn focus_window(app: &AppHandle) {
    let windows: std::collections::HashMap<String, tauri::WebviewWindow> = app.webview_windows();
    windows
        .values()
        .next()
        .expect("Sorry, no window found")
        .set_focus()
        .expect("Can't Bring Window to Focus");
}