//! Calendar with three states:
//! 1. **Month view** — grid + selected-day event list + buttons to open
//!    the day view or create a new event.
//! 2. **Day view** — scrollable 24-hour schedule. Tapping an hour row
//!    opens the editor pre-filled with that hour.
//! 3. **Event editor** — keyboard for the label + color picker +
//!    horizontal `SpinButton` time picker (hour / minute) + Save /
//!    Cancel / (Delete when editing).

extern crate alloc;

use alloc::{format, string::String, vec::Vec};
use chrono::{Datelike, Days, Local, NaiveDate};
use embedded_graphics::pixelcolor::WebColors;
use zest::prelude::*;
use zest::zest_theme::theme::dark;
use zest::zest_widget::widget::calendar::CalendarMode;
use embassy_time::Instant;

const COLORS: &[Rgb565] = &[
    Rgb565::CSS_DEEP_SKY_BLUE,
    Rgb565::CSS_GOLD,
    Rgb565::CSS_TOMATO,
    Rgb565::CSS_LIME_GREEN,
    Rgb565::CSS_MEDIUM_PURPLE,
    Rgb565::CSS_HOT_PINK,
];

#[derive(Clone)]
enum Msg {
    PrevMonth,
    NextMonth,
    PrevDay,
    NextDay,
    SelectDay(u32),
    SelectHour(u32),
    OpenDayView,
    BackToMonth,
    NewEvent,
    EditEvent(usize),
    PickColor(usize),
    HourChange(i32),
    MinuteChange(i32),
    Key(KeyAction),
    Save,
    Cancel,
    Delete,
    Scroll(ScrollMsg),
    ScrollTick,
}

#[derive(Clone)]
struct Event {
    date: NaiveDate,
    label: String,
    color_idx: usize,
    hour: u8,
    minute: u8,
}

impl Event {
    fn color(&self) -> Rgb565 {
        COLORS[self.color_idx % COLORS.len()]
    }
    fn time_label(&self) -> String {
        format!("{:02}:{:02} {}", self.hour, self.minute, self.label)
    }
}

#[derive(Clone, Copy)]
enum View {
    Month,
    Day,
    Editor { existing: Option<usize> },
}

struct Cal {
    theme: Theme<'static, Rgb565>,
    today: NaiveDate,
    month_view: NaiveDate,
    selected: NaiveDate,
    events: Vec<Event>,
    view: View,
    draft_label: String,
    draft_color: usize,
    draft_hour: i32,
    draft_minute: i32,
    kb_mode: KeyboardMode,
    day_scroll: ScrollState,
    last_tick: Instant,
}

impl Cal {
    fn new() -> Self {
        let today = Local::now().date_naive();
        let mut events = Vec::new();
        events.push(Event {
            date: today,
            label: "Wake up early".into(),
            color_idx: 0,
            hour: 7,
            minute: 0,
        });
        events.push(Event {
            date: today,
            label: "Standup".into(),
            color_idx: 1,
            hour: 9,
            minute: 30,
        });
        if let Some(d) = today.checked_add_days(Days::new(2)) {
            events.push(Event {
                date: d,
                label: "Dentist".into(),
                color_idx: 2,
                hour: 14,
                minute: 0,
            });
        }
        Self {
            theme: convert_theme(&dark::THEME),
            today,
            month_view: today,
            selected: today,
            events,
            view: View::Month,
            draft_label: String::new(),
            draft_color: 0,
            draft_hour: 9,
            draft_minute: 0,
            kb_mode: KeyboardMode::TextUpper,
            day_scroll: ScrollState::new(),
            last_tick: Instant::now(),
        }
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    let first = NaiveDate::from_ymd_opt(year, month, 1);
    match (first, next) {
        (Some(a), Some(b)) => (b - a).num_days() as u32,
        _ => 30,
    }
}

fn first_dow(year: i32, month: u32) -> u8 {
    NaiveDate::from_ymd_opt(year, month, 1)
        .map(|d| d.weekday().num_days_from_sunday() as u8)
        .unwrap_or(0)
}

fn month_name(m: u32) -> &'static str {
    [
        "January", "February", "March", "April", "May", "June",
        "July", "August", "September", "October", "November", "December",
    ][(m - 1).min(11) as usize]
}

fn dow_short(d: chrono::Weekday) -> &'static str {
    match d {
        chrono::Weekday::Mon => "Mon",
        chrono::Weekday::Tue => "Tue",
        chrono::Weekday::Wed => "Wed",
        chrono::Weekday::Thu => "Thu",
        chrono::Weekday::Fri => "Fri",
        chrono::Weekday::Sat => "Sat",
        chrono::Weekday::Sun => "Sun",
    }
}

