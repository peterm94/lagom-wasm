use std::cell::RefCell;
use std::rc::Rc;

use image::RgbaImage;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::console;

use crate::renderer::{Renderer, Texture};

mod renderer;

const IMG: &[u8] = include_bytes!("../assets/bg_tileable.png");

type UpdateFn = fn(&mut LagomGame, delta: f64);

const update: UpdateFn = |game, delta| {
    console::log_1(&format!("{}", delta).into());
    game.draw(0, 4, 10);
};

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let mut game = LagomGame::new(update);

    let img = image::load_from_memory(IMG).unwrap();
    let img = img.to_rgba8();
    game.load_texture(img);

    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let outer_f = f.clone();

    let window = web_sys::window().unwrap();
    if let Some(perf) = window.performance() {
        let mut prev_delta = perf.now();

        *outer_f.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let now = perf.now();
            let delta = now - prev_delta;
            prev_delta = now;

            game.update(delta);
            game.render_frame();

            window.request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
                .expect("Wtf even is this");
        }) as Box<dyn FnMut()>))
    }

    let window = web_sys::window().unwrap();
    window.request_animation_frame(outer_f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .expect("more wtf");

    Ok(())
}


struct LagomGame {
    renderer: Renderer,
    textures: Vec<Texture>,

    /// (Texture ID, x, y)
    draw_buffer: Vec<(u32, u32, u32)>,
    update_fn: UpdateFn,
}

impl LagomGame {
    pub fn new(f: UpdateFn) -> Self {
        let renderer = Renderer::new("canvas").unwrap();
        Self { renderer, textures: Vec::new(), draw_buffer: Vec::new(), update_fn: f }
    }

    fn read_input() {}

    fn update(&mut self, delta: f64) {
        (self.update_fn)(self, delta);
    }

    fn render_frame(&mut self) {
        self.renderer.clear();

        for req in &self.draw_buffer {
            self.renderer.draw_image(&self.textures[req.0 as usize], req.1, req.2);
        }

        self.draw_buffer.clear();
    }

    pub fn draw(&mut self, texture: u32, x: u32, y: u32) {
        self.draw_buffer.push((texture, x, y));
    }

    pub fn load_texture(&mut self, source: RgbaImage) -> u32 {
        let tex = self.renderer.load_texture(source);
        self.textures.push(tex);
        return (self.textures.len() - 1) as u32;
    }
}

