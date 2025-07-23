//! This example showcases a simple native custom widget that renders using
//! arbitrary low-level geometry.

use iced::{
    Alignment::{self, Center},
    Color, Element, Length,
    widget::{self, Container, Text, button, center_x, center_y, row},
};
use test::Rainbow;

mod test {
    use std::ops::{Mul, Sub};

    use iced::Renderer;
    use iced::advanced::graphics::{color, mesh};
    use iced::advanced::layout::{self, Layout};
    use iced::advanced::renderer::{self, Quad};
    use iced::advanced::widget::{self, Tree, Widget, tree};
    use iced::widget::Container;
    use iced::{
        Color, Element, Length, Point, Rectangle, Size, Theme, Transformation, Vector, advanced,
        application, border,
    };
    use iced::{Event, mouse};

    pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r, g, b) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        println!("h:{} s:{} v:{}", h, s, v);

        println!("r{} g{} b{}", r + m, g + m, b + m);

        Color::from_rgb(r + m, g + m, b + m)
    }

    pub enum OnColorSelect<'a, Message> {
        Direct(Message),
        Closure(Box<dyn Fn() -> Message + 'a>),
    }

    impl<Message: Clone> OnColorSelect<'_, Message> {
        fn get(&self) -> Message {
            match self {
                OnColorSelect::Direct(message) => message.clone(),
                OnColorSelect::Closure(f) => f(),
            }
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct Rainbow<'a, Message> {
        hue: f32,
        on_pick_message: Option<Box<dyn Fn(Color) -> Message + 'a>>,
    }

    impl<'a, Message> Rainbow<'a, Message> {
        pub fn new(hue: f32) -> Self {
            Rainbow {
                hue,
                on_pick_message: None,
            }
        }
        pub fn on_color_pick(mut self, on_color_pick: impl Fn(Color) -> Message + 'a) -> Self {
            self.on_pick_message = Some(Box::new(on_color_pick));
            self
        }
    }

    impl<'a, Message> Widget<Message, Theme, Renderer> for Rainbow<'a, Message> {
        fn layout(
            &self,
            _tree: &mut widget::Tree,
            _renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            let width = limits.max().width;

            layout::Node::new(Size::new(width, width))
        }

        fn draw(
            &self,
            tree: &widget::Tree,
            renderer: &mut Renderer,
            _theme: &Theme,
            _style: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            _viewport: &Rectangle,
        ) {
            use iced::advanced::Renderer as _;
            use iced::advanced::graphics::mesh::{self, Mesh, Renderer as _, SolidVertex2D};

            let state = tree.state.downcast_ref::<State>();
            let bounds = layout.bounds();

            let posn_black = [bounds.width / 2.0, 0.0];
            let posn_hue = [bounds.width, bounds.height];
            let posn_white = [0.0, bounds.height];

            let mesh = Mesh::Solid {
                buffers: mesh::Indexed {
                    vertices: vec![
                        SolidVertex2D {
                            position: posn_black,
                            color: color::pack(Color::BLACK),
                        },
                        SolidVertex2D {
                            position: posn_hue,
                            color: color::pack(hsv_to_rgb(self.hue, 1.0, 1.0)),
                        },
                        SolidVertex2D {
                            position: posn_white,
                            color: color::pack(Color::WHITE),
                        },
                    ],
                    indices: vec![0, 1, 2, 1, 2, 0, 2, 0, 1],
                },
                transformation: Transformation::IDENTITY,
                clip_bounds: Rectangle::INFINITE,
            };

            renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
                renderer.draw_mesh(mesh);
            });

            renderer.with_layer(bounds, |renderer| {
                renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
                    renderer.fill_quad(
                        Quad {
                            bounds: Rectangle {
                                x: state.saturation,
                                y: state.value,
                                width: 16.0,
                                height: 16.0,
                            },
                            border: border::rounded(10),
                            ..Default::default()
                        },
                        Color::BLACK,
                    );
                });
            });
        }
        fn update(
            &mut self,
            tree: &mut Tree,
            event: &iced::Event,
            layout: Layout<'_>,
            cursor: iced::advanced::mouse::Cursor,
            _renderer: &Renderer,
            _clipboard: &mut dyn iced::advanced::Clipboard,
            shell: &mut iced::advanced::Shell<'_, Message>,
            viewport: &Rectangle,
        ) {
            let state = tree.state.downcast_mut::<State>();
            let bounds = layout.bounds();
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    state.is_dragging = true;
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    state.is_dragging = false;
                }
                Event::Mouse(mouse::Event::CursorMoved { position: _ }) => {
                    // bounds.width, bounds.height = hue
                    // bounds.width/5, 0.0 = BLACK
                    // 0.0, bounds.width = WHITE

                    if let Some(position_over) = cursor.position_in(bounds) {
                        if state.is_dragging {
                            let min_y = bounds
                                .height
                                .sub(position_over.x.mul(2.0))
                                .abs()
                                .clamp(0.0, bounds.height);

                            if position_over.y >= min_y {
                                let actual_y = position_over.y.clamp(min_y, bounds.height);
                                state.value = actual_y;
                                state.saturation = position_over.x;
                                shell.request_redraw();
                            } else {
                                state.is_dragging = false;
                            }

                            if let Some(on_stop) = &self.on_pick_message {
                                shell.publish(on_stop(hsv_to_rgb(
                                    self.hue,
                                    state.saturation / bounds.width,
                                    state.value / bounds.height,
                                )));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        fn size(&self) -> Size<Length> {
            Size {
                width: Length::Fill,
                height: Length::Shrink,
            }
        }

        fn state(&self) -> widget::tree::State {
            widget::tree::State::new(State::new(false, 0.0, 0.0))
        }
        fn tag(&self) -> tree::Tag {
            tree::Tag::of::<State>()
        }
    }

    #[derive(Debug, Clone, Copy)]
    struct State {
        is_dragging: bool,
        value: f32,
        saturation: f32, //        picker_position: Point<f32>,
    }

    impl State {
        fn new(is_dragging: bool, value: f32, saturation: f32) -> Self {
            Self {
                is_dragging,
                value,
                saturation,
            }
        }
    }

    impl Default for State {
        fn default() -> Self {
            Self {
                is_dragging: false,
                value: 0.0,
                saturation: 0.0, //              picker_position: Point::new(, ),
            }
        }
    }
    impl<'a, Message> From<Rainbow<'a, Message>> for Element<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
    {
        fn from(viewer: Rainbow<'a, Message>) -> Element<'a, Message, Theme, Renderer> {
            Element::new(viewer)
        }
    }
}

pub fn main() -> iced::Result {
    iced::run(App::update, App::view)
}

#[derive(Default)]
struct App {
    color: Color,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    ColorChange(Color),
}

impl App {
    fn view(&self) -> Element<'_, Message> {
        let color_picker = Rainbow::new(0.0).on_color_pick(|color| Message::ColorChange(color));

        let content = center_x(
            widget::Column::new()
                .push(color_picker)
                .padding(20)
                .spacing(20)
                .push(Text::new("Hello, World!").color(self.color).size(100))
                .max_width(600),
        );

        center_y(content).into()
    }
    fn update(&mut self, message: Message) {
        match message {
            Message::ColorChange(color) => self.color = color,
        }
    }
}
