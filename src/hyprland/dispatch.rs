use std::fmt::Display;

use super::WorkspaceId;

pub enum Dispatcher {
    ChangeWorkspace(WorkspaceSpec),
}

pub enum WorkspaceSpec {
    Id(WorkspaceId),
    RelativeId(i32),
    MonitorRelativeId(i32),
    MonitorAbsoluteId(u32),
    MonitorIncludingEmptyRelativeId(i32),
    MonitorIncludingEmptyAbsoluteId(u32),
    OpenRelativeId(i32),
    OpenAbsoluteId(u32),
    Name(String),
    Previous,
    PreviousPerMonitor,
    Empty { next: bool, monitor: bool },
    Special(Option<String>),
}

impl Display for Dispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Dispatcher::ChangeWorkspace(spec) => write!(f, "workspace {}", spec),
        }
    }
}

impl Display for WorkspaceSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceSpec::Id(id) => write!(f, "{}", id.0),
            WorkspaceSpec::RelativeId(id) => write!(f, "{id:+}"),
            WorkspaceSpec::MonitorRelativeId(id) => write!(f, "m{id:+}"),
            WorkspaceSpec::MonitorAbsoluteId(id) => write!(f, "m~{id}"),
            WorkspaceSpec::MonitorIncludingEmptyRelativeId(id) => write!(f, "r{id:+}"),
            WorkspaceSpec::MonitorIncludingEmptyAbsoluteId(id) => write!(f, "r~{id}"),
            WorkspaceSpec::OpenRelativeId(id) => write!(f, "e{id:+}"),
            WorkspaceSpec::OpenAbsoluteId(id) => write!(f, "e~{id}"),
            WorkspaceSpec::Name(name) => write!(f, "name:{name}"),
            WorkspaceSpec::Previous => write!(f, "previous"),
            WorkspaceSpec::PreviousPerMonitor => write!(f, "previous_per_monitor"),
            WorkspaceSpec::Empty {
                next: false,
                monitor: false,
            } => write!(f, "empty"),
            WorkspaceSpec::Empty {
                next: true,
                monitor: false,
            } => write!(f, "emptyn"),
            WorkspaceSpec::Empty {
                next: false,
                monitor: true,
            } => write!(f, "emptym"),
            WorkspaceSpec::Empty {
                next: true,
                monitor: true,
            } => write!(f, "emptymn"),
            WorkspaceSpec::Special(None) => write!(f, "special"),
            WorkspaceSpec::Special(Some(name)) => write!(f, "special:{name}"),
        }
    }
}

impl From<WorkspaceId> for WorkspaceSpec {
    fn from(id: WorkspaceId) -> Self {
        Self::Id(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_spec() {
        assert_eq!(WorkspaceSpec::Id(WorkspaceId(1)).to_string(), "1");
        assert_eq!(WorkspaceSpec::RelativeId(1).to_string(), "+1");
        assert_eq!(WorkspaceSpec::MonitorRelativeId(1).to_string(), "m+1");
        assert_eq!(WorkspaceSpec::MonitorRelativeId(-1).to_string(), "m-1");
        assert_eq!(WorkspaceSpec::MonitorAbsoluteId(1).to_string(), "m~1");
    }
}
