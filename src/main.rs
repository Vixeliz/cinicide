use easy_gltf::model::Mode;
use ggez::graphics::{
    Aabb, Camera3dBundle, Canvas3d, DrawParam, DrawParam3d, ImageFormat, Mesh3d, Mesh3dBuilder,
    Rect, Sampler, Transform3d, Vertex3d,
};
use ggez::graphics::{Image, Shader};
use ggez::input::keyboard::KeyCode;
use ggez::{
    event,
    glam::*,
    graphics::{self, Color},
    Context, GameResult,
};
use image::EncodableLayout;
use std::{env, path};

struct Model {
    center: Option<Vec3>,
    rotation: Quat,
    transform: Transform3d,
    meshes: Vec<Mesh3d>,
}
impl Model {
    pub fn to_aabb(&self) -> Option<Aabb> {
        let mut minimum = Vec3::MAX;
        let mut maximum = Vec3::MIN;
        for mesh in self.meshes.iter() {
            for p in mesh.vertices.iter() {
                minimum = minimum.min(Vec3::from_array(p.pos));
                maximum = maximum.max(Vec3::from_array(p.pos));
            }
            if minimum.x != std::f32::MAX
                && minimum.y != std::f32::MAX
                && minimum.z != std::f32::MAX
                && maximum.x != std::f32::MIN
                && maximum.y != std::f32::MIN
                && maximum.z != std::f32::MIN
            {
                return Some(Aabb::from_min_max(minimum, maximum));
            } else {
                return None;
            }
        }
        None
    }
}

struct MainState {
    camera: Camera3dBundle,
    models: Vec<Model>,
    psx: bool,
    psx_shader: Shader,
    custom_shader: Shader,
    rotation: f32,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut camera = Camera3dBundle::default();
        camera.camera.yaw = 0.0;
        camera.camera.pitch = 0.0;
        camera.projection.zfar = 1000.0;
        let mut meshes = Vec::default();
        let path = ctx.fs.resources_dir().join("tree_gun.glb");
        println!("{:?}", path);
        let scenes = easy_gltf::load(path).expect("Failed to load glTF");

        for scene in scenes {
            for model in scene.models {
                if let Some(base_color_texture) = model.material().pbr.base_color_texture.clone() {
                    let image = Image::from_pixels(
                        ctx,
                        base_color_texture.as_bytes(),
                        ImageFormat::Rgba8UnormSrgb,
                        base_color_texture.width(),
                        base_color_texture.height(),
                    );
                    let vertices = model
                        .vertices()
                        .iter()
                        .map(|x| {
                            let pos = Vec3::new(x.position.x, x.position.y, x.position.z);
                            let uv = Vec2::new(x.tex_coords.x, x.tex_coords.y);
                            Vertex3d::new(pos, uv, Color::new(1.0, 1.0, 1.0, 0.0))
                        })
                        .collect();
                    let indices = model.indices();
                    if let Some(indices) = indices.as_ref() {
                        let indices = indices.iter().map(|x| *x as u32).collect();
                        let mesh = Mesh3dBuilder::new()
                            .from_data(vertices, indices, Some(image))
                            .build(ctx);
                        meshes.push(mesh);
                    }
                }
            }
        }

        let mut cincide_model = Model {
            center: None,
            rotation: Quat::IDENTITY,
            transform: Transform3d::default(),
            meshes,
        };

        cincide_model.center = Some(cincide_model.to_aabb().unwrap().center.into());

