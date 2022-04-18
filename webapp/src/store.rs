use yewdux::prelude::*;

pub type StoreProps = DispatchProps<ReducerStore<Store>>;

pub enum Action {
    Authenticate(bool),
}

#[derive(Clone)]
pub struct Store {
    authenticated: bool,
}

impl Store {
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

impl Reducer for Store {
    type Action = Action;

    fn new() -> Self {
        log::info!("Created store");
        Self {
            authenticated: false,
        }
    }

    fn reduce(&mut self, action: Self::Action) -> Changed {
        match action {
            Action::Authenticate(auth) => self.authenticate(auth),
        }
    }
}
