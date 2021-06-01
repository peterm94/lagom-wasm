use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlImageElement, WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlTexture};
use web_sys::console;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let gl: WebGl2RenderingContext = canvas
        .get_context("webgl2")?
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>()?;

    let vert_shader = compile_shader(
        &gl,
        WebGl2RenderingContext::VERTEX_SHADER,
        r#"
        attribute vec4 a_position;
        attribute vec2 a_texcoord;

        uniform mat4 u_matrix;

        varying vec2 v_texcoord;

        void main() {
           gl_Position = u_matrix * a_position;
           v_texcoord = a_texcoord;
        }
        "#)?;

    let frag_shader = compile_shader(
        &gl,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        r#"
        precision mediump float;

        varying vec2 v_texcoord;

        uniform sampler2D u_texture;

        void main() {
           gl_FragColor = texture2D(u_texture, v_texcoord);
        }
        "#)?;

    let program = link_program(&gl, &vert_shader, &frag_shader)?;
    gl.use_program(Some(&program));

    let vertices: [f32; 9] = [-0.7, -0.7, 0.0, 0.7, -0.7, 0.0, 0.0, 0.7, 0.0];

    let tx = load_texture(&gl)?;


    // Or we could do create_buffer()?
    let buffer = gl.create_framebuffer().ok_or("failed to create buffer")?;
    gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&buffer));

    gl.framebuffer_texture_2d(WebGl2RenderingContext::FRAMEBUFFER,
                              WebGl2RenderingContext::COLOR_ATTACHMENT0,
                              WebGl2RenderingContext::TEXTURE_2D,
                              Some(&tx),
                              0);

    console::log_1(&"what now".into());

    // Note that `Float32Array::view` is somewhat dangerous (hence the
    // `unsafe`!). This is creating a raw view into our module's
    // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
    // (aka do a memory allocation in Rust) it'll cause the buffer to change,
    // causing the `Float32Array` to be invalid.
    //
    // As a result, after `Float32Array::view` we have to be very careful not to
    // do any memory allocations before it's dropped.
    unsafe {
        let vert_array = js_sys::Float32Array::view(&vertices);

        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &vert_array,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }

    gl.vertex_attrib_pointer_with_i32(0, 3, WebGl2RenderingContext::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(0);
    gl.clear_color(0.0, 0.40, 0.42, 1.0);
    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
    //
    //
    // // This doesn't do anything
    gl.draw_arrays(
        WebGl2RenderingContext::TRIANGLES,
        0,
        (vertices.len() / 3) as i32,
    );
    Ok(())
}

// https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/Tutorial/Using_textures_in_WebGL
pub fn load_texture(context: &WebGl2RenderingContext) -> Result<Rc<WebGlTexture>, JsValue> {
    let texture: WebGlTexture = context.create_texture().unwrap();
    context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

    let level = 0;
    let internal_format = WebGl2RenderingContext::RGBA;
    let width = 1;
    let height = 1;
    let border = 0;
    let src_format = WebGl2RenderingContext::RGBA;
    let src_type = WebGl2RenderingContext::UNSIGNED_BYTE;

    // Placeholder pixel
    let pixel: [u8; 4] = [0, 0, 255, 255];

    context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
        WebGl2RenderingContext::TEXTURE_2D, level, internal_format as i32,
        width, height, border, src_format, src_type, Some(&pixel),
    );

    // Load the image
    let img = HtmlImageElement::new().unwrap();
    img.set_cross_origin(Some(""));

    let imgrc = Rc::new(img);

    let texture = Rc::new(texture);

    {
        let img = imgrc.clone();
        let texture = texture.clone();
        let gl = Rc::new(context.clone());
        let a = Closure::wrap(Box::new(move || {
            gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

            if let Err(e) = gl.tex_image_2d_with_u32_and_u32_and_html_image_element(
                WebGl2RenderingContext::TEXTURE_2D,
                level,
                internal_format as i32,
                src_format,
                src_type,
                &img,
            ) {
                console::log_1(&e);
                return;
            }

            gl.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);
        }) as Box<dyn FnMut()>);
        imgrc.set_onload(Some(a.as_ref().unchecked_ref()));

        // Normally we'd store the handle to later get dropped at an appropriate
        // time but for now we want it to be a global handler so we use the
        // forget method to drop it without invalidating the closure. Note that
        // this is leaking memory in Rust, so this should be done judiciously!
        // TODO fix this using something from the docs here:
        //  https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen/closure/struct.Closure.html
        a.forget();
    }

    imgrc.set_src("assets/bg_tileable.png");

    Ok(texture)
    // context.bind_texture(WebGl2RenderingContext::ARRAY_BUFFER, Some())
}

// pub fn draw_image(context: &mut WebGl2RenderingContext, texture: &WebGlTexture) {
// context.bind_texture(WebGl2RenderingContext::ARRAY_BUFFER, Some(&texture));
// context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, position_buffer);
// context.enable_vertex_attrib_array(position_location);
// context.vertex_attrib_pointer_with_f64(position_location, 2, WebGl2RenderingContext::FLOAT,
//                                        false, 0, 0);
// context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, tex_coord_buffer);
// context.enable_vertex_attrib_array(tex_coord_location);
// context.vertex_attrib_pointer_with_f64(tex_coord_location, 2, WebGl2RenderingContext::FLOAT,
//                                        false, 0, 0);

// let matrix
// }

pub fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    context: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}
