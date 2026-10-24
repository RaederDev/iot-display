use crate::{
    Result,
    cache::Entry,
    config::Config,
    integrations::{Data, Provider, Request},
};

pub enum Command {
    ApplyConfig(Box<Config>),
    Refresh,
    ClearCache,
    Request(Request),
    TestConnection(Request),
    Shutdown,
}

#[derive(Clone)]
pub enum Event {
    DataUpdated {
        generation: u64,
        key: String,
        entry: Entry,
    },
    RequestFailed {
        generation: u64,
        key: String,
        provider: Provider,
        message: String,
        retry_at: i64,
    },
    ConnectionTestFinished {
        generation: u64,
        provider: Provider,
        result: Result<Data>,
    },
    Notice(String),
    ScreenPowerStatus(crate::screen_power::Status),
    CacheCleared,
}

impl Event {
    pub fn generation(&self) -> Option<u64> {
        match self {
            Self::DataUpdated { generation, .. }
            | Self::RequestFailed { generation, .. }
            | Self::ConnectionTestFinished { generation, .. } => Some(*generation),
            Self::Notice(_) | Self::ScreenPowerStatus(_) | Self::CacheCleared => None,
        }
    }
}
