use std::env;
use std::sync::{Mutex, RwLock};

use keepawake::Builder as KeepAwakeBuilder;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime, WindowEvent,
};
use tauri_plugin_autostart::AutoLaunchManager;
use tauri_plugin_notification::NotificationExt;

mod backup;
mod config;
mod installation;
mod network;
mod parsing;
mod system;

use backup::BackupState;
use network::{NetworkManager, RestoreState};

pub struct SleepGuard(Mutex<Option<keepawake::KeepAwake>>);

impl SleepGuard {
    // Activation ou désactivation de la prévention de mise en veille.
    pub fn set_awake(&self, enable: bool) -> Result<(), String> {
        let mut guard = self.0.lock().map_err(|_| "Failed to lock mutex")?;

        if enable {
            // On ne crée une nouvelle instance KeepAwake que si on n'en a pas déjà une d'active.
            if guard.is_none() {
                let ka = KeepAwakeBuilder::default()
                    .display(false)
                    .idle(true)
                    .sleep(true)
                    .create()
                    .map_err(|e| format!("Failed to prevent sleep: {}", e))?;

                *guard = Some(ka);
            }
        } else {
            // Restauration du comportement de veille normal de l'OS.
            *guard = None;
        }

        Ok(())
    }
}

// Lecture des préférences de démarrage.
fn configure_window_behavior<R: Runtime>(app: &mut tauri::App<R>) {
    let handle = app.handle().clone();
    let config_state = app.state::<config::ConfigState>();

    let config = if let Ok(lock) = config_state.0.read() {
        lock.clone()
    } else {
        eprintln!("Incorrect config file, loading default one.");
        config::AppConfig::default()
    };

    // 1. Application du démarrage automatique dans l'OS.
    let autostart_manager = app.state::<AutoLaunchManager>();
    if config.general.startup {
        let _ = autostart_manager.enable();
    } else {
        let _ = autostart_manager.disable();
    }

    // Application de la prévention de mise en veille.
    if config.general.prevent_sleep {
        let sleep_state = app.state::<SleepGuard>();
        if let Err(e) = sleep_state.inner().set_awake(true) {
            eprintln!("Warning: Failed to set initial sleep state: {}", e);
        }
    }

    // On ne peut pas configurer une fenêtre qui n'existe pas, donc on sort direct si "main" est absente.
    let main_window = match app.get_webview_window("main") {
        Some(w) => w,
        None => return,
    };

    // 3. Gère si l'application doit s'ouvrir de manière visible ou rester silencieuse dans le tray system
    if config.general.start_tray {
        if let Err(e) = main_window.hide() {
            eprintln!("Failed to hide main window on startup: {}", e);
        }
    } else {
        if let Err(e) = main_window.show() {
            eprintln!("Failed to show main window on startup: {}", e);
        }
        if let Err(e) = main_window.set_focus() {
            eprintln!("Failed to set focus to main window on startup: {}", e);
        }
    }

    // 4. Intercepte l'événement de fermeture natif de la fenêtre (ex: clic sur la croix 'X').
    // Si l'utilisateur préfère minimiser dans le tray, on annule l'opération de fermeture et on cache la fenêtre.
    let handle_clone = handle.clone();
    main_window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            let state = handle_clone.state::<config::ConfigState>();

            let minimize_tray = if let Ok(lock) = state.0.read() {
                lock.general.minimize_tray
            } else {
                false
            };

            if minimize_tray {
                api.prevent_close();
                if let Some(w) = handle_clone.get_webview_window("main") {
                    if let Err(e) = w.hide() {
                        eprintln!("Failed to hide window on close: {}", e);
                    }
                }
            }
        }
    });
}

#[tauri::command]
fn set_prevent_sleep(state: tauri::State<'_, SleepGuard>, enable: bool) -> Result<(), String> {
    state.set_awake(enable)
}

// Système de notification pas encore fonctionnel.
#[tauri::command]
fn send_app_notification(app: tauri::AppHandle, title: String, body: String, _type: String) {
    if let Err(e) = app.notification().builder().title(title).body(body).show() {
        eprintln!("Failed to show notification: {}", e);
    }
}

// Correction du problème de fenêtre invisible sous Linux.
#[cfg(target_os = "linux")]
fn fix_ghost_window() {
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    std::env::set_var("WEBKIT_DISABLE_GPU_SANDBOX", "1");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "linux")]
    fix_ghost_window();

    tauri::Builder::default()
        // Initialisation des plugins de base
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        // Injection des Singletons
        .manage(SleepGuard(Mutex::new(None)))
        .manage(BackupState {
            active_pid: Mutex::new(None),
        })
        .manage(NetworkManager::new())
        .manage(RestoreState::default())
        .setup(|app| {
            // On lit la configuration une fois au démarrage et on l'injecte dans le state Tauri.
            let initial_config = config::read_from_disk(app.handle());
            app.manage(config::ConfigState(RwLock::new(initial_config)));

            // Configuration du Tray
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let mut tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false);

            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            } else {
                eprintln!("Icon missing");
            }

            let _tray = tray_builder
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // Applique toutes les règles de fenêtre configurées
            configure_window_behavior(app);

            Ok(())
        })
        // Toutes les fonctions
        .invoke_handler(tauri::generate_handler![
            // Core
            set_prevent_sleep,
            send_app_notification,
            // Sauvegarde
            backup::list_directory,
            backup::cancel_backup,
            backup::run_backup_script,
            backup::get_drives,
            backup::ask_save_path,
            // Network
            network::login_user,
            network::get_client_id_req,
            network::get_repo_key_req,
            network::send_ssh_key_req,
            network::get_server_ssh_key_req,
            network::get_logs_req,
            network::fetch_archives_list_req,
            network::fetch_archive_files_req,
            network::check_internet_connection,
            network::get_backup_logs,
            network::cancel_restore_operation,
            network::restore_to_original_req,
            network::download_and_save_archive_req,
            // System
            system::save_master_key,
            system::get_tunnel_ssh_key,
            system::get_borg_ssh_key,
            system::save_server_ssh_key,
            system::restart_computer,
            system::wsl_setup_user,
            system::wsl_provision_scripts,
            system::wsl_configure_borg_client,
            system::update_backup_schedule,
            // Installation
            installation::check_wsl_installed,
            installation::install_wsl_engine,
            installation::check_ubuntu_installed,
            installation::install_ubuntu_silent,
            installation::check_ssh_installed,
            installation::install_ssh_silent,
            installation::check_ssh_running,
            installation::start_ssh_service,
            installation::stop_ssh_service,
            installation::check_disk_space,
            // Config
            config::load_config,
            config::save_config,
            config::restart_as_admin,
            config::is_low_battery,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
