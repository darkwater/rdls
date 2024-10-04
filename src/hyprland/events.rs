use std::str::Split;

use anyhow::Context as _;
use tokio::{
    io::{self, AsyncBufReadExt as _, BufReader},
    net::UnixStream,
};

use super::{WindowAddress, WorkspaceId, hyprland_rundir};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum HyprlandEvent {
    /// Emitted on workspace change. Is emitted ONLY when a user requests a workspace change, and
    /// is not emitted on mouse movements (see activemon)
    WorkspaceChanged { id: WorkspaceId, name: String },
    /// Emitted on the active monitor being changed.
    FocusedMonitor { name: String, workspace: String },
    /// Emitted on the active window being changed.
    ActiveWindow { address: Option<WindowAddress> },
    /// Emitted when a fullscreen status of a window changes.
    Fullscreen { enter: bool },
    /// Emitted when a monitor is removed (disconnected)
    MonitorRemoved { name: String },
    /// Emitted when a monitor is added (connected)
    MonitorAdded {
        id: WorkspaceId,
        name: String,
        description: String,
    },
    /// Emitted when a workspace is created
    CreateWorkspace { id: WorkspaceId, name: String },
    /// Emitted when a workspace is destroyed
    DestroyWorkspace { id: WorkspaceId, name: String },
    /// Emitted when a workspace is moved to a different monitor
    MoveWorkspace {
        id: WorkspaceId,
        name: String,
        monitor: String,
    },
    /// Emitted when a workspace is renamed
    RenameWorkspace { id: WorkspaceId, new_name: String },
    /// Emitted when the special workspace opened in a monitor changes (closing results in an empty
    /// WORKSPACENAME)
    ActiveSpecial { workspace: String, monitor: String },
    /// Emitted on a layout change of the active keyboard
    ActiveLayout { keyboard: String, layout: String },
    /// Emitted when a window is opened
    OpenWindow {
        address: WindowAddress,
        workspace: String,
        class: String,
        title: String,
    },
    /// Emitted when a window is closed
    CloseWindow { address: WindowAddress },
    /// Emitted when a window is moved to a workspace
    MoveWindow {
        address: WindowAddress,
        workspace_id: WorkspaceId,
        workspace: String,
    },
    /// Emitted when a layerSurface is mapped
    OpenLayer { namespace: String },
    /// Emitted when a layerSurface is unmapped
    CloseLayer { namespace: String },
    /// Emitted when a keybind submap changes. Empty means default.
    SubMap { name: String },
    /// Emitted when a window changes its floating mode. FLOATING is either 0 or 1.
    ChangeFloatingMode {
        address: WindowAddress,
        floating: bool,
    },
    /// Emitted when a window requests an urgent state
    Urgent { address: WindowAddress },
    /// Emitted when a screencopy state of a client changes. Keep in mind there might be multiple
    Screencast { state: bool, owner: ScreencastOwner },
    /// Emitted when a window title changes.
    WindowTitle {
        address: WindowAddress,
        title: String,
    },
    /// Emitted when togglegroup command is used.
    ToggleGroup {
        created: bool,
        handles: Vec<WindowAddress>,
    },
    /// Emitted when the window is merged into a group.
    MoveIntoGroup { address: WindowAddress },
    /// Emitted when the window is removed from a group.
    MoveOutOfGroup { address: WindowAddress },
    /// Emitted when ignoregrouplock is toggled.
    IgnoreGroupLock { state: bool },
    /// Emitted when lockgroups is toggled.
    LockGroups { state: bool },
    /// Emitted when the config is done reloading
    ConfigReloaded,
    /// Emitted when a window is pinned or unpinned
    Pin {
        address: WindowAddress,
        pinned: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreencastOwner {
    Monitor,
    Window,
}
pub struct EventStream;

impl EventStream {
    pub async gen fn listen() -> io::Result<HyprlandEvent> {
        let stream = try {
            let path = hyprland_rundir()?.join(".socket2.sock");

            let stream = UnixStream::connect(&path)
                .await
                .context("failed to connect to event stream")?;

            BufReader::new(stream)
        };

        let mut stream = match stream {
            Ok(stream) => stream,
            Err(e) => {
                yield Err(io::Error::new::<anyhow::Error>(io::ErrorKind::Other, e));
                return;
            }
        };

        loop {
            let mut line = String::new();
            if let Err(e) = stream.read_line(&mut line).await {
                yield Err(e);
                continue;
            }

            line.pop(); // remove newline

            let Some((event, data)) = line.split_once(">>") else {
                yield Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid event format",
                ));

                continue;
            };

            let mut data = DataParser::new(data);

            yield try {
                match event {
                    "workspacev2" => HyprlandEvent::WorkspaceChanged {
                        id: data.next_workspace_id()?,
                        name: data.next_string()?,
                    },
                    "focusedmon" => HyprlandEvent::FocusedMonitor {
                        name: data.next_string()?,
                        workspace: data.next_string()?,
                    },
                    "activewindowv2" => HyprlandEvent::ActiveWindow {
                        address: data.next_maybe_window_address()?,
                    },
                    "fullscreen" => HyprlandEvent::Fullscreen {
                        enter: data.next_bool()?,
                    },
                    "monitorremoved" => HyprlandEvent::MonitorRemoved {
                        name: data.next_string()?,
                    },
                    "monitoraddedv2" => HyprlandEvent::MonitorAdded {
                        id: data.next_workspace_id()?,
                        name: data.next_string()?,
                        description: data.next_string()?,
                    },
                    "createworkspacev2" => HyprlandEvent::CreateWorkspace {
                        id: data.next_workspace_id()?,
                        name: data.next_string()?,
                    },
                    "destroyworkspacev2" => HyprlandEvent::DestroyWorkspace {
                        id: data.next_workspace_id()?,
                        name: data.next_string()?,
                    },
                    "moveworkspacev2" => HyprlandEvent::MoveWorkspace {
                        id: data.next_workspace_id()?,
                        name: data.next_string()?,
                        monitor: data.next_string()?,
                    },
                    "renameworkspace" => HyprlandEvent::RenameWorkspace {
                        id: data.next_workspace_id()?,
                        new_name: data.next_string()?,
                    },
                    "activespecial" => HyprlandEvent::ActiveSpecial {
                        workspace: data.next_string()?,
                        monitor: data.next_string()?,
                    },
                    "activelayout" => HyprlandEvent::ActiveLayout {
                        keyboard: data.next_string()?,
                        layout: data.next_string()?,
                    },
                    "openwindow" => HyprlandEvent::OpenWindow {
                        address: data.next_window_address()?,
                        workspace: data.next_string()?,
                        class: data.next_string()?,
                        title: data.next_string()?,
                    },
                    "closewindow" => HyprlandEvent::CloseWindow {
                        address: data.next_window_address()?,
                    },
                    "movewindowv2" => HyprlandEvent::MoveWindow {
                        address: data.next_window_address()?,
                        workspace_id: data.next_workspace_id()?,
                        workspace: data.next_string()?,
                    },
                    "openlayer" => HyprlandEvent::OpenLayer {
                        namespace: data.next_string()?,
                    },
                    "closelayer" => HyprlandEvent::CloseLayer {
                        namespace: data.next_string()?,
                    },
                    "submap" => HyprlandEvent::SubMap {
                        name: data.next_string()?,
                    },
                    "changefloatingmode" => HyprlandEvent::ChangeFloatingMode {
                        address: data.next_window_address()?,
                        floating: data.next_bool()?,
                    },
                    "urgent" => HyprlandEvent::Urgent {
                        address: data.next_window_address()?,
                    },
                    "screencast" => HyprlandEvent::Screencast {
                        state: data.next_bool()?,
                        owner: match data.next_bool()? {
                            false => ScreencastOwner::Monitor,
                            true => ScreencastOwner::Window,
                        },
                    },
                    "windowtitlev2" => HyprlandEvent::WindowTitle {
                        address: data.next_window_address()?,
                        title: data.next_string()?,
                    },
                    "togglegroup" => HyprlandEvent::ToggleGroup {
                        created: data.next_bool()?,
                        handles: data.vec_window_ids()?,
                    },
                    "moveintogroup" => HyprlandEvent::MoveIntoGroup {
                        address: data.next_window_address()?,
                    },
                    "moveoutofgroup" => HyprlandEvent::MoveOutOfGroup {
                        address: data.next_window_address()?,
                    },
                    "ignoregrouplock" => HyprlandEvent::IgnoreGroupLock {
                        state: data.next_bool()?,
                    },
                    "lockgroups" => HyprlandEvent::LockGroups {
                        state: data.next_bool()?,
                    },
                    "configreloaded" => HyprlandEvent::ConfigReloaded,
                    "pin" => HyprlandEvent::Pin {
                        address: data.next_window_address()?,
                        pinned: data.next_bool()?,
                    },
                    "workspace" | "activewindow" | "monitoradded" | "createworkspace"
                    | "destroyworkspace" | "moveworkspace" | "movewindow" | "windowtitle" => {
                        // ignore old events
                        continue;
                    }
                    _ => do yeet io::Error::new(io::ErrorKind::InvalidData, "unknown event"),
                }
            };
        }
    }
}

