use std::rc::Rc;

use cgmath::Matrix4;
use cgmath::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{console, HtmlImageElement, WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlTexture, WebGlUniformLocation, WebGlVertexArrayObject};

pub struct Renderer {
    gl: WebGl2RenderingContext,
    program: WebGlProgram,
    tex_location: WebGlUniformLocation,
    vertex_array: WebGlVertexArrayObject,
    matrix_location: WebGlUniformLocation,
}


fn compile_shader(
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

fn link_program(
    context: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program().ok_or_else(|| String::from("Unable to create shader object"))?;

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


impl Renderer {
    pub fn new(canvas_id: &str) -> Result<Self, JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id(canvas_id).unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
        let gl: WebGl2RenderingContext = canvas
            .get_context("webgl2")?
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()?;

        let vert_shader = compile_shader(
            &gl,
            WebGl2RenderingContext::VERTEX_SHADER,
            r#"#version 300 es

        in vec4 a_position;
        in vec2 a_texcoord;

        uniform mat4 u_matrix;

        out vec2 v_texcoord;

        void main() {
           gl_Position = u_matrix * a_position;
           v_texcoord = a_texcoord;
        }
        "#)?;

        let frag_shader = compile_shader(
            &gl,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            r#"#version 300 es

        precision highp float;

        in vec2 v_texcoord;

        uniform sampler2D u_texture;

        out vec4 outColor;

        void main() {
           outColor = texture(u_texture, v_texcoord);
        }
        "#)?;

        let program = link_program(&gl, &vert_shader, &frag_shader)?;
        gl.use_program(Some(&program));

        // look up position.
        let pos_attribute = gl.get_attrib_location(&program, "a_position") as u32;
        let tex_attribute = gl.get_attrib_location(&program, "a_texcoord") as u32;

        // Uniforms
        let matrix_location = gl.get_uniform_location(&program, "u_matrix").expect("no matrix");
        let tex_location = gl.get_uniform_location(&program, "u_texture").expect("no texture");

        let vertex_array = gl.create_vertex_array().expect("broken");
        gl.bind_vertex_array(Some(&vertex_array));

        let position_buffer = gl.create_buffer().expect("create buffer failed");
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&position_buffer));

        // Note that `Float32Array::view` is somewhat dangerous (hence the
        // `unsafe`!). This is creating a raw view into our module's
        // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
        // (aka do a memory allocation in Rust) it'll cause the buffer to change,
        // causing the `Float32Array` to be invalid.
        //
        // As a result, after `Float32Array::view` we have to be very careful not to
        // do any memory allocations before it's dropped.
        let positions: [f32; 12] = [0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let pos_array = unsafe { js_sys::Float32Array::view(&positions) };

        gl.buffer_data_with_array_buffer_view(WebGl2RenderingContext::ARRAY_BUFFER, &pos_array, WebGl2RenderingContext::STATIC_DRAW);
        gl.enable_vertex_attrib_array(pos_attribute);
        gl.vertex_attrib_pointer_with_i32(pos_attribute, 2, WebGl2RenderingContext::FLOAT, false, 0, 0);

        let tex_coord_buffer = gl.create_buffer().expect("create buffer failed.");
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&tex_coord_buffer));
        let tex_coords: [f32; 12] = [0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let tex_array = unsafe { js_sys::Float32Array::view(&tex_coords) };
        gl.buffer_data_with_array_buffer_view(WebGl2RenderingContext::ARRAY_BUFFER, &tex_array, WebGl2RenderingContext::STATIC_DRAW);
        gl.enable_vertex_attrib_array(tex_attribute);
        gl.vertex_attrib_pointer_with_i32(tex_attribute, 2, WebGl2RenderingContext::FLOAT, true, 0, 0);

        Ok(Self { gl, program, tex_location, vertex_array, matrix_location })
    }

    pub fn draw_frame(&self) {
        self.gl.viewport(0, 0, 512, 512);
        self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
        self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT);
        // draw_image(&*tx, 256, 256, 0, 0, &gl, &program, &tex_location, &vertex_array, &matrix_location);
        // draw_image(&*tx, 256, 256, 256, 256, &gl, &program, &tex_location, &vertex_array, &matrix_location);
    }

    pub fn draw_image(&self, texture: &WebGlTexture, width: usize, height: usize, x: usize, y: usize) {

        // Do I need these?
        self.gl.use_program(Some(&self.program));
        self.gl.bind_vertex_array(Some(&self.vertex_array));

        let texture_unit: i32 = 0;
        self.gl.uniform1i(Some(&self.tex_location), texture_unit);

        self.gl.active_texture(WebGl2RenderingContext::TEXTURE0 + texture_unit as u32);
        self.gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

        // TODO canvas size.
        let matrix: cgmath::Matrix4<f32> = cgmath::ortho(0_f32, 512_f32, 512_f32,
                                                         0_f32, -1_f32, 1_f32);

        let translation = Matrix4::from_translation(cgmath::vec3(x as f32, y as f32, 1.0));
        let scale = Matrix4::from_nonuniform_scale(width as f32, height as f32, 1.0);
        let matrix: Matrix4<f32> = matrix * translation * scale;

        // this clone is not ideal, but I don't know what else I can do.
        let arr = cgmath::conv::array4(matrix).iter().flatten().cloned().collect::<Vec<f32>>();

        self.gl.uniform_matrix4fv_with_f32_array(Some(&self.matrix_location), false, &arr);
        self.gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);
    }

    // TODO how can I write a test?
    pub fn read_pixels(&self) -> Vec<u8> {
        // TODO canvas size
        let mut dest = vec![0u8; 512 * 512 * 4];
        self.gl.read_pixels_with_opt_u8_array(0, 0, 512, 512, WebGl2RenderingContext::RGBA, WebGl2RenderingContext::UNSIGNED_BYTE, Some(&mut dest));
        return dest;
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/Tutorial/Using_textures_in_WebGL
    pub fn load_texture(&self, source: &str) -> Result<Rc<WebGlTexture>, JsValue> {
        let texture: WebGlTexture = self.gl.create_texture().unwrap();
        self.gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

        // Placeholder pixel
        let pixel: [u8; 4] = [0, 0, 255, 255];

        let rgba = WebGl2RenderingContext::RGBA;

        self.gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D, 0, rgba as i32,
            1, 1, 0, rgba, WebGl2RenderingContext::UNSIGNED_BYTE, Some(&pixel),
        );

        self.gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_WRAP_S,
                               WebGl2RenderingContext::CLAMP_TO_EDGE as i32);
        self.gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_WRAP_T,
                               WebGl2RenderingContext::CLAMP_TO_EDGE as i32);


        // Load the image
        let img = HtmlImageElement::new().unwrap();
        img.set_cross_origin(Some(""));

        let imgrc = Rc::new(img);
        let texture = Rc::new(texture);

        {
            let img: Rc<HtmlImageElement> = imgrc.clone();
            let texture = texture.clone();
            let gl = Rc::new(self.gl.clone());
            let a = Closure::wrap(Box::new(move || {
                gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
                gl.tex_image_2d_with_u32_and_u32_and_html_image_element(WebGl2RenderingContext::TEXTURE_2D, 0,
                                                                        rgba as i32, rgba, WebGl2RenderingContext::UNSIGNED_BYTE, &img);
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
        console::log_1(&"setting url".into());
        imgrc.set_src(source);

        Ok(texture)
    }
}