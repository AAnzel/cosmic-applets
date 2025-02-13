//! # DBus interface proxy for: `org.freedesktop.UPower.KbdBacklight`
//!
//! This code was generated by `zbus-xmlgen` `2.0.1` from DBus introspection data.
//! Source: `Interface '/org/freedesktop/UPower/KbdBacklight' from service 'org.freedesktop.UPower' on system bus`.

use cosmic::iced;
use iced::subscription;
use std::{fmt::Debug, hash::Hash};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use zbus::dbus_proxy;
#[dbus_proxy(
    default_service = "org.freedesktop.UPower",
    interface = "org.freedesktop.UPower.KbdBacklight",
    default_path = "/org/freedesktop/UPower/KbdBacklight"
)]
trait KbdBacklight {
    /// GetBrightness method
    fn get_brightness(&self) -> zbus::Result<i32>;

    /// GetMaxBrightness method
    fn get_max_brightness(&self) -> zbus::Result<i32>;

    /// SetBrightness method
    fn set_brightness(&self, value: i32) -> zbus::Result<()>;

    /// BrightnessChanged signal
    #[dbus_proxy(signal)]
    fn brightness_changed(&self, value: i32) -> zbus::Result<()>;

    /// BrightnessChangedWithSource signal
    #[dbus_proxy(signal)]
    fn brightness_changed_with_source(&self, value: i32, source: &str) -> zbus::Result<()>;
}

pub fn kbd_backlight_subscription<I: 'static + Hash + Copy + Send + Sync + Debug>(
    id: I,
) -> iced::Subscription<(I, KeyboardBacklightUpdate)> {
    subscription::unfold(id, State::Ready, move |state| start_listening(id, state))
}

#[derive(Debug)]
pub enum State {
    Ready,
    Waiting(
        KbdBacklightProxy<'static>,
        UnboundedReceiver<KeyboardBacklightRequest>,
    ),
    Finished,
}

async fn start_listening<I: Copy>(
    id: I,
    state: State,
) -> (Option<(I, KeyboardBacklightUpdate)>, State) {
    match state {
        State::Ready => {
            let conn = match zbus::Connection::system().await {
                Ok(conn) => conn,
                Err(_) => return (None, State::Finished),
            };
            let kbd_proxy = match KbdBacklightProxy::builder(&conn).build().await {
                Ok(p) => p,
                Err(_) => return (None, State::Finished),
            };
            let (tx, rx) = unbounded_channel();

            let b = kbd_proxy.get_brightness().await.unwrap_or_default() as f64
                / kbd_proxy.get_max_brightness().await.unwrap_or(1) as f64;
            (
                Some((id, KeyboardBacklightUpdate::Init(tx, b))),
                State::Waiting(kbd_proxy, rx),
            )
        }
        State::Waiting(proxy, mut rx) => match rx.recv().await {
            Some(req) => match req {
                KeyboardBacklightRequest::Get => {
                    let b = proxy.get_brightness().await.unwrap_or_default() as f64
                        / proxy.get_max_brightness().await.unwrap_or(1) as f64;
                    (
                        Some((id, KeyboardBacklightUpdate::Update(b))),
                        State::Waiting(proxy, rx),
                    )
                }
                KeyboardBacklightRequest::Set(value) => {
                    if let Ok(max_brightness) = proxy.get_max_brightness().await {
                        let value = value.clamp(0., 1.) * (max_brightness as f64);
                        let value = value.round() as i32;
                        let _ = proxy.set_brightness(value).await;
                    }

                    (None, State::Waiting(proxy, rx))
                }
            },
            None => (None, State::Finished),
        },
        State::Finished => iced::futures::future::pending().await,
    }
}

#[derive(Debug, Clone)]
pub enum KeyboardBacklightUpdate {
    Update(f64),
    Init(UnboundedSender<KeyboardBacklightRequest>, f64),
}

#[derive(Debug, Clone)]
pub enum KeyboardBacklightRequest {
    Get,
    Set(f64),
}
