//! A compact, sleek visual colour picker — a saturation/brightness square you
//! drag, a rainbow hue slider, a live preview, and an editable hex field.
//!
//! It's shared by every swatch in the UI (the Appearance palette and the alert
//! flash / gradient colours) through a single `on_change` closure that maps a
//! new `#AARRGGBB` string to whatever Message the caller wants. The original
//! 8-digit alpha of the colour is preserved — the picker only edits RGB.

use iced::widget::canvas::{self, Fill, Frame, Path, Stroke, Style};
use iced::widget::{column, container, row, slider, text_input, Space};
use iced::{Border, Color, Element, Length, Point, Rectangle, Renderer, Theme};
use std::f32::consts::FRAC_PI_2;

use crate::style::Palette;
use crate::Message;

// ── colour maths ────────────────────────────────────────────────────────────

/// `(a, r, g, b)` bytes from `#AARRGGBB` or `#RRGGBB`; mid-grey on anything odd.
fn parse_argb(hex: &str) -> (u8, u8, u8, u8) {
    let h = hex.trim_start_matches('#');
    if !h.is_ascii() {
        return (255, 128, 128, 128);
    }
    let (a, rgb) = match h.len() {
        8 => (&h[0..2], &h[2..]),
        6 => ("FF", h),
        _ => return (255, 128, 128, 128),
    };
    let p = |x: &str| u8::from_str_radix(x, 16).unwrap_or(0);
    (p(a), p(&rgb[0..2]), p(&rgb[2..4]), p(&rgb[4..6]))
}

fn fmt_argb(a: u8, r: u8, g: u8, b: u8) -> String {
    format!("#{a:02X}{r:02X}{g:02X}{b:02X}")
}

fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let (r, g, b) = (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let h = if d == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / d).rem_euclid(6.0))
    } else if max == g {
        60.0 * (((b - r) / d) + 2.0)
    } else {
        60.0 * (((r - g) / d) + 4.0)
    };
    let h = if h < 0.0 { h + 360.0 } else { h };
    let s = if max == 0.0 { 0.0 } else { d / max };
    (h, s, max)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let h6 = (h / 60.0).rem_euclid(6.0);
    let x = c * (1.0 - (h6 % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match h6 as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    )
}

fn hue_color(h: f32) -> Color {
    let (r, g, b) = hsv_to_rgb(h, 1.0, 1.0);
    Color::from_rgb8(r, g, b)
}

// ── the saturation/brightness square (interactive canvas) ────────────────────

struct SvSquare<F> {
    hue: f32,
    sat: f32,
    val: f32,
    alpha: u8,
    on_change: F,
}

impl<F> SvSquare<F>
where
    F: Fn(String) -> Message,
{
    /// Map an absolute cursor point to a colour and build the caller's Message.
    fn emit(&self, abs: Point, bounds: Rectangle) -> Message {
        let s = ((abs.x - bounds.x) / bounds.width).clamp(0.0, 1.0);
        let v = 1.0 - ((abs.y - bounds.y) / bounds.height).clamp(0.0, 1.0);
        let (r, g, b) = hsv_to_rgb(self.hue, s, v);
        (self.on_change)(fmt_argb(self.alpha, r, g, b))
    }
}

