#![feature(async_for_loop)]
#![feature(gen_blocks)]
#![feature(try_blocks)]
#![feature(yeet_expr)]

use std::collections::BTreeMap;

use iced::futures::SinkExt as _;
use iced::widget::{button, row};
use iced::{Element, Subscription, Task, Theme, stream};
use iced_layershell::Application;
use iced_layershell::actions::LayershellCustomActions;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity};
use iced_layershell::settings::{LayerShellSettings, Settings};

use self::hyprland::WorkspaceId;
use self::hyprland::commands::{Client, Workspace};
use self::hyprland::dispatch::{Dispatcher, WorkspaceSpec};
use self::hyprland::events::HyprlandEvent;

pub mod hyprland;

fn main() -> Result<(), iced_layershell::Error> {
    Bar::run(Settings {
        layer_settings: LayerShellSettings {
            size: Some((0, 30)),
            anchor: Anchor::Bottom | Anchor::Left | Anchor::Right,
            keyboard_interactivity: KeyboardInteractivity::None,
            exclusive_zone: 30,
            ..Default::default()
        },
        ..Default::default()
    })?;
    std::thread::sleep(std::time::Duration::from_millis(1));
    Ok(())
}

struct Bar {
    workspaces: BTreeMap<WorkspaceId, Workspace>,
    active_workspace: Option<WorkspaceId>,
    active_window_title: Option<String>,
    clients: Vec<Client>,
}

#[derive(Debug, Clone)]
enum Message {
    SwitchWorkspace(WorkspaceId),
    HyprlandEvent(HyprlandEvent),
    UpdateWorkspaces(Vec<Workspace>),
    UpdateClients(Vec<Client>),
}

impl TryFrom<Message> for LayershellCustomActions {
    type Error = Message;

    fn try_from(msg: Message) -> Result<Self, Self::Error> {
        Err(msg)
    }
}

fn fetch_workspaces() -> iced::Task<Message> {
    Task::future(async move {
        let out = hyprland::commands::Command::new()
            .await
            .unwrap()
            .workspaces()
            .await
            .unwrap();

        Message::UpdateWorkspaces(out)
    })
}

fn fetch_clients() -> iced::Task<Message> {
    Task::future(async move {
        let out = hyprland::commands::Command::new()
            .await
            .unwrap()
            .clients()
            .await
            .unwrap();

        Message::UpdateClients(out)
    })
}

impl Application for Bar {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Task<Message>) {
        (
            Self {
                workspaces: Default::default(),
                active_workspace: None,
                active_window_title: None,
                clients: Default::default(),
            },
            Task::batch([fetch_workspaces(), fetch_clients()]),
        )
    }

    fn namespace(&self) -> String {
        String::from("rdls")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SwitchWorkspace(id) => Task::future(async move {
                hyprland::commands::Command::new()
                    .await
                    .unwrap()
                    .dispatch(Dispatcher::ChangeWorkspace(WorkspaceSpec::Id(id)))
                    .await
                    .unwrap();
            })
            .discard(),

            Message::HyprlandEvent(HyprlandEvent::WorkspaceChanged { id, .. }) => {
                self.active_workspace = Some(id);
                Task::none()
            }

            Message::HyprlandEvent(
                HyprlandEvent::CreateWorkspace { .. }
                | HyprlandEvent::DestroyWorkspace { .. }
                | HyprlandEvent::MoveWorkspace { .. }
                | HyprlandEvent::RenameWorkspace { .. },
            ) => fetch_workspaces(),

            Message::HyprlandEvent(
                HyprlandEvent::MoveWindow { .. }
                | HyprlandEvent::OpenWindow { .. }
                | HyprlandEvent::CloseWindow { .. }
                | HyprlandEvent::WindowTitle { .. },
            ) => fetch_clients(),

            Message::HyprlandEvent(HyprlandEvent::ActiveWindow {
                address: Some(address),
            }) => {
                self.active_window_title = self
                    .clients
                    .iter()
                    .find(|client| client.address == address)
                    .map(|client| client.title.clone());

                Task::none()
            }
            Message::HyprlandEvent(HyprlandEvent::ActiveWindow { address: None }) => {
                self.active_window_title = None;
                Task::none()
            }

            Message::HyprlandEvent(_) => Task::none(),

            Message::UpdateWorkspaces(workspaces) => {
                self.workspaces = workspaces.into_iter().map(|w| (w.id, w)).collect();
                Task::none()
            }
            Message::UpdateClients(clients) => {
                self.clients = clients;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        row(self
            .workspaces
            .iter()
            .map(|(id, workspace)| {
                button(workspace.name.as_str())
                    .padding(5)
                    .style(if Some(*id) == self.active_workspace {
                        button::primary
                    } else {
                        button::secondary
                    })
                    .on_press(Message::SwitchWorkspace(*id))
                    .into()
            })
            .chain(std::iter::once(
                self.active_window_title
                    .as_deref()
                    .unwrap_or("No active window")
                    .into(),
            )))
        .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::run(|| {
            stream::channel(4, |mut tx| async move {
                for await event in hyprland::events::EventStream::listen() {
                    match event {
                        Ok(event) => tx.send(Message::HyprlandEvent(event)).await.unwrap(),
                        Err(event) => eprintln!("Error: {:?}", event),
                    }
                }
            })
        })
    }

    fn theme(&self) -> Self::Theme {
        Theme::TokyoNight
    }
}
