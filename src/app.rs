// SPDX-License-Identifier: GPL-3.0-only

use cosmic::app::{Command, Core};
use cosmic::iced::wayland::popup::{destroy_popup, get_popup};
use cosmic::iced::window::Id;
use cosmic::iced::Limits;
use cosmic::iced_style::application;
use cosmic::widget::{self, settings};
use cosmic::widget::{TextInput};
use cosmic::{Application, Element, Theme};
use reqwest::Error;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;

use crate::fl;

/// This is the struct that represents your application.
/// It is used to define the data that will be used by your application.
#[derive(Default)]
pub struct YourApp {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// The popup id.
    popup: Option<Id>,
    // Add a state for the text input
    input_value: String,
    // Add a state for the exchange rate
    exchange_rate: Arc<Mutex<String>>,
}

/// This is the enum that contains all the possible variants that your application will need to transmit messages.
/// This is used to communicate between the different parts of your application.
/// If your application does not need to send messages, you can use an empty enum or `()`.
#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    InputChanged(String),
}

/// Implement the `Application` trait for your application.
/// This is where you define the behavior of your application.
///
/// The `Application` trait requires you to define the following types and constants:
/// - `Executor` is the async executor that will be used to run your application's commands.
/// - `Flags` is the data that your application needs to use before it starts.
/// - `Message` is the enum that contains all the possible variants that your application will need to transmit messages.
/// - `APP_ID` is the unique identifier of your application.
impl Application for YourApp {
    type Executor = cosmic::executor::Default;

    type Flags = ();

    type Message = Message;

    const APP_ID: &'static str = "com.example.CosmicAppletTemplate";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// This is the entry point of your application, it is where you initialize your application.
    ///
    /// Any work that needs to be done before the application starts should be done here.
    ///
    /// - `core` is used to passed on for you by libcosmic to use in the core of your own application.
    /// - `flags` is used to pass in any data that your application needs to use before it starts.
    /// - `Command` type is used to send messages to your application. `Command::none()` can be used to send no messages to your application.
    fn init(core: Core, _flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let app = YourApp {
            core,
            input_value: "USDBRL".to_string(), // Set default value here
            ..Default::default()
        };

        let exchange_rate = Arc::clone(&app.exchange_rate);
        let input_value = app.input_value.clone();
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            loop {
                rt.block_on(async {
                    match fetch_exchange_rate(&input_value).await {
                        Ok(rate) => {
                            let mut exchange_rate = exchange_rate.lock().unwrap();
                            *exchange_rate = rate.trim_matches('"').to_string();
                        }
                        Err(e) => eprintln!("Error fetching exchange rate: {:?}", e),
                    }
                });
                thread::sleep(Duration::from_secs(600)); // 10 minutes
            }
        });

        (app, Command::none())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    /// This is the main view of your application, it is the root of your widget tree.
    ///
    /// The `Element` type is used to represent the visual elements of your application,
    /// it has a `Message` associated with it, which dictates what type of message it can send.
    ///
    /// To get a better sense of which widgets are available, check out the `widget` module.
    fn view(&self) -> Element<Self::Message> {
        let exchange_rate = self.exchange_rate.lock().unwrap().clone();
        cosmic::widget::button::text(exchange_rate)
            .on_press(Message::TogglePopup)
            .style(cosmic::theme::Button::AppletIcon)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<Self::Message> {
        let content_list = widget::list_column()
            .padding(5)
            .spacing(0)
            .add(settings::item(
                fl!("example-row"),
                // Shows a text input that allows the user to enter a string for the exchange rate to show.
                // For example USDEUR for USD to EUR exchange rate
                TextInput::new("Enter exchange rate", &self.input_value)
                    .on_input(Message::InputChanged)
                    .padding(10)
                    .size(20),
            ));

        self.core.applet.popup_container(content_list).into()
    }

    /// Application messages are handled here. The application state can be modified based on
    /// what message was received. Commands may be returned for asynchronous execution on a
    /// background thread managed by the application's executor.
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings =
                        self.core
                            .applet
                            .get_popup_settings(Id::MAIN, new_id, None, None, None);
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(372.0)
                        .min_width(300.0)
                        .min_height(200.0)
                        .max_height(1080.0);
                    get_popup(popup_settings)
                }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::InputChanged(new_value) => {
                self.input_value = new_value;
            }
        }
        Command::none()
    }

    fn style(&self) -> Option<<Theme as application::StyleSheet>::Style> {
        Some(cosmic::applet::style())
    }
}

async fn fetch_exchange_rate(input_value: &str) -> Result<String, Error> {
    // Get the first 3 letter from the input_value
    let from_currency = &input_value[..3];
    // Get the last 3 letter from the input_value
    let to_currency = &input_value[3..];
    let response = reqwest::get(format!(
        "https://economia.awesomeapi.com.br/last/{from_currency}-{to_currency}",
    ))
    .await?
    .json::<Value>()
    .await?;
    Ok(response[input_value]["bid"].to_string())
}
