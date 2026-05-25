//! Full demo: tab-bar navigation across Clock, Weather, and Settings screens.
//!
//! Mirrors the layout of a hardware "magic mirror" app, but renders entirely
//! through the Zest widget framework so it runs in the simulator.

extern crate alloc;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use chrono::{Datelike, Local, Timelike, Weekday};
use embassy_time::Duration;
use serde::Deserialize;
use zest::net;
use zest::prelude::*;
use zest::zest_theme::theme;
use zest::zest_widget::{WeatherCondition, WeatherIcon};

const USER_AGENT: &str = "zest-widget-demo/0.1";

const NAV_HEIGHT: u32 = 18;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScreenId {
    Clock,
    Weather,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsView {
    Menu,
    Wifi,
    Location,
    ZipEdit,
    TimeFormat,
}

#[derive(Clone)]
enum AppMessage {
    Navigate(ScreenId),
    SettingsOpen(SettingsView),
    SettingsBack,
    ToggleTimeFormat,
    ClockTick,
    RefreshWeather,
    WeatherFetched(WeatherInfo),
    WeatherError(String),
    Key(KeyAction),
}

const ZIP_MAX_LEN: usize = 5;

// The subscription layer only needs to discriminate subscription kinds;
// hashing by discriminant alone keeps `WeatherInfo` (with non-`Hash` fields
// like `WeatherCondition`) usable inside a message variant.
impl core::hash::Hash for AppMessage {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

#[derive(Debug, Clone)]
struct DailyForecast {
    day_name: String,
    high_f: Option<i32>,
    low_f: Option<i32>,
    condition: WeatherCondition,
}

#[derive(Debug, Clone)]
struct WeatherInfo {
    current_temp: i32,
    current_label: String,
    current_condition: WeatherCondition,
    forecast: Vec<DailyForecast>,
}

#[derive(Debug, Clone)]
struct Settings {
    wifi_ssid: String,
    zip_code: String,
    time_24h: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            wifi_ssid: "home-network".into(),
            zip_code: "97201".into(),
            time_24h: false,
        }
    }
}

struct MirrorScreen {
    active: ScreenId,
    settings_view: SettingsView,
    settings: Settings,
    clock_time: String,
    clock_date: String,
    weather: Option<WeatherInfo>,
    weather_status: String,
    theme: Theme<'static, Rgb565>,
    zip_input: String,
    kb_mode: KeyboardMode,
}

impl MirrorScreen {
    fn new() -> Self {
        Self {
            active: ScreenId::Clock,
            settings_view: SettingsView::Menu,
            settings: Settings::default(),
            clock_time: "--:--".into(),
            clock_date: "syncing...".into(),
            weather: None,
            weather_status: "loading weather...".into(),
            theme: convert_theme(&theme::dracula::THEME),
            zip_input: String::new(),
            kb_mode: KeyboardMode::Number,
        }
    }

    fn refresh_clock(&mut self) {
        let now = Local::now();
        self.clock_time = if self.settings.time_24h {
            format!("{:02}:{:02}", now.hour(), now.minute())
        } else {
            let h = now.hour();
            let (hour12, ampm) = if h == 0 {
                (12, "AM")
            } else if h < 12 {
                (h, "AM")
            } else if h == 12 {
                (12, "PM")
            } else {
                (h - 12, "PM")
            };
            format!("{hour12}:{:02} {ampm}", now.minute())
        };
        self.clock_date = format!(
            "{}, {} {}",
            weekday_name(now.weekday()),
            month_name(now.month()),
            now.day()
        );
    }

