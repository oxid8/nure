use crate::input::Input;
use crate::widget::{scroll, tip, url};

use service::search::Data;
use service::{
    SearchContract,
    search::{self, Entry, Result},
};

use iced::widget::{Column, button, checkbox, column, container, lazy, pick_list, row, text};
use iced::{Element, Length::Fill, Task};
use std::sync::Arc;
use strum::{Display, VariantArray};

pub struct Search<S> {
    input: Input<search::Search>,
    mode: Mode,
    order: Order,
    ascending: bool,
    exact: bool,
    limit: u8,

    state: State,
    service: Arc<S>,
}

#[derive(Default)]
enum State {
    #[default]
    None,
    Searching,
    // Aborted,
    Table(Table),
    Error(String),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Display, VariantArray)]
pub enum Mode {
    Url,
    Name,
    #[strum(to_string = "Package Base")]
    PackageBase,
    Description,
    #[strum(to_string = "Base description")]
    BaseDescription,
    #[default]
    #[strum(to_string = "Name and Description")]
    NameAndDescription,
    User,
    Flagger,
    Packager,
    Submitter,
    Maintainer,
}
impl From<Mode> for search::Mode {
    fn from(value: Mode) -> Self {
        match value {
            Mode::Url => Self::Url,
            Mode::Name => Self::Name,
            Mode::PackageBase => Self::PackageBase,
            Mode::Description => Self::Description,
            Mode::BaseDescription => Self::BaseDescription,
            Mode::NameAndDescription => Self::NameAndDescription,
            Mode::User => Self::User,
            Mode::Flagger => Self::Flagger,
            Mode::Packager => Self::Packager,
            Mode::Submitter => Self::Submitter,
            Mode::Maintainer => Self::Maintainer,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Display, VariantArray)]
pub enum Order {
    Name,
    Version,
    #[strum(to_string = "Base Name")]
    BaseName,
    // Submitter,
    #[default]
    #[strum(to_string = "Last update")]
    UpdatedAt,
    #[strum(to_string = "Created time")]
    CreatedAt,
}
impl From<Order> for search::Order {
    fn from(value: Order) -> Self {
        match value {
            Order::Name => Self::Name,
            Order::Version => Self::Version,
            Order::BaseName => Self::BaseName,
            Order::UpdatedAt => Self::UpdatedAt,
            Order::CreatedAt => Self::CreatedAt,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    // Search bar
    Reset,
    Search,
    SearchChanged(String),
    ModePicked(Mode),
    OrderPicked(Order),
    AscendingToggled(bool),
    ExactToggled(bool),
    ShowEntriesPicked(u8),
    // Table
    PackagePressed(u64),
    BasePressed(u64),
    URLPressed(Box<str>),

    RequestResult(Arc<Result<Vec<Entry>>>),
}

pub enum Event {
    Task(Task<Message>),
    OpenPackage(u64),
    OpenBase(u64),
    OpenURL(Box<str>),
}
impl From<Task<Message>> for Event {
    fn from(value: Task<Message>) -> Self {
        Self::Task(value)
    }
}

#[derive(Debug, Hash)]
struct Table(Vec<Entry>);

impl Table {
    pub fn view(&self) -> Element<'static, Message> {
        let mut table: Vec<_> = [
            "Package",      // 0
            "Version",      // 1
            "Base",         // 2
            "URL",          // 3
            "Description",  // 4
            "Last Updated", // 5
            "Created",      // 6
        ]
        .into_iter()
        .map(|s| {
            let mut v = Vec::with_capacity(self.0.len());
            v.push(s.into());
            v.push("".into());
            v
        })
        .collect();

        for entry in &self.0 {
            table[0].push(url(&entry.name, Message::PackagePressed(entry.id)));
            table[1].push(text(entry.version.to_string()).into());
            table[2].push(url(&entry.base_name, Message::BasePressed(entry.base_id)));
            table[3].push(entry.url.as_ref().map_or("-".into(), |s| {
                tip(
                    url(&"link", Message::URLPressed(s.clone())),
                    s.clone(),
                    tip::Position::Bottom,
                )
            }));
            table[4].push(text(entry.description.to_string()).into());
            table[5].push(text(entry.updated_at.to_string()).into());
            table[6].push(text(entry.created_at.to_string()).into());
            // table[5].push(Element::from(column( entry .maintainers .iter() .map(|(id, s)| url(s, Message::UserPressed(*id))),)));
        }

        scroll(
            row(table
                .into_iter()
                .map(|v| Column::from_vec(v).spacing(5).into()))
            .spacing(20)
            .padding(30),
        )
    }
}

impl<S: SearchContract + Sync + 'static> Search<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self {
            input: Input::new("search_input"),
            mode: Mode::NameAndDescription,
            order: Order::UpdatedAt,
            ascending: false,
            exact: false,
            limit: 25,
            state: State::default(),
            service,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let search_bar = container(scroll(
            column![
                row![
                    self.input
                        .view("Search")
                        .on_input(Message::SearchChanged)
                        .on_submit(Message::Search),
                    tip(
                        button("Go").on_press(Message::Search),
                        "Perform the search",
                        tip::Position::Bottom,
                    ),
                ]
                .spacing(10),
                row![
                    tip(
                        button("Reset").on_press(Message::Reset),
                        "Reset the search bar",
                        tip::Position::Bottom,
                    ),
                    tip(
                        pick_list(Mode::VARIANTS, Some(&self.mode), Message::ModePicked),
                        "Search mode",
                        tip::Position::Bottom,
                    ),
                    tip(
                        pick_list(Order::VARIANTS, Some(&self.order), Message::OrderPicked),
                        "Field used to sort the results",
                        tip::Position::Bottom,
                    ),
                    tip(
                        checkbox("Exact", self.exact).on_toggle(Message::ExactToggled),
                        "Exact search",
                        tip::Position::Bottom,
                    ),
                    tip(
                        checkbox("Ascending", self.ascending).on_toggle(Message::AscendingToggled),
                        "Sort order of results",
                        tip::Position::Bottom,
                    ),
                    tip(
                        pick_list(
                            [25, 50, 75, 100],
                            Some(self.limit),
                            Message::ShowEntriesPicked
                        ),
                        "Number of results to show",
                        tip::Position::Bottom,
                    ),
                ]
                .spacing(10)
            ]
            .padding(20)
            .width(750)
            .spacing(10),
        ))
        .center_x(Fill);

