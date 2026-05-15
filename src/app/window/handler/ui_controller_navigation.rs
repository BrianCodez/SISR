use std::mem::{Discriminant, discriminant};

use winit::event_loop::ActiveEventLoop;

use crate::app::window::{
    event::{UiControllerAction, WindowRunnerEvent},
    handler::router::EventHandler,
    runner::WindowRunner,
};

#[derive(Default)]
pub struct Handler {}

impl EventHandler for Handler {
    fn handle_event(
        &self,
        runner: &mut WindowRunner,
        _event_loop: &ActiveEventLoop,
        event: &WindowRunnerEvent,
    ) {
        let action = match event {
            WindowRunnerEvent::UiControllerAction(action) => action,
            _ => return,
        };

        let Some(webview) = runner.get_webview_mut() else {
            return;
        };

        if !webview.is_visible() {
            return;
        }

        webview.handle_controller_action(*action);
    }

    fn listen_events(&self) -> Vec<Discriminant<WindowRunnerEvent>> {
        vec![discriminant(&WindowRunnerEvent::UiControllerAction(
            UiControllerAction::Activate,
        ))]
    }
}