    fn nav_bar(&self) -> TabBar<'_, Rgb565, AppMessage> {
        TabBar::new([
            Tab::new(
                "Clock",
                AppMessage::Navigate(ScreenId::Clock),
                self.active == ScreenId::Clock,
            ),
            Tab::new(
                "Weather",
                AppMessage::Navigate(ScreenId::Weather),
                self.active == ScreenId::Weather,
            ),
            Tab::new(
                "Settings",
                AppMessage::Navigate(ScreenId::Settings),
                self.active == ScreenId::Settings,
            ),
        ])
        .height(NAV_HEIGHT)
        .spacing(4)
    }

    fn clock_view(&self) -> Element<'_, Rgb565, AppMessage> {
        Column::new()
            .spacing(6)
            .push(
                Text::new(self.clock_time.clone())
                    .align_x(Horizontal::Center)
                    .color(self.theme.background.on_base),
            )
            .push(
                Text::new(self.clock_date.clone())
                    .align_x(Horizontal::Center)
                    .color(self.theme.background.divider),
            )
            .into_element()
    }

    fn weather_view(&self) -> Element<'_, Rgb565, AppMessage> {
        let Some(weather) = &self.weather else {
            return Column::new()
                .spacing(8)
                .push(
                    Text::new(self.weather_status.clone())
                        .align_x(Horizontal::Center)
                        .color(self.theme.background.divider),
                )
                .push(
                    Button::new("Refresh")
                        .on_press(AppMessage::RefreshWeather)
                        .class(ButtonClass::Standard)
                        .height(Length::Fixed(24)),
                )
                .into_element();
        };

        let current_row = Row::new()
            .spacing(8)
            .push(
                WeatherIcon::new(weather.current_condition)
                    .width(64)
                    .height(64),
            )
            .push(
                Column::new()
                    .spacing(2)
                    .push(
                        Text::new(format!("{}°F", weather.current_temp))
                            .color(self.theme.background.on_base),
                    )
                    .push(
                        Text::new(weather.current_label.clone())
                            .color(self.theme.background.divider),
                    ),
            );

        let mut forecast_row = Row::new().spacing(4);
        for day in weather.forecast.iter().take(3) {
            forecast_row = forecast_row.push(
                Column::new()
                    .spacing(2)
                    .push(
                        Text::new(day.day_name.clone())
                            .align_x(Horizontal::Center)
                            .color(self.theme.background.divider),
                    )
                    .push(WeatherIcon::new(day.condition).width(28).height(28))
                    .push(
                        Text::new(format_high_low(day))
                            .align_x(Horizontal::Center)
                            .color(self.theme.background.on_base),
                    ),
            );
        }

        Column::new()
            .spacing(4)
            .push(current_row)
            .push(forecast_row)
            .push(
                Button::new("Refresh")
                    .on_press(AppMessage::RefreshWeather)
                    .class(ButtonClass::Standard)
                    .height(Length::Fixed(24)),
            )
            .into_element()
    }

    fn settings_view(&self) -> Element<'_, Rgb565, AppMessage> {
        match self.settings_view {
            SettingsView::Menu => Column::new()
                .spacing(6)
                .push(self.menu_button("WiFi", SettingsView::Wifi))
                .push(self.menu_button("Location", SettingsView::Location))
                .push(self.menu_button("Time Format", SettingsView::TimeFormat))
                .into_element(),
            SettingsView::Wifi => {
                self.detail_view("WiFi", &format!("SSID: {}", self.settings.wifi_ssid), None)
            }
            SettingsView::Location => self.detail_view(
                "Location",
                &format!("ZIP: {}", self.settings.zip_code),
                Some(("Edit ZIP", AppMessage::SettingsOpen(SettingsView::ZipEdit))),
            ),
            SettingsView::ZipEdit => Keyboard::new(self.kb_mode, AppMessage::Key)
                .title("Enter ZIP code")
                .input(self.zip_input.clone())
                .show_field(true)
                .into_element(),
            SettingsView::TimeFormat => {
                let current = if self.settings.time_24h {
                    "24-hour"
                } else {
                    "12-hour"
                };
                self.detail_view(
                    "Time Format",
                    &format!("Currently: {current}"),
                    Some(("Toggle", AppMessage::ToggleTimeFormat)),
                )
            }
        }
    }

    fn menu_button(
        &self,
        label: &'static str,
        target: SettingsView,
    ) -> Button<'static, Rgb565, AppMessage> {
        Button::new(label)
            .on_press(AppMessage::SettingsOpen(target))
            .class(ButtonClass::Standard)
            .height(Length::Fixed(28))
    }

    fn detail_view(
        &self,
        title: &str,
        body: &str,
        action: Option<(&'static str, AppMessage)>,
    ) -> Element<'_, Rgb565, AppMessage> {
        let mut col = Column::new()
            .spacing(6)
            .push(
                Button::new("< Back")
                    .on_press(AppMessage::SettingsBack)
                    .class(ButtonClass::Standard)
                    .height(Length::Fixed(24)),
            )
            .push(
                Text::new(title.to_string())
                    .align_x(Horizontal::Center)
                    .color(self.theme.background.on_base),
            )
            .push(
                Text::new(body.to_string())
                    .align_x(Horizontal::Center)
                    .color(self.theme.background.divider),
            );

        if let Some((label, msg)) = action {
            col = col.push(
                Button::new(label)
                    .on_press(msg)
                    .class(ButtonClass::Suggested)
                    .height(Length::Fixed(24)),
            );
        }

        col.into_element()
    }
}

impl ScreenView<Rgb565, AppMessage> for MirrorScreen {
    fn name(&self) -> &'static str {
        "magic mirror"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, AppMessage> {
        let content = match self.active {
            ScreenId::Clock => self.clock_view(),
            ScreenId::Weather => self.weather_view(),
            ScreenId::Settings => self.settings_view(),
        };

        Column::new()
            .spacing(0)
            .push(self.nav_bar())
            .push(content)
            .into_element()
    }
}

