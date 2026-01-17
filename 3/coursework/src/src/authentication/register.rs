use crate::input::{Input, Value};
use crate::widget::centerbox;
use service::authentication::{Email, Name, Password, RegisterData};
use service::{
    Authenticated, AuthenticationContract,
    authentication::{Error, Result},
};

use iced::futures::lock::Mutex;
use iced::widget::{Space, button, checkbox, column, container, row, text};
use iced::{Length, Task, padding};
use std::sync::Arc;

pub struct Register<S> {
    name: Input<Name>,
    email: Input<Email>,
    password: Input<Password>,
    repeat: Input<String>,
    show_password: bool,

    state: State,
    service: Arc<Mutex<S>>,
}
enum State {
    None,
    Success,
    Requesting,
    Error(String),
}

pub enum Event {
    SwitchToLogin,
    Task(Task<Message>),
    Authenticated(Authenticated),
}
impl From<Task<Message>> for Event {
    fn from(value: Task<Message>) -> Self {
        Self::Task(value)
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    NameChanged(String),
    EmailChanged(String),
    PasswordChanged(String),
    RepeatChanged(String),
    ShowPasswordToggled(bool),

    EmailSubmitted,
    NameSubmitted,
    PasswordSubmitted,
    RepeatSubmitted,

    RegisterPressed,
    LoginPressed,

    RequestResult(Arc<Result<Authenticated>>),
}

impl<S: AuthenticationContract + 'static> Register<S> {
    pub fn new(service: Arc<Mutex<S>>) -> Self {
        Self {
            name: Input::new("register_name"),
            email: Input::new("register_email"),
            password: Input::new("register_password"),
            repeat: Input::new("register_repeat"),
            show_password: false,

            state: State::None,
            service,
        }
    }

    fn check_passwords(&mut self) {
        if self.password.as_ref() == self.repeat.as_ref() {
            self.repeat
                .set_value(Value::Valid(self.repeat.as_ref().to_string()));
        } else {
            self.repeat.set_error(&"passwords are different");
        }
    }

    pub fn update(&mut self, message: Message) -> Option<Event> {
        match message {
            Message::NameChanged(s) => self.name.update(s),
            Message::EmailChanged(s) => self.email.update(s),
            Message::PasswordChanged(s) => {
                self.password.update(s);
                self.check_passwords();
            }
            Message::RepeatChanged(s) => {
                self.repeat.set_value(Value::Valid(s));
                self.check_passwords();
            }
            Message::ShowPasswordToggled(b) => self.show_password = b,

            Message::NameSubmitted if self.name.critical() => (),
            Message::NameSubmitted => return Some(self.email.focus().into()),
            Message::EmailSubmitted if self.email.critical() => (),
            Message::EmailSubmitted => return Some(self.password.focus().into()),
            Message::PasswordSubmitted if self.password.critical() => (),
            Message::PasswordSubmitted => return Some(self.repeat.focus().into()),
            Message::RegisterPressed | Message::RepeatSubmitted
                if self.repeat.error().is_some() =>
            {
                return Some(self.repeat.focus().into());
            }
            Message::RegisterPressed | Message::RepeatSubmitted => {
                let register_data = RegisterData {
                    name: match self.name.submit() {
                        Ok(x) => x,
                        Err(t) => return Some(t.into()),
                    },
                    email: match self.email.submit() {
                        Ok(x) => x,
                        Err(t) => return Some(t.into()),
                    },
                    password: match self.password.submit() {
                        Ok(x) => x,
                        Err(t) => return Some(t.into()),
                    },
                };

                self.state = State::Requesting;
                let arc = self.service.clone();

                return Some(
                    Task::perform(
                        async move {
                            let Some(mut service) = arc.try_lock() else {
                                return Err(Error::Other(
                                    "other authentication request is being performed".into(),
                                ));
                            };
                            service.register(register_data).await
                        },
                        |r| Message::RequestResult(Arc::new(r)),
                    )
                    .into(),
                );
            }

            Message::LoginPressed => return Some(Event::SwitchToLogin),
            Message::RequestResult(r) => match &*r {
                Ok(a) => {
                    self.state = State::Success;
                    return Some(Event::Authenticated(a.clone()));
                }

                Err(e) => {
                    self.state = State::None;
                    match e {
                        Error::NameExists => self.name.set_warning(e),
                        Error::EmailExists => self.email.set_warning(e),
                        Error::IncorrectPassword => self.password.set_warning(e),
                        Error::InvalidPassword(_) => self.password.set_error(e),

                        _ => self.state = State::Error(e.to_string()),
                    }
                }
            },
        }

        None
    }

    pub fn view(&self) -> iced::Element<Message> {
        centerbox(
            column![
                container(text(self.title()).size(20))
                    .center_x(Length::Fill)
                    .padding(padding::bottom(10)),
                self.name
                    .view("Username")
                    .on_input(Message::NameChanged)
                    .on_submit(Message::NameSubmitted),
                self.email
                    .view("Email")
                    .on_input(Message::EmailChanged)
                    .on_submit(Message::EmailSubmitted),
                self.password
                    .view("Password")
                    .on_input(Message::PasswordChanged)
                    .on_submit(Message::PasswordSubmitted)
                    .secure(!self.show_password),
                self.repeat
                    .view("Repeat Password")
                    .on_input(Message::RepeatChanged)
                    .on_submit(Message::RepeatSubmitted)
                    .secure(!self.show_password),
                checkbox("Show password", self.show_password)
                    .on_toggle(Message::ShowPasswordToggled),
                row![
                    button(text("Login").center().size(18))
                        .on_press(Message::LoginPressed)
                        .style(button::secondary)
                        .width(Length::FillPortion(3))
                        .padding(10),
                    Space::with_width(Length::FillPortion(2)),
                    button(text("Register").center().size(18))
                        .on_press(Message::RegisterPressed)
                        .style(button::primary)
                        .width(Length::FillPortion(3))
                        .padding(10),
                ]
                .padding(padding::top(15)),
            ]
            .width(Length::Fixed(250.))
            .spacing(20),
        )
    }

    pub fn title(&self) -> String {
        let errors = [
            self.name.error(),
            self.email.error(),
            self.password.error(),
            self.repeat.error(),
            self.name.warning(),
            self.email.warning(),
            self.password.warning(),
            self.repeat.warning(),
        ];
        let error = errors.into_iter().flatten().next();

        match &self.state {
            State::None => error.map_or_else(|| "Register".into(), Into::into),
            State::Success => "Success".into(),
            State::Requesting => "Requesting...".into(),
            State::Error(e) => e.into(),
        }
    }
}
