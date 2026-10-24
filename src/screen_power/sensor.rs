use super::policy::TIMEOUT;
use crate::Result;
use i2cdev::{core::I2CDevice, linux::LinuxI2CDevice};
use std::time::Instant;
use tokio::task::JoinHandle;

pub(super) fn read() -> Result<bool> {
    let mut device = LinuxI2CDevice::new("/dev/i2c-1", 0x17).map_err(|error| {
        format!("Cannot open /dev/i2c-1: {error}. Enable I²C and give your session user access to the i2c group.")
    })?;
    device.smbus_read_byte_data(0x0D).map(|byte| byte == 1).map_err(|error| {
        format!("Cannot read the EP-0106 at address 0x17: {error}. Check the board's wiring and power.")
    })
}

pub(super) struct Read {
    task: JoinHandle<(Instant, Result<bool>)>,
    deadline: Instant,
    timed_out: bool,
}

impl Read {
    pub fn start(now: Instant) -> Self {
        Self::spawn(read, now)
    }

    fn spawn(read: impl FnOnce() -> Result<bool> + Send + 'static, now: Instant) -> Self {
        Self {
            task: tokio::task::spawn_blocking(move || {
                let result = read();
                (Instant::now(), result)
            }),
            deadline: now + TIMEOUT,
            timed_out: false,
        }
    }

    pub fn finished(&self) -> bool {
        self.task.is_finished()
    }

    pub fn timeout(&mut self, now: Instant) -> bool {
        if !self.timed_out && now >= self.deadline {
            self.timed_out = true;
            true
        } else {
            false
        }
    }

    pub async fn result(self) -> Result<bool> {
        let (finished, result) = self
            .task
            .await
            .map_err(|error| format!("Sensor reader failed: {error}"))?;
        if self.timed_out || finished >= self.deadline {
            Err("Sensor read timed out. Check the EP-0106 wiring and power.".into())
        } else {
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn timed_out_read_remains_outstanding_and_its_late_result_is_discarded() {
        let (release, wait) = std::sync::mpsc::channel();
        let now = Instant::now();
        let mut read = Read::spawn(
            move || {
                wait.recv().unwrap();
                Ok(true)
            },
            now,
        );
        assert!(!read.timeout(now + TIMEOUT - std::time::Duration::from_millis(1)));
        assert!(read.timeout(now + TIMEOUT));
        assert!(!read.timeout(now + TIMEOUT));
        assert!(!read.finished());
        release.send(()).unwrap();
        assert!(read.result().await.unwrap_err().contains("timed out"));
    }
}