struct DataParser<'a>(Split<'a, char>);

impl<'a> DataParser<'a> {
    fn new(data: &'a str) -> Self {
        Self(data.split(','))
    }

    fn next(&mut self) -> io::Result<&str> {
        self.0
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "unexpected end of data"))
    }

    fn next_string(&mut self) -> io::Result<String> {
        self.next().map(ToOwned::to_owned)
    }

    fn next_workspace_id(&mut self) -> io::Result<WorkspaceId> {
        i32::from_str_radix(self.next()?, 16)
            .map(WorkspaceId)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid integer"))
    }

    fn next_window_address(&mut self) -> io::Result<WindowAddress> {
        u64::from_str_radix(self.next()?, 16)
            .map(WindowAddress)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid integer"))
    }

    fn next_maybe_window_address(&mut self) -> io::Result<Option<WindowAddress>> {
        let f = self.next()?;
        if f.is_empty() {
            return Ok(None);
        }

        u64::from_str_radix(f, 16)
            .map(WindowAddress)
            .map(Some)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid integer"))
    }

    fn next_bool(&mut self) -> io::Result<bool> {
        self.next()?
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid boolean"))
    }

    fn vec_window_ids(self) -> io::Result<Vec<WindowAddress>> {
        self.0
            .map(|s| s.parse().map(WindowAddress))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid window address"))
    }
}
