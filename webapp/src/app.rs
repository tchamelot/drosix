use yew::{html, Bridge, Bridged, Callback, Component, ComponentLink, Html, ShouldRender};
use yew_router::{prelude::*, Switch};

use crate::agents::store::Store;
use crate::components::controller::Controller;
use crate::components::visualizer::Visualizer;

#[derive(Switch, Debug, Clone)]
enum AppRoute {
    #[to = "/visulizer"]
    Visualizer,
    #[to = "/controller"]
    Controller,
    #[to = "/settings"]
    Settings,
}

pub struct App {
    store: Box<dyn Bridge<Store>>,
}

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App {
            store: Store::bridge(link.callback(|_| ())),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        false
    }

    fn mounted(&mut self) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="main column">
                <nav class ="nav">
                    <RouterButton<AppRoute> route=AppRoute::Controller classes="nav-item">
                        {"Controller"}
                    </RouterButton<AppRoute>>
                    <RouterButton<AppRoute> route=AppRoute::Visualizer classes="nav-item">
                        {"Visualizer"}
                    </RouterButton<AppRoute>>
                    <RouterButton<AppRoute> route=AppRoute::Settings classes="nav-item">
                        {"Settings"}
                    </RouterButton<AppRoute>>
                </nav>
                <div class="main column">
                    <Router<AppRoute>
                        render = Router::render(|switch: AppRoute| {
                            match switch {
                                AppRoute::Visualizer => html! { <Visualizer/> },
                                AppRoute::Controller => html! { <Controller/> },
                                _ => html! {"Page not implemented yet"},
                            }
                        })
                        redirect = Router::redirect(|route: Route| {
                            AppRoute::Controller
                        })
                    />
                </div>
            </div>
        }
    }
}