impl Cal {
    fn day_events(&self, date: NaiveDate) -> Vec<(usize, &Event)> {
        let mut v: Vec<(usize, &Event)> = self
            .events
            .iter()
            .enumerate()
            .filter(|(_, e)| e.date == date)
            .collect();
        v.sort_by_key(|(_, e)| (e.hour, e.minute));
        v
    }

    fn month_view_element(&self) -> Element<'_, Rgb565, Msg> {
        let year = self.month_view.year();
        let month = self.month_view.month();
        let label = format!("{} {year}", month_name(month));

        let events_in_view: Vec<CalendarEvent<Rgb565>> = self
            .events
            .iter()
            .filter(|e| e.date.year() == year && e.date.month() == month)
            .map(|e| CalendarEvent {
                day: e.date.day(),
                color: e.color(),
                time: Some((e.hour, e.minute)),
                label: e.label.clone(),
            })
            .collect();

        let mut cal = Calendar::new(year, month, label)
            .days_in_month(days_in_month(year, month))
            .first_day_of_week(first_dow(year, month))
            .on_prev(Msg::PrevMonth)
            .on_next(Msg::NextMonth)
            .on_select(Msg::SelectDay)
            .events(events_in_view);
        if self.month_view.year() == self.selected.year()
            && self.month_view.month() == self.selected.month()
        {
            cal = cal.selected(self.selected.day());
        }
        if self.month_view.year() == self.today.year()
            && self.month_view.month() == self.today.month()
        {
            cal = cal.today(self.today.day());
        }

        let day_events = self.day_events(self.selected);
        let mut event_list = Column::new().spacing(1);
        event_list = event_list.push(
            Text::new(format!(
                "{} {} {}",
                dow_short(self.selected.weekday()),
                month_name(self.selected.month()),
                self.selected.day()
            ))
            .color(self.theme.background.on_base)
            .height(Length::Fixed(14)),
        );
        event_list = event_list.push(horizontal_divider());
        if day_events.is_empty() {
            event_list = event_list.push(
                Text::new("No events.").color(self.theme.background.divider),
            );
        } else {
            for (idx, ev) in &day_events {
                event_list = event_list.push(
                    Button::new(ev.time_label())
                        .on_press(Msg::EditEvent(*idx))
                        .class(ButtonClass::Text)
                        .height(Length::Fixed(14)),
                );
            }
        }

        let actions = Row::new()
            .spacing(4)
            .height(Length::Fixed(22))
            .push(
                Button::new("+ Event")
                    .on_press(Msg::NewEvent)
                    .class(ButtonClass::Suggested),
            )
            .push(
                Button::new("Day view")
                    .on_press(Msg::OpenDayView)
                    .class(ButtonClass::Standard),
            );

