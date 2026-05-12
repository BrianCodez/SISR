use std::{env, sync::Arc};

use winit::window::Window;
use wry::WebViewBuilder;
#[cfg(target_os = "linux")]
use wry::Rect;

use crate::app::api::get_api_port;

pub struct WebView {
    webview: wry::WebView,
    visible: bool,
}

impl WebView {
    pub fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let webview;

        #[cfg(target_os = "linux")]
        {
            let _ = gtk::init();
        }

        let webview_url = if env::var("DEV") == Ok("1".to_string()) {
            "http://localhost:5173/".to_string()
        } else {
            format!("http://localhost:{}/", get_api_port().unwrap_or(5173)) 
        };

        #[cfg(target_os = "linux")]
        let size = window.inner_size();
        #[cfg(target_os = "linux")]
        let bounds = Rect {
            position: wry::dpi::LogicalPosition::new(0.0, 0.0).into(),
            size: wry::dpi::LogicalSize::new(size.width as f64, size.height as f64).into(),
        };

        #[cfg(target_os = "linux")]
        {
            use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

            let handle = window.window_handle().map_err(|e| {
                anyhow::anyhow!("Failed to obtain window handle for webview creation: {e}")
            })?;

            webview = match handle.as_raw() {
                RawWindowHandle::Xlib(_) => {
                    let builder = WebViewBuilder::new()
                        .with_url(&webview_url)
                        .with_transparent(true)
                        .with_bounds(bounds);

                    match builder.build_as_child(window.as_ref()) {
                        Ok(wv) => wv,
                        Err(e) => {
                            tracing::warn!(
                                "Failed to build Linux child webview on X11, falling back to top-level build: {e}"
                            );
                            WebViewBuilder::new()
                                .with_url(&webview_url)
                                .with_transparent(true)
                                .build(window.as_ref())
                                .map_err(|fallback_e| {
                                    anyhow::anyhow!(
                                        "Failed to build Linux webview (child error: {e}; fallback error: {fallback_e})"
                                    )
                                })?
                        }
                    }
                }
                _ => {
                    return Err(anyhow::anyhow!(
                        "Linux Wayland window handles are not supported by wry::WebViewBuilder::build/build_as_child in this app path. \
                         Use WebViewBuilderExtUnix::build_gtk with a gtk::Fixed container for native Wayland support."
                    ));
                }
            };
        }

        #[cfg(not(target_os = "linux"))]
        {
            use std::env;

            let mut builder = WebViewBuilder::new()
                .with_url(&webview_url)
                .with_transparent(true);

            if env::var("DEV") == Ok("1".to_string()) {
                use wry::WebViewBuilderExtWindows;
                tracing::info!("Enabling remote debugging for webview");
                builder = builder.with_additional_browser_args("--remote-debugging-port=9223");
            }

            webview = builder.build(window.as_ref())?;
        }

        Ok(Self {
            webview,
            visible: true,
        })
    }

    pub fn reload(&mut self) {
        if let Err(e) = self.webview.reload() {
            tracing::warn!("Failed to reload webview: {e}");
        }
    }

    pub fn invalidate_svelte_state(&mut self) {
        if let Err(e) = self.webview.evaluate_script("window.invalidateAll();") {
            tracing::warn!("Failed to invalidate Svelte state: {e}");
        }
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn show(&mut self) {
        self.visible = true;
        if let Err(e) = self.webview.set_visible(self.visible) {
            tracing::warn!("Failed to show webview: {e}");
        }
    }
    pub fn hide(&mut self) {
        self.visible = false;
        if let Err(e) = self.webview.set_visible(self.visible) {
            tracing::warn!("Failed to hide webview by default: {e}");
        }
        if let Err(e) = self.webview.focus_parent() {
            tracing::warn!("Failed to focus parent window after hiding webview: {e}");
        }
    }

    #[cfg(target_os = "linux")]
    pub fn resize(&mut self, width: u32, height: u32) {
        if let Err(e) = self.webview.set_bounds(wry::Rect {
            position: wry::dpi::LogicalPosition::new(0.0, 0.0).into(),
            size: wry::dpi::LogicalSize::new(width as f64, height as f64).into(),
        }) {
            tracing::warn!("Failed to resize webview: {e}");
        }
    }
}
