use std::cell::RefCell;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

use cgmath::{Matrix4, vec3, Vector3};
use cgmath::conv::array4;
use cgmath::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlImageElement, WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlTexture, WebGlUniformLocation, WebGlVertexArrayObject};
use web_sys::console;

use crate::renderer::Renderer;

mod renderer;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let renderer = Renderer::new("canvas").unwrap();

    let tx: Rc<WebGlTexture> = renderer.load_texture("assets/bg_tileable.png")?;

    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let outer_f = f.clone();

    let window = web_sys::window().unwrap();
    if let Some(perf) = window.performance() {
        let start = perf.now();
        // let gl = Rc::new(gl.clone());
        // let program = Rc::new(program.clone());
        // let tex_location = Rc::new(tex_location);
        // let vertex_array = Rc::new(vertex_array);
        *outer_f.borrow_mut() = Some(Closure::wrap(Box::new(move || {

            // the image draw calls need to go in the renderer or a frame buffer or something
            renderer.draw_frame();
            renderer.draw_image(&*tx, 256, 256, 0, 0);
            renderer.draw_image(&*tx, 256, 256, 256, 256);

            window.request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
                .expect("Wtf even is this");
        }) as Box<dyn FnMut()>))
    }

    let window = web_sys::window().unwrap();
    window.request_animation_frame(outer_f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .expect("more wtf");


    Ok(())
}


