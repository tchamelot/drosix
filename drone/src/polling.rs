use anyhow::{Context, Result};
use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Token};
use std::os::fd::AsRawFd;
use std::time::Duration;

pub struct Poller {
    inner: Poll,
    events: Events,
}

impl Poller {
    pub fn new(capacity: usize) -> Result<Self> {
        let inner = Poll::new().context("Error creating poller")?;
        let events = Events::with_capacity(capacity);
        Ok(Self {
            inner,
            events,
        })
    }

    pub fn register<T: AsRawFd>(&mut self, event: T, token: Token, interest: Interest) -> Result<()> {
        self.inner
            .registry()
            .register(&mut SourceFd(&event.as_raw_fd()), token, interest)
            .context("Error registering event")
    }

    pub fn poll(&mut self, timeout: Option<Duration>) -> Result<&Events> {
        self.inner.poll(&mut self.events, timeout).context("Error polling for events")?;
        Ok(&self.events)
    }
}
