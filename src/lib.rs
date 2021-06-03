use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use crate::renderer::{Renderer, Texture};

mod renderer;

const IMG: &[u8] = include_bytes!("../assets/bg_tileable.png");

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let renderer = Renderer::new("canvas").unwrap();

    let img = image::load_from_memory(IMG).unwrap();
    let img = img.to_rgba8();
    let tx: Texture = renderer.load_texture(img);

    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let outer_f = f.clone();

    let window = web_sys::window().unwrap();
    if let Some(perf) = window.performance() {
        let _start = perf.now();

        *outer_f.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            renderer.clear();
            renderer.draw_image(&tx, 0, 0);
            renderer.draw_image(&tx, 256, 256);

            window.request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
                .expect("Wtf even is this");
        }) as Box<dyn FnMut()>))
    }

    let window = web_sys::window().unwrap();
    window.request_animation_frame(outer_f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .expect("more wtf");

    Ok(())
}