        Column::new()
            .spacing(2)
            .push(cal.height(Length::Fill))
            .push(horizontal_divider())
            .push(event_list.height(Length::Fixed(60)))
            .push(actions)
            .into_element()
    }

    fn day_view_element(&self) -> Element<'_, Rgb565, Msg> {
        let events_today: Vec<CalendarEvent<Rgb565>> = self
            .day_events(self.selected)
            .into_iter()
            .map(|(_, e)| CalendarEvent {
                day: e.date.day(),
                color: e.color(),
                time: Some((e.hour, e.minute)),
                label: e.label.clone(),
            })
            .collect();

        let label = format!(
            "{} {} {}",
            dow_short(self.selected.weekday()),
            month_name(self.selected.month()),
            self.selected.day()
        );
        let day_cal = Calendar::new(self.selected.year(), self.selected.month(), String::new())
            .mode(CalendarMode::Day)
            .day_label(label)
            .on_prev(Msg::PrevDay)
            .on_next(Msg::NextDay)
            .on_select(Msg::SelectHour)
            .events(events_today);

        let scroll_area = Scrollable::new(day_cal)
            .scroll_state(&self.day_scroll)
            .on_scroll(Msg::Scroll);

        let actions = Row::new()
            .spacing(4)
            .height(Length::Fixed(22))
            .push(
                Button::new("< Month")
                    .on_press(Msg::BackToMonth)
                    .class(ButtonClass::Standard),
            )
            .push(
                Button::new("+ Event")
                    .on_press(Msg::NewEvent)
                    .class(ButtonClass::Suggested),
            );

        Column::new()
            .spacing(2)
            .push(scroll_area.height(Length::Fill))
            .push(actions)
            .into_element()
    }

    fn editor_element(&self, existing: Option<usize>) -> Element<'_, Rgb565, Msg> {
        let title = if existing.is_some() {
            format!("Edit - {}/{}", self.selected.month(), self.selected.day())
        } else {
            format!("New - {}/{}", self.selected.month(), self.selected.day())
        };

        let mut swatch_row = Row::new().spacing(4).height(Length::Fixed(6));
        for color in COLORS {
            swatch_row = swatch_row.push(horizontal_divider().color(*color).thickness(6));
        }
        let mut color_row = Row::new().spacing(4).height(Length::Fixed(18));
        for i in 0..COLORS.len() {
            let label = if i == self.draft_color { "*" } else { " " };
            color_row = color_row.push(
                Button::new(label)
                    .on_press(Msg::PickColor(i))
                    .class(ButtonClass::Text),
            );
        }

        let time_row = Row::new()
            .spacing(4)
            .height(Length::Fixed(22))
            .push(
                Text::new("Time:")
                    .color(self.theme.background.on_base)
                    .width(Length::Fixed(40)),
            )
            .push(
                horizontal_spin_button(self.draft_hour)
                    .min(0)
                    .max(23)
                    .step(1)
                    .display(format!("{:02}", self.draft_hour))
                    .on_change(Msg::HourChange),
            )
            .push(
                Text::new(":")
                    .color(self.theme.background.on_base)
                    .align_x(Horizontal::Center)
                    .width(Length::Fixed(8)),
            )
            .push(
                horizontal_spin_button(self.draft_minute)
                    .min(0)
                    .max(55)
                    .step(5)
                    .display(format!("{:02}", self.draft_minute))
                    .on_change(Msg::MinuteChange),
            );

        let mut actions = Row::new().spacing(4).height(Length::Fixed(22));
        actions = actions.push(
            Button::new("Save")
                .on_press_maybe(if self.draft_label.is_empty() {
                    None
                } else {
                    Some(Msg::Save)
                })
                .class(ButtonClass::Success),
        );
        actions = actions.push(
            Button::new("Cancel")
                .on_press(Msg::Cancel)
                .class(ButtonClass::Standard),
        );
        if existing.is_some() {
            actions = actions.push(
                Button::new("Delete")
                    .on_press(Msg::Delete)
                    .class(ButtonClass::Destructive),
            );
        }

        Column::new()
            .spacing(2)
            .push(
                Text::new(title)
                    .align_x(Horizontal::Center)
                    .color(self.theme.background.on_base)
                    .height(Length::Fixed(14)),
            )
            .push(swatch_row)
            .push(color_row)
            .push(time_row)
            .push(actions)
            .push(
                Keyboard::new(self.kb_mode, Msg::Key)
                    .title("Event label")
                    .input(self.draft_label.clone())
                    .show_field(true)
                    .height(Length::Fill),
            )
            .into_element()
    }
}

impl ScreenView<Rgb565, Msg> for Cal {
    fn name(&self) -> &'static str {
        "Calendar"
    }

    fn theme(&self) -> &Theme<'_, Rgb565> {
        &self.theme
    }

    fn view(&self) -> Element<'_, Rgb565, Msg> {
        match self.view {
            View::Month => self.month_view_element(),
            View::Day => self.day_view_element(),
            View::Editor { existing } => self.editor_element(existing),
        }
    }
}

struct App {
    screen: Cal,
}

impl Application for App {
    type Message = Msg;
    type Color = Rgb565;
    type Screen = Cal;

    fn init() -> (Self, Task<Msg>) {
        (Self { screen: Cal::new() }, Task::none())
    }

