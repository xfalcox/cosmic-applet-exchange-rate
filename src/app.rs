// SPDX-License-Identifier: GPL-3.0-only

use cosmic::app::{Core, Task};
use cosmic::iced::window::Id;
use cosmic::iced::Limits;
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::widget;
use cosmic::{Application, Element, Action};
use cosmic::iced::{Alignment, Length, Rectangle};
use cosmic::widget::rectangle_tracker::*;
use once_cell::sync::Lazy;
use cosmic::widget::autosize;
use cosmic::widget::Id as WidgetId;
use serde::Deserialize;
use std::time::Duration;
use std::io::Write;

static AUTOSIZE_MAIN_ID: Lazy<WidgetId> = Lazy::new(|| WidgetId::new("autosize-main"));

#[derive(Debug, Deserialize)]
struct ExchangeRate {
    code: String,
    codein: String,
    bid: String,
}

#[derive(Debug, Deserialize)]
struct ExchangeResponse {
    #[serde(flatten)]
    rates: std::collections::HashMap<String, ExchangeRate>,
}

/// This is the struct that represents your application.
/// It is used to define the data that will be used by your application.
#[derive(Default)]
pub struct ExchangeRateApp {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// The popup id.
    popup: Option<Id>,
    /// Current exchange rate
    exchange_rate: Option<f64>,
    /// From currency
    from_currency: String,
    /// To currency
    to_currency: String,
    /// Error message if any
    error: Option<String>,
    /// Rectangle tracker for autosize
    rectangle_tracker: Option<RectangleTracker<u32>>,
    /// Rectangle for popup positioning
    rectangle: Rectangle,
}

/// This is the enum that contains all the possible variants that your application will need to transmit messages.
/// This is used to communicate between the different parts of your application.
/// If your application does not need to send messages, you can use an empty enum or `()`.
#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    UpdateExchangeRate,
    SetFromCurrency(String),
    SetToCurrency(String),
    ExchangeRateUpdated(Result<f64, String>),
    Rectangle(RectangleUpdate<u32>),
}

/// Implement the `Application` trait for your application.
/// This is where you define the behavior of your application.
///
/// The `Application` trait requires you to define the following types and constants:
/// - `Executor` is the async executor that will be used to run your application's commands.
/// - `Flags` is the data that your application needs to use before it starts.
/// - `Message` is the enum that contains all the possible variants that your application will need to transmit messages.
/// - `APP_ID` is the unique identifier of your application.
impl Application for ExchangeRateApp {
    type Executor = cosmic::executor::Default;

    type Flags = ();

    type Message = Message;

    const APP_ID: &'static str = "com.system76.CosmicExchangeRate";

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
    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        writeln!(std::io::stderr(), "DEBUG INIT: Initializing exchange rate applet").ok();
        
        let app = ExchangeRateApp {
            core,
            from_currency: "USD".to_string(),
            to_currency: "BRL".to_string(),
            ..Default::default()
        };
        
        writeln!(std::io::stderr(), "DEBUG INIT: App initialized with currencies: {}, {}", 
                 app.from_currency, app.to_currency).ok();

        // Fetch exchange rate immediately
        let update_task = Task::perform(
            async { Message::UpdateExchangeRate },
            |msg| Action::App(msg)
        );
        
        // Also start periodic updates
        let periodic_task = Task::perform(
            async {
                tokio::time::sleep(Duration::from_secs(300)).await;
                Message::UpdateExchangeRate
            },
            |msg| Action::App(msg),
        );

        (app, Task::batch(vec![update_task, periodic_task]))
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
        // Debug print the current state
        writeln!(std::io::stderr(), "DEBUG VIEW: exchange_rate: {:?}, error: {:?}", 
                 self.exchange_rate, self.error).ok();
        
        // Create the display text for the applet - SIMPLIFIED VERSION
        let display_text = if let Some(rate) = self.exchange_rate {
            // Format with exactly 2 decimal places
            format!("${:.2}", rate)
        } else if let Some(_error) = &self.error {
            "Error".to_string()
        } else {
            "...".to_string()
        };
        
        // Log what we're displaying
        writeln!(std::io::stderr(), "DEBUG VIEW: Using display text: '{}'", display_text).ok();
        
        // Create a text widget with the exchange rate
        let exchange_text = self.core.applet.text(display_text);
        
        // Create a button with the exchange rate text
        let button = cosmic::widget::button::custom(
            widget::column()
                .push(exchange_text)
                // Make sure we don't constrain the width and let autosize handle it
                .width(Length::Shrink)
                .height(Length::Shrink)
                .align_x(Alignment::Center),
        )
        .on_press(Message::TogglePopup)
        .class(cosmic::theme::Button::AppletIcon);
            
        writeln!(std::io::stderr(), "DEBUG VIEW: Created applet text button using autosize approach").ok();
        
        // Wrap with autosize and rectangle tracker
        let element = if let Some(tracker) = self.rectangle_tracker.as_ref() {
            autosize::autosize(
                tracker.container(0, button).ignore_bounds(true),
                AUTOSIZE_MAIN_ID.clone(),
            )
        } else {
            autosize::autosize(
                button,
                AUTOSIZE_MAIN_ID.clone(),
            )
        };
        
