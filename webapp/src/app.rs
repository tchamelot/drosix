use yew::prelude::*;
use yew_router::prelude::*;

// use crate::agents::store::Store;
// use crate::components::controller::Controller;
// use crate::components::visualizer::Visualizer;
use crate::routes::login::Login;

#[derive(Debug, Clone, PartialEq, Routable)]
enum Route {
    #[at("/login")]
    Login,
    #[at("/visulizer")]
    Visualizer,
    #[at("/controller")]
    Controller,
    #[at("/settings")]
    Settings,
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub struct App {
    // store: Box<dyn Bridge<Store>>,
}

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        App {
            //store: Store::bridge(link.callback(|_| ())),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="main column">
                <BrowserRouter>
                    <nav class ="navbar">
                        <b> {"Drosix"} </b>
                        <ul class="nav-links">
                            <input type="checkbox" id="checkbox_toggle"/>
                            <label for="checkbox_toggle" class="hamburger">{'\u{2630}'}</label>
                            <div class="menu">
                                <li><Link<Route>to={Route::Login}>{ "login" }</Link<Route>></li>
                                <li><Link<Route>to={Route::Settings}>{ "settings" }</Link<Route>></li>
                                <li><Link<Route>to={Route::Controller}>{ "control" }</Link<Route>></li>
                                <li><Link<Route>to={Route::Visualizer}>{ "monitor" }</Link<Route>></li>
                            </div>
                        </ul>
                    </nav>
                    <Switch<Route> render={Switch::render(Self::switch)}/>
                </BrowserRouter>
            </div>
            // <div class="main column">
            //     <nav class ="nav">
            //         <RouterButton<AppRoute> route=AppRoute::Controller classes="nav-item">
            //             {"Controller"}
            //         </RouterButton<AppRoute>>
            //         <RouterButton<AppRoute> route=AppRoute::Visualizer classes="nav-item">
            //             {"Visualizer"}
            //         </RouterButton<AppRoute>>
            //         <RouterButton<AppRoute> route=AppRoute::Settings classes="nav-item">
            //             {"Settings"}
            //         </RouterButton<AppRoute>>
            //         <RouterButton<AppRoute> route=AppRoute::Login classes="nav-item">
            //             {"Login"}
            //         </RouterButton<AppRoute>>
            //     </nav>
            //     <div class="main column">
            //         <Router<AppRoute>
            //             render = Router::render(|switch: AppRoute| {
            //                 match switch {
            //                     // AppRoute::Visualizer => html! { <Visualizer/> },
            //                     // AppRoute::Controller => html! { <Controller/> },
            //                     AppRoute::Login => html! { <Login/> },
            //                     _ => html! {"Page not implemented yet"},
            //                 }
            //             })
            //             redirect = Router::redirect(|route: Route| {
            //                 AppRoute::Controller
            //             })
            //         />
            //     </div>
            // </div>
        }
    }
}

impl App {
    fn switch(route: &Route) -> Html {
        match route {
            Route::Login => html! { <Login/> },
            _ => html! {"Page not implemented yet"},
        }
    }
}
