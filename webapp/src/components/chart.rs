use circular_queue::CircularQueue;
use plotters::prelude::*;
use std::cell::RefCell;
use std::f32;
use std::rc::Rc;
use web_sys::HtmlCanvasElement;
use yew::prelude::*;
//use yewtil::NeqAssign;

struct NordTheme;

impl Palette for NordTheme {
    const COLORS: &'static [(u8, u8, u8)] = &[
        (0x2e, 0x34, 0x40),
        (0x3b, 0x42, 0x52),
        (0x43, 0x4c, 0x5e),
        (0x4c, 0x56, 0x6a),
        (0xd8, 0xde, 0xe9),
        (0xe5, 0xe9, 0xf0),
        (0xec, 0xef, 0xf4),
        (0x8f, 0xbc, 0xbb),
        (0x88, 0xc0, 0xd0),
        (0x81, 0xa1, 0xc1),
        (0x5e, 0x81, 0xac),
        (0xbf, 0x61, 0x6a),
        (0xd0, 0x87, 0x70),
        (0xeb, 0xcb, 0x8b),
        (0xa3, 0xbe, 0x8c),
        (0xb4, 0x8e, 0xad),
    ];
}

pub type ChartCallback = Rc<RefCell<Callback<Vec<Rc<CircularQueue<f32>>>>>>;

pub struct Chart {
    root: Option<DrawingArea<CanvasBackend, plotters::coord::Shift>>,
    node_ref: NodeRef,
    width: Option<i32>,
    height: Option<i32>,
    style: String,
    labels: Option<Vec<String>>,
}

pub enum Msg {
    NewData(Vec<Rc<CircularQueue<f32>>>),
}

#[derive(Clone, Properties)]
pub struct Props {
    #[prop_or_default]
    pub width: Option<i32>,
    #[prop_or_default]
    pub height: Option<i32>,
    #[prop_or_default]
    pub style: String,
    #[prop_or_default]
    pub labels: Option<String>,
    #[prop_or_default]
    pub cb: ChartCallback,
}

impl Component for Chart {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        *props.cb.borrow_mut() = link.callback(|data| Msg::NewData(data));
        Chart {
            root: None,
            node_ref: NodeRef::default(),
            width: props.width,
            height: props.height,
            style: props.style,
            labels: props
                .labels
                .as_ref()
                .map(|x| x.split_whitespace().collect())
                .map(|x: Vec<&str>| x.into_iter().map(|y| String::from(y)).collect()),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::NewData(data) => {
                if self.root.is_none() {
                    let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();
                    let canvas = CanvasBackend::with_canvas_object(canvas).unwrap();
                    self.root = Some(canvas.into_drawing_area());
                }
                self.draw(data.iter().map(|x| x.as_ref().asc_iter()).collect());
            }
        }
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        let mut redraw = false;
        if self.width != props.width {
            self.width = props.width;
            self.root = None;
            redraw = true;
        }
        if self.height != props.height {
            self.height = props.height;
            self.root = None;
            redraw = true;
        }
        if self.style != props.style {
            self.style = props.style;
            redraw = true;
        }
        self.labels = props
            .labels
            .as_ref()
            .map(|x| x.split_whitespace().collect())
            .map(|x: Vec<&str>| x.into_iter().map(|y| String::from(y)).collect());
        redraw
    }

    fn view(&self) -> Html {
        html! {
            <canvas ref={self.node_ref.clone()}
                width=self.width.unwrap_or(400)
                height=self.height.unwrap_or(200)
                class={self.style.as_str()}/>
        }
    }

    fn mounted(&mut self) -> ShouldRender {
        let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();
        let canvas = CanvasBackend::with_canvas_object(canvas).unwrap();
        self.root = Some(canvas.into_drawing_area());
        self.draw(vec![Vec::new().iter()]);
        true
    }
}

impl Chart {
    fn draw<'a, I: Iterator<Item = &'a f32>>(&mut self, series: Vec<I>) {
        let root = if let Some(root) = &self.root {
            root
        } else {
            log::info!("No canvas backend for Chart");
            return;
        };

        root.fill(&NordTheme::pick(0)).unwrap();

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_ranged(-1f32..1f32, -f32::consts::PI..f32::consts::PI)
            .unwrap();

        chart
            .configure_mesh()
            .x_labels(10)
            .y_labels(10)
            .axis_style(&NordTheme::pick(3))
            .line_style_1(&NordTheme::pick(3))
            .line_style_2(&NordTheme::pick(1))
            .label_style(TextStyle::from(("sans-serif", 12).into_font()).color(&NordTheme::pick(4)))
            .draw()
            .unwrap();

        for (i, serie) in series.into_iter().enumerate() {
            chart
                .draw_series(LineSeries::new(
                    (-50..=50)
                        .map(|x| x as f32 / 50.0)
                        .zip(serie)
                        .map(|(a, b)| (a, *b)),
                    &NordTheme::pick(11 + i),
                ))
                .unwrap()
                .label(
                    self.labels
                        .as_ref()
                        .and_then(|labels| labels.get(i))
                        .map_or(i.to_string().as_str(), |label| label.as_str()),
                )
                .legend(move |(x, y)| {
                    PathElement::new(vec![(x, y), (x + 20, y)], &NordTheme::pick(11 + i))
                });
        }
        chart
            .configure_series_labels()
            .position(SeriesLabelPosition::LowerRight)
            .label_font(TextStyle::from(("sans-serif", 12).into_font()).color(&NordTheme::pick(4)))
            .draw()
            .unwrap();
        root.present().unwrap();
    }
}
