use anyhow::Context as _;
use serde::Deserialize;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt as _},
    net::UnixStream,
};

use super::{WindowAddress, WorkspaceId, dispatch::Dispatcher, hyprland_rundir};

pub struct Command {
    stream: UnixStream,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub name: String,
    pub monitor: String,
    #[serde(rename = "monitorID")]
    pub monitor_id: i32,
    pub windows: i32,
    #[serde(rename = "hasfullscreen")]
    pub has_fullscreen: bool,
    #[serde(rename = "lastwindow")]
    pub last_window: WindowAddress,
    #[serde(rename = "lastwindowtitle")]
    pub last_window_title: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Client {
    pub address: WindowAddress,
    pub title: String,
    pub monitor: i32,
    pub workspace: ClientWorkspace,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ClientWorkspace {
    pub id: WorkspaceId,
    pub name: String,
}

impl Command {
    pub async fn new() -> anyhow::Result<Self> {
        let path = hyprland_rundir()?.join(".socket.sock");

        let stream = UnixStream::connect(&path)
            .await
            .context("failed to connect to event stream")?;

        Ok(Self { stream })
    }

    async fn exec(mut self, command: &str) -> io::Result<Vec<u8>> {
        self.stream.write_all(command.as_bytes()).await?;
        self.stream.flush().await?;

        let mut out = Vec::new();
        self.stream.read_to_end(&mut out).await?;

        Ok(out)
    }

    async fn json_vec<T: for<'de> Deserialize<'de>>(self, command: &str) -> io::Result<Vec<T>> {
        let out = self.exec(command).await?;

        serde_json::from_slice(&out).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    pub async fn workspaces(self) -> io::Result<Vec<Workspace>> {
        self.json_vec("j/workspaces").await
    }

    pub async fn clients(self) -> io::Result<Vec<Client>> {
        self.json_vec("j/clients").await
    }

    pub async fn dispatch(self, dispatcher: Dispatcher) -> io::Result<()> {
        self.exec(&format!("j/dispatch {dispatcher}")).await?;

        Ok(())
    }
}