struct App {
    screen: MirrorScreen,
}

impl Application for App {
    type Message = AppMessage;
    type Color = Rgb565;
    type Screen = MirrorScreen;

    fn init() -> (Self, Task<AppMessage>) {
        let mut screen = MirrorScreen::new();
        screen.refresh_clock();
        let app = Self { screen };
        let startup = Task::future(async { AppMessage::RefreshWeather });
        (app, startup)
    }

    fn update(&mut self, msg: AppMessage) -> Task<AppMessage> {
        let s = &mut self.screen;
        match msg {
            AppMessage::Navigate(id) => {
                s.active = id;
                if id == ScreenId::Settings {
                    s.settings_view = SettingsView::Menu;
                }
                Task::none()
            }
            AppMessage::SettingsOpen(view) => {
                if view == SettingsView::ZipEdit {
                    s.zip_input = s.settings.zip_code.clone();
                    s.kb_mode = KeyboardMode::Number;
                }
                s.settings_view = view;
                Task::none()
            }
            AppMessage::SettingsBack => {
                s.settings_view = SettingsView::Menu;
                Task::none()
            }
            AppMessage::Key(action) => match action {
                KeyAction::Char(ch) => {
                    if ch.is_ascii_digit() && s.zip_input.len() < ZIP_MAX_LEN {
                        s.zip_input.push(ch);
                    }
                    Task::none()
                }
                KeyAction::Backspace => {
                    s.zip_input.pop();
                    Task::none()
                }
                KeyAction::Mode(m) => {
                    s.kb_mode = m;
                    Task::none()
                }
                // OK or Enter submits the ZIP.
                KeyAction::Ready | KeyAction::Newline => {
                    if s.zip_input.len() == ZIP_MAX_LEN
                        && s.zip_input.chars().all(|c| c.is_ascii_digit())
                    {
                        s.settings.zip_code = core::mem::take(&mut s.zip_input);
                        s.settings_view = SettingsView::Location;
                        s.weather_status = "loading weather...".into();
                        let zip = s.settings.zip_code.clone();
                        Task::future(async move {
                            match fetch_weather(zip).await {
                                Ok(info) => AppMessage::WeatherFetched(info),
                                Err(err) => AppMessage::WeatherError(err),
                            }
                        })
                    } else {
                        Task::none()
                    }
                }
                KeyAction::Cancel => {
                    s.zip_input.clear();
                    s.settings_view = SettingsView::Location;
                    Task::none()
                }
                // No cursor in this append-only ZIP field.
                KeyAction::CursorLeft | KeyAction::CursorRight => Task::none(),
                KeyAction::ToggleReveal => Task::none(),
            },
            AppMessage::ToggleTimeFormat => {
                s.settings.time_24h = !s.settings.time_24h;
                s.refresh_clock();
                Task::none()
            }
            AppMessage::ClockTick => {
                s.refresh_clock();
                Task::none()
            }
            AppMessage::RefreshWeather => {
                s.weather_status = "loading weather...".into();
                let zip = s.settings.zip_code.clone();
                Task::future(async move {
                    match fetch_weather(zip).await {
                        Ok(info) => AppMessage::WeatherFetched(info),
                        Err(err) => AppMessage::WeatherError(err),
                    }
                })
            }
            AppMessage::WeatherFetched(info) => {
                s.weather = Some(info);
                s.weather_status = "ok".into();
                Task::none()
            }
            AppMessage::WeatherError(err) => {
                s.weather = None;
                s.weather_status = format!("error: {err}");
                Task::none()
            }
        }
    }

    fn view(&self) -> &MirrorScreen {
        &self.screen
    }

    fn subscription(&self) -> Subscription<AppMessage> {
        Subscription::batch([
            zest::time::every(Duration::from_secs(1), AppMessage::ClockTick),
            zest::time::every(Duration::from_secs(300), AppMessage::RefreshWeather),
        ])
    }
}

// Pure async fetch via `zest::net`. The HTTP work runs on a hidden
// worker thread inside the helper; example code never touches threads
// or channels. On hardware, swap `zest::net::http_get_json` for
// `embassy-net` + `reqwless`.
async fn fetch_weather(zip: String) -> Result<WeatherInfo, String> {
    let zp: ZipResp = net::http_get_json(format!("https://api.zippopotam.us/us/{zip}"), USER_AGENT)
        .await
        .map_err(|e| format!("zip lookup: {e}"))?;
    let place = zp
        .places
        .first()
        .ok_or_else(|| "no places for ZIP".to_string())?;
    // NWS rejects more than 4 decimal places; truncate.
    let lat = trim_decimals(&place.latitude, 4);
    let lon = trim_decimals(&place.longitude, 4);

    let pt: NwsPoints = net::http_get_json(
        format!("https://api.weather.gov/points/{lat},{lon}"),
        USER_AGENT,
    )
    .await
    .map_err(|e| format!("points: {e}"))?;

    let resp: NwsForecast = net::http_get_json(pt.properties.forecast, USER_AGENT)
        .await
        .map_err(|e| format!("forecast: {e}"))?;

    let periods = resp.properties.periods;
    let current = periods
        .first()
        .ok_or_else(|| "no forecast periods".to_string())?;
    let forecast = build_forecast(&periods);

    Ok(WeatherInfo {
        current_temp: current.temperature,
        current_condition: classify(&current.short_forecast),
        current_label: current.short_forecast.clone(),
        forecast,
    })
}

