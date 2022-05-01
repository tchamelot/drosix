use yewdux::prelude::*;

pub type Store = ReducerStore<State>;
pub type StoreProps = DispatchProps<Store>;

pub enum Action {
    Authenticated(bool),
}

#[derive(Clone)]
pub struct State {
    authenticated: bool,
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
        Self {
            authenticated: false,
        }
    }

    fn reduce(&mut self, action: Self::Action) -> Changed {
        match action {
            Action::Authenticated(auth) => self.authenticate(auth),
        }
    }
}
