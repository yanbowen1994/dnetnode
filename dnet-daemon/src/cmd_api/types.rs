use dnet_types::status::TunnelState;
use crate::settings::Settings;

/// Trait representing something that can broadcast daemon events.
pub trait EventListener {
    /// Notify that the tunnel Status changed.
    fn notify_new_state(&self, new_state: TunnelState);

    /// Notify that the settings changed.
    fn notify_settings(&self, settings: Settings);
}