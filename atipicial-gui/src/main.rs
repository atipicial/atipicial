//! # atipicial-gui
//!
//! Operator GUI for interacting with local or remote Atipicial nodes.
//!
//! ## Boundary
//!
//! This application crate owns UI composition and must call lower service/RPC
//! APIs instead of reimplementing protocol logic.
//!
//! ## Contents
//!
//! - `client`: Client-side adapters for remote services and RPC access.
//! - `runtime`: Runtime flags, execution context state, and VM-facing support
//!   types.
//! - `screens`: Operator UI screens grouped by workflow area.
//! - `shell`: GUI application shell, event loop, and top-level window
//!   composition.
//! - `sync`: Shared synchronization helpers for GUI state.
//! - `ui`: Reusable GUI theme and widget helpers.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod client;
mod runtime;
mod screens;
mod shell;
mod ui;

use client::rpc;
use runtime::node;
use shell::app;
use ui::{theme, widgets};

use app::AtipicialGuiApp;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "atipicial_gui=info".into()),
        )
        .init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([900.0, 560.0])
            .with_title("Atipicial Node Manager"),
        ..Default::default()
    };

    eframe::run_native(
        "atipicial-gui",
        native_options,
        Box::new(|cc| Ok(Box::new(AtipicialGuiApp::new(cc)))),
    )
}