fn trim_decimals(s: &str, places: usize) -> String {
    match s.find('.') {
        Some(dot) => {
            let end = (dot + 1 + places).min(s.len());
            s[..end].to_string()
        }
        None => s.to_string(),
    }
}

fn build_forecast(periods: &[NwsPeriod]) -> Vec<DailyForecast> {
    let mut out: Vec<DailyForecast> = Vec::new();
    let mut i = 1;
    while i < periods.len() && out.len() < 3 {
        let p = &periods[i];
        if p.is_daytime {
            let mut day = DailyForecast {
                day_name: short_day(&p.name),
                high_f: Some(p.temperature),
                low_f: None,
                condition: classify(&p.short_forecast),
            };
            if let Some(night) = periods.get(i + 1) {
                if !night.is_daytime {
                    day.low_f = Some(night.temperature);
                    i += 1;
                }
            }
            out.push(day);
        } else {
            out.push(DailyForecast {
                day_name: short_day(&p.name),
                high_f: None,
                low_f: Some(p.temperature),
                condition: classify(&p.short_forecast),
            });
        }
        i += 1;
    }
    out
}

fn short_day(name: &str) -> String {
    name.split(' ')
        .next()
        .unwrap_or(name)
        .chars()
        .take(3)
        .collect()
}

fn classify(short: &str) -> WeatherCondition {
    let s = short.to_ascii_lowercase();
    if s.contains("thunder") || s.contains("lightning") {
        WeatherCondition::Thunderstorm
    } else if s.contains("snow") || s.contains("flurr") || s.contains("blizzard") {
        WeatherCondition::Snow
    } else if s.contains("shower") {
        WeatherCondition::Showers
    } else if s.contains("rain") || s.contains("drizzle") {
        WeatherCondition::Rain
    } else if s.contains("fog") || s.contains("mist") || s.contains("haze") {
        WeatherCondition::Fog
    } else if s.contains("wind") {
        WeatherCondition::Windy
    } else if s.contains("partly") {
        WeatherCondition::PartlyCloudy
    } else if s.contains("cloud") || s.contains("overcast") {
        WeatherCondition::Cloudy
    } else if s.contains("sun") || s.contains("clear") || s.contains("fair") {
        WeatherCondition::Sunny
    } else {
        WeatherCondition::Unknown
    }
}

fn format_high_low(d: &DailyForecast) -> String {
    match (d.high_f, d.low_f) {
        (Some(h), Some(l)) => format!("{h}°/{l}°"),
        (Some(h), None) => format!("{h}°"),
        (None, Some(l)) => format!("/{l}°"),
        (None, None) => "--".into(),
    }
}

fn weekday_name(d: Weekday) -> &'static str {
    match d {
        Weekday::Mon => "Mon",
        Weekday::Tue => "Tue",
        Weekday::Wed => "Wed",
        Weekday::Thu => "Thu",
        Weekday::Fri => "Fri",
        Weekday::Sat => "Sat",
        Weekday::Sun => "Sun",
    }
}

fn month_name(m: u32) -> &'static str {
    match m {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "?",
    }
}

#[derive(Debug, Deserialize)]
struct NwsForecast {
    properties: NwsProperties,
}

#[derive(Debug, Deserialize)]
struct NwsProperties {
    periods: Vec<NwsPeriod>,
}

#[derive(Debug, Deserialize)]
struct NwsPeriod {
    name: String,
    temperature: i32,
    #[serde(rename = "shortForecast")]
    short_forecast: String,
    #[serde(rename = "isDaytime")]
    is_daytime: bool,
}

#[derive(Debug, Deserialize)]
struct ZipResp {
    places: Vec<ZipPlace>,
}

#[derive(Debug, Deserialize)]
struct ZipPlace {
    latitude: String,
    longitude: String,
}

#[derive(Debug, Deserialize)]
struct NwsPoints {
    properties: NwsPointsProps,
}

#[derive(Debug, Deserialize)]
struct NwsPointsProps {
    forecast: String,
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("Demo App").await;
}