    fn update(&mut self, m: Msg) -> Task<Msg> {
        let s = &mut self.screen;
        match m {
            Msg::PrevMonth => {
                let (y, mo) = if s.month_view.month() == 1 {
                    (s.month_view.year() - 1, 12)
                } else {
                    (s.month_view.year(), s.month_view.month() - 1)
                };
                if let Some(d) = NaiveDate::from_ymd_opt(y, mo, 1) {
                    s.month_view = d;
                }
            }
            Msg::NextMonth => {
                let (y, mo) = if s.month_view.month() == 12 {
                    (s.month_view.year() + 1, 1)
                } else {
                    (s.month_view.year(), s.month_view.month() + 1)
                };
                if let Some(d) = NaiveDate::from_ymd_opt(y, mo, 1) {
                    s.month_view = d;
                }
            }
            Msg::PrevDay => {
                if let Some(d) = s.selected.checked_sub_days(Days::new(1)) {
                    s.selected = d;
                }
            }
            Msg::NextDay => {
                if let Some(d) = s.selected.checked_add_days(Days::new(1)) {
                    s.selected = d;
                }
            }
            Msg::SelectDay(day) => {
                if let Some(d) =
                    NaiveDate::from_ymd_opt(s.month_view.year(), s.month_view.month(), day)
                {
                    s.selected = d;
                }
            }
            Msg::SelectHour(hour) => {
                s.draft_label.clear();
                s.draft_color = s.events.len() % COLORS.len();
                s.draft_hour = hour as i32;
                s.draft_minute = 0;
                s.kb_mode = KeyboardMode::TextUpper;
                s.view = View::Editor { existing: None };
            }
            Msg::OpenDayView => {
                s.view = View::Day;
                let mut st = ScrollState::new();
                st.offset = Point::new(0, (s.today.day() as i32).min(8) * 18);
                s.day_scroll = st;
            }
            Msg::BackToMonth => {
                s.view = View::Month;
            }
            Msg::NewEvent => {
                s.draft_label.clear();
                s.draft_color = s.events.len() % COLORS.len();
                s.draft_hour = 9;
                s.draft_minute = 0;
                s.kb_mode = KeyboardMode::TextUpper;
                s.view = View::Editor { existing: None };
            }
            Msg::EditEvent(idx) => {
                if let Some(ev) = s.events.get(idx) {
                    s.draft_label = ev.label.clone();
                    s.draft_color = ev.color_idx;
                    s.draft_hour = ev.hour as i32;
                    s.draft_minute = ev.minute as i32;
                    s.kb_mode = KeyboardMode::TextLower;
                    s.view = View::Editor { existing: Some(idx) };
                }
            }
            Msg::PickColor(i) => {
                s.draft_color = i.min(COLORS.len() - 1);
            }
            Msg::HourChange(v) => s.draft_hour = v.clamp(0, 23),
            Msg::MinuteChange(v) => s.draft_minute = v.clamp(0, 55),
            Msg::Scroll(sm) => {
                let now = Instant::now();
                let was = s.day_scroll.is_animating();
                s.day_scroll.apply(sm, now.as_millis());
                if s.day_scroll.is_animating() && !was {
                    s.last_tick = now;
                    return tick_task(Msg::ScrollTick);
                }
            }
            Msg::ScrollTick => {
                let now = Instant::now();
                let dt = (now - s.last_tick).as_millis() as u32;
                s.last_tick = now;
                s.day_scroll.tick(dt, SnapMode::None, &[]);
                if s.day_scroll.is_animating() {
                    return tick_task(Msg::ScrollTick);
                }
            }
            Msg::Key(action) => match action {
                // Uppercase is its own keymap, so the char is already cased.
                KeyAction::Char(ch) => {
                    if s.draft_label.len() < 32 {
                        s.draft_label.push(ch);
                    }
                }
                KeyAction::Backspace => {
                    s.draft_label.pop();
                }
                KeyAction::Mode(m) => s.kb_mode = m,
                // Single-line label: OK or Enter saves.
                KeyAction::Ready | KeyAction::Newline => {
                    return Task::future(async { Msg::Save });
                }
                KeyAction::Cancel => return Task::future(async { Msg::Cancel }),
                // No cursor in this append-only label field.
                KeyAction::CursorLeft | KeyAction::CursorRight => {}
            },
            Msg::Save => {
                if !s.draft_label.is_empty() {
                    if let View::Editor { existing: Some(idx) } = s.view {
                        if let Some(ev) = s.events.get_mut(idx) {
                            ev.label = s.draft_label.clone();
                            ev.color_idx = s.draft_color;
                            ev.hour = s.draft_hour as u8;
                            ev.minute = s.draft_minute as u8;
                        }
                    } else {
                        s.events.push(Event {
                            date: s.selected,
                            label: s.draft_label.clone(),
                            color_idx: s.draft_color,
                            hour: s.draft_hour as u8,
                            minute: s.draft_minute as u8,
                        });
                    }
                }
                s.draft_label.clear();
                s.view = View::Month;
            }
            Msg::Cancel => {
                s.draft_label.clear();
                s.view = View::Month;
            }
            Msg::Delete => {
                if let View::Editor { existing: Some(idx) } = s.view {
                    if idx < s.events.len() {
                        s.events.remove(idx);
                    }
                }
                s.draft_label.clear();
                s.view = View::Month;
            }
        }
        Task::none()
    }

    fn view(&self) -> &Cal {
        &self.screen
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - Calendar").await;
}