        column![
            search_bar,
            match &self.state {
                State::None => Element::from(""),
                State::Searching => "Searching...".into(),
                // State::Aborted => "Aborted".into(),
                State::Error(e) => text(e).into(),
                State::Table(table) => container(lazy(table, |t| t.view())).center_x(Fill).into(),
            }
        ]
        .into()
    }

    pub fn update(&mut self, message: Message) -> Option<Event> {
        match message {
            Message::SearchChanged(s) => self.input.update(s),
            Message::ModePicked(mode) => self.mode = mode,
            Message::OrderPicked(order) => self.order = order,
            Message::AscendingToggled(b) => self.ascending = b,
            Message::ExactToggled(b) => self.exact = b,
            Message::ShowEntriesPicked(x) => self.limit = x,
            Message::Reset => {
                let state = std::mem::take(&mut self.state);
                *self = Self::new(self.service.clone());
                self.state = state;
            }

            Message::PackagePressed(id) => return Some(Event::OpenPackage(id)),
            Message::BasePressed(id) => return Some(Event::OpenBase(id)),
            Message::URLPressed(url) => return Some(Event::OpenURL(url)),

            Message::Search => {
                let search_data = Data {
                    mode: self.mode.into(),
                    order: self.order.into(),
                    search: match self.input.submit() {
                        Ok(x) => x,
                        Err(t) => return Some(t.into()),
                    },
                    limit: self.limit.into(),
                    exact: self.exact,
                    ascending: self.ascending,
                };

                self.state = State::Searching;
                let arc = self.service.clone();

                return Some(
                    Task::perform(async move { arc.search(search_data).await }, |r| {
                        Message::RequestResult(Arc::new(r))
                    })
                    .into(),
                );
            }

            Message::RequestResult(r) => match &*r {
                Ok(v) => self.state = State::Table(Table(v.clone())),
                Err(e) => self.state = State::Error(e.to_string()),
            },
        }

        None
    }

    pub fn title(&self) -> String {
        let errors = [self.input.error(), self.input.warning()];
        let error = errors.into_iter().flatten().next();

        match &self.state {
            State::None => error.map_or_else(|| "Search".into(), Into::into),
            State::Searching => "Searching...".into(),
            State::Table(_) => "Displaying search results".into(),
            State::Error(e) => e.into(),
        }
    }
}