impl<F> canvas::Program<Message> for SvSquare<F>
where
    F: Fn(String) -> Message,
{
    type State = bool; // true while dragging

    fn update(
        &self,
        dragging: &mut bool,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        use canvas::event::Status;
        use iced::mouse::{Button, Event as Mouse};
        match event {
            canvas::Event::Mouse(Mouse::ButtonPressed(Button::Left)) => {
                if cursor.is_over(bounds) {
                    *dragging = true;
                    let pos = cursor.position().unwrap_or(bounds.position());
                    return (Status::Captured, Some(self.emit(pos, bounds)));
                }
                (Status::Ignored, None)
            }
            canvas::Event::Mouse(Mouse::CursorMoved { .. }) if *dragging => {
                if let Some(pos) = cursor.position() {
                    return (Status::Captured, Some(self.emit(pos, bounds)));
                }
                (Status::Ignored, None)
            }
            canvas::Event::Mouse(Mouse::ButtonReleased(Button::Left)) => {
                *dragging = false;
                (Status::Ignored, None)
            }
            _ => (Status::Ignored, None),
        }
    }

    fn draw(
        &self,
        _state: &bool,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let size = bounds.size();
        // Canvas gradients are defined by start/end points (not an angle).
        // Base: white (left, S=0) → pure hue (right, S=1).
        let sat = canvas::Gradient::Linear(
            canvas::gradient::Linear::new(Point::ORIGIN, Point::new(size.width, 0.0))
                .add_stop(0.0, Color::WHITE)
                .add_stop(1.0, hue_color(self.hue)),
        );
        frame.fill_rectangle(Point::ORIGIN, size, Fill { style: Style::Gradient(sat), ..Fill::default() });
        // Overlay: transparent (top, V=1) → black (bottom, V=0).
        let val = canvas::Gradient::Linear(
            canvas::gradient::Linear::new(Point::ORIGIN, Point::new(0.0, size.height))
                .add_stop(0.0, Color::TRANSPARENT)
                .add_stop(1.0, Color::BLACK),
        );
        frame.fill_rectangle(Point::ORIGIN, size, Fill { style: Style::Gradient(val), ..Fill::default() });
        // Marker: a white ring with a dark halo so it reads on any colour.
        let mx = self.sat * size.width;
        let my = (1.0 - self.val) * size.height;
        let c = Point::new(mx, my);
        frame.stroke(
            &Path::circle(c, 6.5),
            Stroke::default().with_width(3.5).with_color(Color { a: 0.4, ..Color::BLACK }),
        );
        frame.stroke(
            &Path::circle(c, 6.5),
            Stroke::default().with_width(2.0).with_color(Color::WHITE),
        );
        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &bool,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> iced::mouse::Interaction {
        if cursor.is_over(bounds) {
            iced::mouse::Interaction::Crosshair
        } else {
            iced::mouse::Interaction::default()
        }
    }
}

// ── the rainbow hue slider style ─────────────────────────────────────────────

fn hue_rail_style(p: Palette) -> impl Fn(&Theme, slider::Status) -> slider::Style {
    move |_t, s| {
        use iced::widget::slider::{Handle, HandleShape, Rail};
        let hot = matches!(s, slider::Status::Hovered | slider::Status::Dragged);
        let rainbow = iced::Background::Gradient(iced::Gradient::Linear(
            iced::gradient::Linear::new(iced::Radians(FRAC_PI_2))
                .add_stop(0.0, Color::from_rgb8(255, 0, 0))
                .add_stop(0.166, Color::from_rgb8(255, 255, 0))
                .add_stop(0.333, Color::from_rgb8(0, 255, 0))
                .add_stop(0.5, Color::from_rgb8(0, 255, 255))
                .add_stop(0.666, Color::from_rgb8(0, 0, 255))
                .add_stop(0.833, Color::from_rgb8(255, 0, 255))
                .add_stop(1.0, Color::from_rgb8(255, 0, 0)),
        ));
        slider::Style {
            rail: Rail {
                backgrounds: (rainbow.clone(), rainbow),
                width: 12.0,
                border: Border { radius: 6.0.into(), width: 1.0, color: Color { a: 0.3, ..p.muted } },
            },
            handle: Handle {
                shape: HandleShape::Circle { radius: if hot { 9.0 } else { 8.0 } },
                background: iced::Background::Color(Color::WHITE),
                border_width: if hot { 3.0 } else { 2.0 },
                border_color: Color { a: 0.55, ..Color::BLACK },
            },
        }
    }
}

// ── the assembled picker ──────────────────────────────────────────────────────

/// A full picker panel for the colour `hex`. `on_change` turns a new
/// `#AARRGGBB` string into the caller's Message (e.g. `SetHexColor(slot, _)`).
pub fn view<'a, F>(hex: &str, on_change: F, p: Palette) -> Element<'a, Message>
where
    F: Fn(String) -> Message + Clone + 'a,
{
    let (a, r, g, b) = parse_argb(hex);
    let (hh, ss, vv) = rgb_to_hsv(r, g, b);
    let hex_owned = hex.to_string();
    let cur = Color::from_rgba8(r, g, b, 1.0);

    // Saturation/brightness square, clipped into a rounded, bordered tile.
    let sv = canvas::Canvas::new(SvSquare { hue: hh, sat: ss, val: vv, alpha: a, on_change: on_change.clone() })
        .width(Length::Fill)
        .height(Length::Fill);
    let sv_box = container(sv)
        .width(Length::Fill)
        .height(Length::Fixed(118.0))
        .padding(0)
        .clip(true)
        .style(move |_| iced::widget::container::Style {
            border: Border { radius: 8.0.into(), width: 1.0, color: Color { a: 0.3, ..p.muted } },
            ..Default::default()
        });

    // Rainbow hue slider.
    let oc_hue = on_change.clone();
    let hue_slider = slider(0.0..=360.0, hh, move |nh| {
        let (r, g, b) = hsv_to_rgb(nh, ss, vv);
        oc_hue(fmt_argb(a, r, g, b))
    })
    .step(1.0)
    .height(18.0)
    .style(hue_rail_style(p));

    // Hex field + live preview chip.
    let oc_hex = on_change.clone();
    let hex_input = text_input("#AARRGGBB", &hex_owned)
        .size(11)
        .font(iced::Font::with_name("Consolas"))
        .width(150)
        .on_input(move |s| oc_hex(s))
        .style(crate::style::dark_input_style(p));
    let preview = container(Space::new(Length::Fixed(30.0), Length::Fixed(22.0)))
        .style(move |_| iced::widget::container::Style {
            background: Some(iced::Background::Color(cur)),
            border: Border { radius: 5.0.into(), width: 1.0, color: Color { a: 0.5, ..p.text } },
            ..Default::default()
        });

    container(
        column![
            sv_box,
            Space::with_height(10),
            hue_slider,
            Space::with_height(10),
            row![hex_input, Space::with_width(Length::Fill), preview]
                .align_y(iced::Alignment::Center),
        ]
        .spacing(0),
    )
    .padding(10)
    .style(move |_| iced::widget::container::Style {
        background: Some(iced::Background::Color(Color { a: 0.5, ..p.tile })),
        border: Border { radius: 8.0.into(), width: 1.0, color: Color { a: 0.25, ..p.muted } },
        ..Default::default()
    })
    .into()
}