        // Convert to element and return
        element.into()
    }

    fn view_window(&self, _id: Id) -> Element<Self::Message> {
        let content_list = widget::list_column()
            .padding(5)
            .spacing(10)
            .add(widget::text_input("From Currency", &self.from_currency)
                .on_input(Message::SetFromCurrency))
            .add(widget::text_input("To Currency", &self.to_currency)
                .on_input(Message::SetToCurrency))
            .add(widget::button::standard("Update Now")
                .on_press(Message::UpdateExchangeRate));

        self.core.applet.popup_container(content_list).into()
    }

    /// Application messages are handled here. The application state can be modified based on
    /// what message was received. Commands may be returned for asynchronous execution on a
    /// background thread managed by the application's executor.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    
                    // Use the tracked rectangle for better popup positioning
                    popup_settings.positioner.anchor_rect = Rectangle::<i32> {
                        x: self.rectangle.x.max(1.) as i32,
                        y: self.rectangle.y.max(1.) as i32,
                        width: self.rectangle.width.max(1.) as i32,
                        height: self.rectangle.height.max(1.) as i32,
                    };
                    
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
            Message::Rectangle(update) => {
                match update {
                    RectangleUpdate::Rectangle(r) => {
                        self.rectangle = r.1;
                    }
                    RectangleUpdate::Init(tracker) => {
                        self.rectangle_tracker = Some(tracker);
                    }
                }
            }
            Message::UpdateExchangeRate => {
                let from = self.from_currency.clone();
                let to = self.to_currency.clone();
                return Task::perform(
                    async move {
                        match fetch_exchange_rate(&from, &to).await {
                            Ok(rate) => Message::ExchangeRateUpdated(Ok(rate)),
                            Err(e) => Message::ExchangeRateUpdated(Err(e)),
                        }
                    },
                    |msg| Action::App(msg),
                );
            }
            Message::SetFromCurrency(currency) => {
                self.from_currency = currency.to_uppercase();
                return Task::perform(
                    async { Message::UpdateExchangeRate },
                    |msg| Action::App(msg),
                );
            }
            Message::SetToCurrency(currency) => {
                self.to_currency = currency.to_uppercase();
                return Task::perform(
                    async { Message::UpdateExchangeRate },
                    |msg| Action::App(msg),
                );
            }
            Message::ExchangeRateUpdated(result) => {
                match result {
                    Ok(rate) => {
                        writeln!(std::io::stderr(), "DEBUG UPDATE: Exchange rate updated successfully: {} (formatted as {:.2})", 
                                 rate, rate).ok();
                        self.exchange_rate = Some(rate);
                        self.error = None;
                    }
                    Err(e) => {
                        writeln!(std::io::stderr(), "DEBUG UPDATE: Exchange rate update failed: {}", e).ok();
                        self.error = Some(e);
                        self.exchange_rate = None;
                    }
                }
            }
        }
        Task::none()
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        rectangle_tracker_subscription(0).map(|e| Message::Rectangle(e.1))
    }
    
    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}

async fn fetch_exchange_rate(from: &str, to: &str) -> Result<f64, String> {
    let url = format!("https://economia.awesomeapi.com.br/last/{}-{}", from, to);
    
    // Log fetch attempt
    writeln!(std::io::stderr(), "DEBUG: Fetching exchange rate from URL: {}", url).ok();
    
    let response = reqwest::get(&url)
        .await
        .map_err(|e| {
            let err_msg = format!("Failed to fetch exchange rate: {}", e);
            writeln!(std::io::stderr(), "DEBUG: API error: {}", err_msg).ok();
            err_msg
        })?;

    // Log response status
    writeln!(std::io::stderr(), "DEBUG: Response status: {}", response.status()).ok();
    
    let response_text = response.text().await
        .map_err(|e| {
            let err_msg = format!("Failed to get response text: {}", e);
            writeln!(std::io::stderr(), "DEBUG: {}", err_msg).ok();
            err_msg
        })?;
    
    // Log raw response
    writeln!(std::io::stderr(), "DEBUG: Response text: {}", response_text).ok();
    
    // Parse the JSON manually
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&response_text);
    if let Err(e) = parsed {
        let err_msg = format!("Failed to parse JSON: {}", e);
        writeln!(std::io::stderr(), "DEBUG: {}", err_msg).ok();
        return Err(err_msg);
    }
    
    let parsed = parsed.unwrap();
    writeln!(std::io::stderr(), "DEBUG: Parsed JSON: {:?}", parsed).ok();
    
    // Try to extract the rate manually
    let key = format!("{}{}", from, to);
    writeln!(std::io::stderr(), "DEBUG: Looking for key: {}", key).ok();
    
    if let Some(rate_obj) = parsed.get(&key) {
        writeln!(std::io::stderr(), "DEBUG: Found rate object: {:?}", rate_obj).ok();
        
        if let Some(bid) = rate_obj.get("bid") {
            if let Some(bid_str) = bid.as_str() {
                writeln!(std::io::stderr(), "DEBUG: Got bid string: {}", bid_str).ok();
                
                match bid_str.parse::<f64>() {
                    Ok(rate) => {
                        writeln!(std::io::stderr(), "DEBUG: Parsed rate: {} (formatted: {:.2})", rate, rate).ok();
                        return Ok(rate);
                    },
                    Err(e) => {
                        let err_msg = format!("Failed to parse rate string '{}': {}", bid_str, e);
                        writeln!(std::io::stderr(), "DEBUG: {}", err_msg).ok();
                        return Err(err_msg);
                    }
                }
            } else {
                let err_msg = format!("Bid value is not a string: {:?}", bid);
                writeln!(std::io::stderr(), "DEBUG: {}", err_msg).ok();
                return Err(err_msg);
            }
        } else {
            let err_msg = format!("No 'bid' field in rate object");
            writeln!(std::io::stderr(), "DEBUG: {}", err_msg).ok();
            return Err(err_msg);
        }
    } else {
        let err_msg = format!("Exchange rate for {}-{} not found", from, to);
        writeln!(std::io::stderr(), "DEBUG: {}", err_msg).ok();
        return Err(err_msg);
    }
}