        Ok(MainState {
            models: vec![cincide_model],
            camera,
            custom_shader: graphics::ShaderBuilder::from_path("/fancy.wgsl")
                .build(&ctx.gfx)
                .unwrap(),
            psx_shader: graphics::ShaderBuilder::from_path("/psx.wgsl")
                .build(&ctx.gfx)
                .unwrap(),
            psx: true,
            rotation: 0.0,
        })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // set_cursor_hidden(ctx, true);
        // set_cursor_grabbed(ctx, true)?;
        let k_ctx = &ctx.keyboard.clone();
        let (yaw_sin, yaw_cos) = self.camera.camera.yaw.sin_cos();
        let dt = ctx.time.delta().as_secs_f32();
        let forward = Vec3::new(yaw_cos, 0.0, yaw_sin).normalize() * 15.0 * dt;
        let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize() * 15.0 * dt;
        self.rotation += 2.5 * dt;
        // if k_ctx.is_key_pressed(KeyCode::Q) {
        //     self.meshes[0].1 += 1.0 * dt;
        // }
        // if k_ctx.is_key_pressed(KeyCode::E) {
        //     self.meshes[0].1 -= 1.0 * dt;
        // }
        if k_ctx.is_key_pressed(KeyCode::Space) {
            self.camera.camera.position.y += 10.0 * dt;
        }
        if k_ctx.is_key_pressed(KeyCode::C) {
            self.camera.camera.position.y -= 10.0 * dt;
        }
        if k_ctx.is_key_pressed(KeyCode::W) {
            self.camera.camera.position += forward;
        }
        if k_ctx.is_key_just_pressed(KeyCode::K) {
            self.psx = !self.psx;
        }
        if k_ctx.is_key_pressed(KeyCode::S) {
            self.camera.camera.position -= forward;
        }
        if k_ctx.is_key_pressed(KeyCode::D) {
            self.camera.camera.position += right;
        }
        if k_ctx.is_key_pressed(KeyCode::A) {
            self.camera.camera.position -= right;
        }
        if k_ctx.is_key_pressed(KeyCode::Right) {
            self.camera.camera.yaw += 1.0_f32.to_radians() * dt * 50.0;
        }
        if k_ctx.is_key_pressed(KeyCode::Left) {
            self.camera.camera.yaw -= 1.0_f32.to_radians() * dt * 50.0;
        }
        if k_ctx.is_key_pressed(KeyCode::Up) {
            self.camera.camera.pitch += 1.0_f32.to_radians() * dt * 50.0;
        }
        if k_ctx.is_key_pressed(KeyCode::Down) {
            self.camera.camera.pitch -= 1.0_f32.to_radians() * dt * 50.0;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let canvas_image = Image::new_canvas_image(ctx, ImageFormat::Bgra8UnormSrgb, 320, 240, 1);
        let mut canvas3d =
            Canvas3d::from_image(ctx, &mut self.camera, canvas_image.clone(), Color::BLACK);
        canvas3d.set_sampler(Sampler::nearest_clamp());
        if self.psx {
            canvas3d.set_shader(self.psx_shader.clone());
        } else {
            canvas3d.set_shader(self.custom_shader.clone());
        }
        for model in self.models.iter() {
            for mesh in model.meshes.iter() {
                canvas3d.draw(
                    ctx,
                    mesh.clone(),
                    DrawParam3d::default()
                        .pivot(model.center.unwrap() + Vec3::from(model.transform.position))
                        .transform(model.transform)
                        .rotation(Quat::from_euler(EulerRot::YZX, self.rotation, 0.0, 0.0)),
                );
            }
        }
        canvas3d.finish(ctx)?;
        let mut canvas = graphics::Canvas::from_frame(ctx, None);

        // Do ggez drawing
        canvas.set_sampler(Sampler::nearest_clamp());
        let params = DrawParam::new().dest(Vec2::new(0.0, 0.0)).scale(Vec2::new(
            ctx.gfx.drawable_size().0 / 320.0,
            ctx.gfx.drawable_size().1 / 240.0,
        ));
        canvas.draw(&canvas_image, params);
        let dest_point1 = Vec2::new(10.0, 210.0);
        let dest_point2 = Vec2::new(10.0, 250.0);
        canvas.draw(
            &graphics::Text::new("You can mix 3d and 2d drawing;"),
            dest_point1,
        );
        canvas.draw(
            &graphics::Text::new(
                "
                WASD: Move
                Arrow Keys: Look
                K: Toggle default shader and custom shader
                C/Space: Up and Down
                ",
            ),
            dest_point2,
        );

        canvas.finish(ctx)?;

        Ok(())
    }
}

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("cube", "ggez")
        .window_mode(ggez::conf::WindowMode::default().resizable(true))
        .add_resource_path(resource_dir);

    let (mut ctx, events_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, events_loop, state)
}
