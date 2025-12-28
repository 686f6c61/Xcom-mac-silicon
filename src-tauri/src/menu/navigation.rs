// X - Cliente no oficial de X (Twitter) para macOS
// Copyright © 2024 686f6c61
//
// Author: 686f6c61 (https://github.com/686f6c61)
// Repository: https://github.com/686f6c61/Xcom-mac-silicon
//
// Helpers para navegación dentro de X.com

use tauri::{AppHandle, Manager, Runtime};

/// Navega a una URL específica dentro del iframe de X
pub fn navigate_to_url<R: Runtime>(app: &AppHandle<R>, url: &str) {
    if let Some(window) = app.get_webview_window("main") {
        let js = format!(
            r#"
            const iframe = document.getElementById('twitter-frame');
            if (iframe) {{
                iframe.src = '{}';
            }}
            "#,
            url
        );

        let _ = window.eval(&js);
    }
}

/// Ejecuta un click en un elemento de X por data-testid
pub fn click_element_by_testid<R: Runtime>(app: &AppHandle<R>, testid: &str) {
    if let Some(window) = app.get_webview_window("main") {
        let js = format!(
            r#"
            const iframe = document.getElementById('twitter-frame');
            if (iframe && iframe.contentWindow) {{
                const element = iframe.contentWindow.document.querySelector('[data-testid="{}"]');
                if (element) element.click();
            }}
            "#,
            testid
        );

        let _ = window.eval(&js);
    }
}
