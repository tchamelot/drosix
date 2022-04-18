use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

use crate::routes::Login;
use crate::store::StoreProps;

#[derive(Debug, Clone, PartialEq, Routable)]
enum Route {
    #[at("/login")]
    Login,
    #[not_found]
    #[at("/404")]
    NotFound,
}
pub struct App {}

impl Component for App {
    type Message = ();
    type Properties = StoreProps;

    fn create(_ctx: &Context<Self>) -> Self {
        App {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let authenticated = ctx.props().state().is_authenticated();
        log::info!("authenticated: {}", authenticated);
        html! {
            <BrowserRouter>
                <nav class ="navbar">
                    <b> {"Drosix"} </b>
                    <ul class="nav-links">
                        <input type="checkbox" id="checkbox_toggle"/>
                        <label for="checkbox_toggle" class="hamburger">{'\u{2630}'}</label>
                        <div class="menu">
                            if authenticated {
                            } else {
                                <li><Link<Route>to={Route::Login}>{ "Login" }</Link<Route>></li>
                            }
                        </div>
                    </ul>
                </nav>
                if authenticated {
                    <Switch<Route> render={Switch::render(Self::switch)}/>
                } else {
                    <WithDispatch<Login>/>
                }
            </BrowserRouter>
        }
    }
}

impl App {
    fn switch(route: &Route) -> Html {
        match route {
            Route::Login => html! { <Redirect<Route> to={Route::NotFound}/> },
            _ => html! {"Page not implemented yet"},
        }
    }
}
