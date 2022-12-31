use nannou::image;
use nannou::prelude::*;
use nannou_egui::{self, egui, Egui};
use rayon::prelude::*;

#[allow(non_upper_case_globals)]
const SIZE_usize: usize = 720;
#[allow(non_upper_case_globals)]
const SIZE_u32: u32 = SIZE_usize as u32;
#[allow(non_upper_case_globals)]
const SIZE_f64: f64 = SIZE_usize as f64;
const PI: f64 = std::f64::consts::PI;

fn main() {
    nannou::app(model).update(update).run();
}

#[derive(PartialEq, Clone)]
struct Settings {
    n: usize,
    m: usize,
    cos: bool,
}

struct Model {
    im: image::DynamicImage,
    texture: wgpu::Texture,
    egui: Egui,
    settings: Settings,
    prev_settings: Settings,
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .size_pixels(SIZE_u32, SIZE_u32)
        .resizable(false)
        .raw_event(raw_window_event)
        .view(view)
        .build()
        .unwrap();
    let im = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
        SIZE_u32,
        SIZE_u32,
        image::Rgb([128, 128, 128]),
    ));
    let window = app.window(window_id).unwrap();
    let egui = Egui::from_window(&window);
    let settings = Settings {
        n: 1,
        m: 1,
        cos: false,
    };
    let prev_settings = settings.clone();
    let texture = wgpu::Texture::from_image(app, &im);
    Model {
        im,
        texture,
        egui,
        settings,
        prev_settings,
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    let egui = &mut model.egui;
    let settings = &mut model.settings;
    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();
    egui::Window::new("Settings").show(&ctx, |ui| {
        ui.label("n");
        ui.add(egui::Slider::new(&mut settings.n, 1..=9));
        ui.label("m");
        ui.add(egui::Slider::new(&mut settings.m, 1..=9));
        ui.add(egui::Checkbox::new(&mut settings.cos, "cos"))
    });

    if model.prev_settings != *settings {
        let Settings { n, m, cos } = settings;
        let n = *n as f64;
        let m = *m as f64;
        let cos = *cos;

        model
            .im
            .as_mut_rgb8()
            .unwrap()
            .par_chunks_exact_mut(3)
            .enumerate()
            .for_each(|(idx, rgb)| {
                let x = idx % SIZE_usize;
                let y = idx / SIZE_usize;
                let x = map_range((0.0, SIZE_f64), (-1.0, 1.0), x as f64);
                let y = map_range((0.0, SIZE_f64), (-1.0, 1.0), y as f64);
                let sin_cos_n_pi_x = sin_or_cos(n * PI * x, cos);
                let sin_cos_m_pi_y = sin_or_cos(m * PI * y, cos);
                let sin_cos_m_pi_x = sin_or_cos(m * PI * x, cos);
                let sin_cos_n_pi_y = sin_or_cos(n * PI * y, cos);
                let out = (sin_cos_n_pi_x * sin_cos_m_pi_y) - (sin_cos_m_pi_x * sin_cos_n_pi_y);
                let gray = 255 - map_range((0.0, 1.0), (0.0, 255.0), out.abs()) as u8;
                rgb[0] = gray;
                rgb[1] = gray;
                rgb[2] = gray;
            });

        model.texture = wgpu::Texture::from_image(app, &model.im);
        model.prev_settings = settings.clone()
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(BLACK);
    let draw = app.draw();
    draw.texture(&model.texture);
    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);
}

// Stolen from https://rosettacode.org/wiki/Map_range#Rust
pub fn map_range<T: Copy>(from_range: (T, T), to_range: (T, T), s: T) -> T
where
    T: std::ops::Add<T, Output = T>
        + std::ops::Sub<T, Output = T>
        + std::ops::Mul<T, Output = T>
        + std::ops::Div<T, Output = T>,
{
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}

fn sin_or_cos(f: f64, cos: bool) -> f64 {
    if cos {
        f.cos()
    } else {
        f.sin()
    }
}
