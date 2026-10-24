use super::App;
use crate::{runtime::Event, ui::data::DataError};

impl App {
    pub(super) fn receive(&mut self) {
        while let Ok(event) = self.runtime.events.try_recv() {
            // Results can already be queued when Save applies a new configuration.
            if event
                .generation()
                .is_some_and(|generation| generation != self.generation)
            {
                continue;
            }
            self.widgets.event(&event);
            match event {
                Event::DataUpdated { key, entry, .. } => {
                    self.data.errors.remove(&key);
                    if self
                        .notice
                        .starts_with(&format!("{}:", entry.data.provider().name()))
                    {
                        self.notice.clear();
                    }
                    if self.notice == format!("Loading {}…", entry.data.provider().name()) {
                        self.notice.clear();
                    }
                    self.data.insert(key, entry);
                }
                Event::RequestFailed {
                    key,
                    provider,
                    message,
                    retry_at,
                    ..
                } => {
                    self.notice = format!("{}: {message}", provider.name());
                    self.data.record_error(
                        key,
                        DataError {
                            provider,
                            message,
                            retry_at,
                        },
                    );
                }
                Event::ConnectionTestFinished {
                    provider, result, ..
                } => {
                    self.notice = match &result {
                        Ok(_) => format!("{} connection succeeded", provider.name()),
                        Err(error) => format!("{}: {error}", provider.name()),
                    };
                    self.settings.test_results.insert(provider, result);
                }
                Event::Notice(message) => self.notice = message,
                Event::ScreenPowerStatus(status) => self.screen_power = status,
                Event::CacheCleared => {
                    self.data.clear();
                    self.settings.test_results.clear();
                    self.notice = "Cache cleared".into();
                }
            }
        }
    }
}
