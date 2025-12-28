// X - Cliente no oficial de X (Twitter) para macOS
// Copyright © 2024 686f6c61
//
// Author: 686f6c61 (https://github.com/686f6c61)
// Repository: https://github.com/686f6c61/Xcom-mac-silicon
//
// Event handlers para menús nativos de macOS

use tauri::{AppHandle, Manager, Runtime};
use super::items::*;

/// Configura los handlers para eventos de menú
pub fn setup_menu_handlers<R: Runtime>(app: &AppHandle<R>) {
    app.on_menu_event(move |app, event| {
        let event_id = event.id().as_ref();
        tracing::info!("Menu event: {}", event_id);

        match event_id {
            // Navegación de X
            NAV_EXPLORE => navigate_to(app, URL_NAV_EXPLORE),
            NAV_GROK => navigate_to(app, URL_NAV_GROK),
            NAV_NOTIFICATIONS => navigate_to(app, URL_NAV_NOTIFICATIONS),
            NAV_MESSAGES => navigate_to(app, URL_NAV_MESSAGES),
            NAV_BOOKMARKS => navigate_to(app, URL_NAV_BOOKMARKS),
            NAV_LISTS => navigate_to(app, URL_NAV_LISTS),
            NAV_PROFILE => navigate_to(app, URL_NAV_PROFILE),

            // Scroll - no funciona con ventana directa de X.com por CORS
            SCROLL_TOP => {
                tracing::warn!("Scroll commands not supported when loading X.com directly");
            },
            SCROLL_BOTTOM => {
                tracing::warn!("Scroll commands not supported when loading X.com directly");
            },

            // Vista
            VIEW_RELOAD => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.eval("location.reload();");
                }
            },
            VIEW_FULLSCREEN => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_fullscreen(!window.is_fullscreen().unwrap_or(false));
                }
            },
            VIEW_ZOOM_IN => {
                tracing::warn!("Zoom commands not supported when loading X.com directly");
            },
            VIEW_ZOOM_OUT => {
                tracing::warn!("Zoom commands not supported when loading X.com directly");
            },
            VIEW_ZOOM_RESET => {
                tracing::warn!("Zoom commands not supported when loading X.com directly");
            },
            VIEW_TEXT_LARGER => {
                tracing::warn!("Text size commands not supported when loading X.com directly");
            },
            VIEW_TEXT_SMALLER => {
                tracing::warn!("Text size commands not supported when loading X.com directly");
            },
            VIEW_TEXT_RESET => {
                tracing::warn!("Text size commands not supported when loading X.com directly");
            },

            // Archivo
            FILE_NEW_POST => navigate_to(app, URL_COMPOSE),
            FILE_CLOSE_WINDOW => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.close();
                }
            },
            FILE_CLOSE_ALL => {
                app.exit(0);
            },

            // App
            APP_PREFERENCES => navigate_to(app, URL_SETTINGS),
            APP_TOGGLE_DARK_MODE => {
                tracing::warn!("Dark mode toggle not supported when loading X.com directly - use X settings");
            },
            APP_SWITCH_ACCOUNT => navigate_to(app, URL_NAV_PROFILE),

            // Búsqueda
            EDIT_FIND => navigate_to(app, URL_NAV_EXPLORE),

            // Check updates - Abrir directamente la página de releases
            "check_updates" => {
                let _ = tauri_plugin_opener::open_url(
                    "https://github.com/686f6c61/Xcom-mac-silicon/releases",
                    None::<String>,
                );
            },

            // Cuentas - Agregar
            ACCOUNTS_ADD => handle_add_account(app),

            // Cuentas - Eliminar cuenta activa
            ACCOUNTS_DELETE_ACTIVE => handle_delete_active_account(app),

            _ => {
                // Verificar si es un evento de cambio de cuenta
                if event_id.starts_with(ACCOUNTS_ACCOUNT_PREFIX) {
                    let username = event_id.trim_start_matches(ACCOUNTS_ACCOUNT_PREFIX);
                    handle_switch_account(app, username);
                } else {
                    tracing::warn!("Unhandled menu event: {}", event_id);
                }
            }
        }
    });
}

/// Ejecuta JavaScript en la ventana principal
/// Nota: Como la ventana carga directamente X.com, no podemos ejecutar JS arbitrario por CORS
fn execute_js<R: Runtime>(app: &AppHandle<R>, js_code: &str) {
    if let Some(window) = app.get_webview_window("main") {
        if let Err(e) = window.eval(js_code) {
            tracing::error!("Failed to execute JS: {}", e);
        }
    }
}

/// Navega la ventana a una URL específica
fn navigate_to<R: Runtime>(app: &AppHandle<R>, url: &str) {
    if let Some(window) = app.get_webview_window("main") {
        let js = format!("window.location.href = '{}';", url);
        if let Err(e) = window.eval(&js) {
            tracing::error!("Failed to navigate: {}", e);
        }
    }
}

// =============================================================================
// Handlers de Cuentas
// =============================================================================

/// Maneja "Agregar Cuenta..." - Navega al flujo de login de X
fn handle_add_account<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.eval("window.location.href = 'https://x.com/i/flow/login';");
        tracing::info!("Navigating to login page to add account");
    }
}

/// Maneja "Eliminar Cuenta Activa"
fn handle_delete_active_account<R: Runtime>(app: &AppHandle<R>) {
    let app_clone = app.clone();

    tauri::async_runtime::spawn(async move {
        match crate::accounts::get_active_account() {
            Ok(Some(username)) => {
                tracing::info!("Deleting active account: {}", username);

                if let Err(e) = crate::accounts::remove_account(&username) {
                    tracing::error!("Failed to delete account: {}", e);
                } else {
                    tracing::info!("Account deleted successfully: {}", username);

                    // Reconstruir menú
                    if let Err(e) = crate::menu::builder::rebuild_menu(&app_clone) {
                        tracing::error!("Failed to rebuild menu: {}", e);
                    }

                    // Recargar ventana
                    if let Some(window) = app_clone.get_webview_window("main") {
                        let _ = window.eval("location.reload();");
                    }
                }
            }
            Ok(None) => {
                tracing::warn!("No active account to delete");
            }
            Err(e) => {
                tracing::error!("Failed to get active account: {}", e);
            }
        }
    });
}

/// Maneja cambio de cuenta (click en @username)
fn handle_switch_account<R: Runtime>(app: &AppHandle<R>, username: &str) {
    let username = username.to_string();
    let app_clone = app.clone();

    tracing::info!("Switching to account: {}", username);

    tauri::async_runtime::spawn(async move {
        if let Err(e) = crate::accounts::set_active_account(&username) {
            tracing::error!("Failed to switch account: {}", e);
            return;
        }

        tracing::info!("Account switched successfully to: {}", username);

        // Reconstruir menú para actualizar checkmark
        if let Err(e) = crate::menu::builder::rebuild_menu(&app_clone) {
            tracing::error!("Failed to rebuild menu: {}", e);
        }

        // Recargar ventana para cargar sesión de la nueva cuenta
        if let Some(window) = app_clone.get_webview_window("main") {
            let _ = window.eval("location.reload();");
        }
    });
}
