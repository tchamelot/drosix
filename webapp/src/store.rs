use crate::api::Webrtc;
use yewdux::prelude::*;

pub type Store = ReducerStore<State>;
pub type StoreProps = DispatchProps<Store>;

pub enum Action {
    Authenticated(bool),
    WebrtcIn,
    WebrtcStatus,
}

#[derive(Clone)]
pub struct State {
    authenticated: bool,
    webrtc: Webrtc,
}

impl State {
    fn authenticate(&mut self, auth: bool) -> bool {
        if self.authenticated != auth {
            log::info!("Authenticated: {}", auth);
            self.authenticated = auth;
            true
        } else {
            false
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }
}

impl Reducer for State {
    type Action = Action;

    fn new() -> Self {
        log::info!("Created store");
        let webrtc = Webrtc::new();
        Self {
            authenticated: false,
            webrtc,
        }
    }

    fn reduce(&mut self, action: Self::Action) -> Changed {
        match action {
            Action::Authenticated(auth) => self.authenticate(auth),
            Action::WebrtcStatus => {
                log::info!("Webrtc status changed");
                false
            },
            _ => false,
        }
    }
}
