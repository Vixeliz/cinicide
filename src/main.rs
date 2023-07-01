use ggez::graphics::{
    Aabb, Camera3dBundle, Canvas3d, DrawParam, DrawParam3d, ImageFormat, Mesh3d, Mesh3dBuilder,
    Sampler, Transform3d, Vertex3d,
};
use ggez::graphics::{Drawable3d, Model};
use ggez::graphics::{Image, Shader};
use ggez::input::keyboard::KeyCode;
use ggez::{
    event,
    glam::*,
    graphics::{self, Color},
    Context, GameResult,
};
use image::EncodableLayout;
use std::path::PathBuf;
use std::{env, path};

pub struct ModelPos {
    center: Vec3,
    transform: Transform3d,
    model: Model,
}
impl ModelPos {
    fn new(model: Model, transform: Transform3d) -> Self {
        Self {
            center: model.to_aabb().unwrap_or_default().center.into(),
            model,
            transform,
        }
    }
}

impl Drawable3d for ModelPos {
    fn draw(
        &self,
        gfx: &mut impl ggez::context::HasMut<graphics::GraphicsContext>,
        canvas: &mut Canvas3d,
        param: impl Into<DrawParam3d>,
    ) {
        canvas.draw(
            gfx,
            &self.model,
            DrawParam3d::default()
                .transform(self.transform)
                .offset(self.center),
        )
    }
}

struct MainState {
    camera: Camera3dBundle,
    models: Vec<ModelPos>,
    no_view_models: Vec<ModelPos>,
    psx: bool,
    psx_shader: Shader,
    custom_shader: Shader,
    skybox: ModelPos,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut camera = Camera3dBundle::default();
        camera.camera.yaw = 0.0;
        camera.camera.pitch = 0.0;
        camera.projection.zfar = 1000.0;
        let rot = Quat::from_euler(EulerRot::YZX, 0.0_f32.to_radians(), 0.0, 0.0);
        let tree_gun = ModelPos::new(
            Model::from_path(ctx, "/tree_gun.glb", None)?,
            Transform3d {
                position: Vec3::new(3.0, -1.5, 0.9).into(),
                rotation: rot.into(),
                scale: Vec3::splat(3.0).into(),
            },
        );
        let cin_gun = ModelPos::new(
            Model::from_path(
                ctx,
                "/skybox.obj",
                Image::from_color(ctx, 1, 1, Some(Color::RED)),
            )?,
            Transform3d {
                position: Vec3::new(10.0, 5.0, -10.0).into(),
                rotation: Quat::IDENTITY.into(),
                scale: Vec3::splat(-10.0).into(),
            },
        );
        let skybox = ModelPos::new(
            Model::from_path(ctx, "/skybox.gltf", None)?,
            Transform3d {
                position: Vec3::ZERO.into(),
                rotation: Quat::IDENTITY.into(),
                scale: Vec3::splat(100.0).into(),
            },
        );

        Ok(MainState {
            models: vec![cin_gun],
            no_view_models: vec![tree_gun],
            skybox,
            camera,
            custom_shader: graphics::ShaderBuilder::from_path("/fancy.wgsl")
                .build(&ctx.gfx)
                .unwrap(),
            psx_shader: graphics::ShaderBuilder::from_path("/psx.wgsl")
                .build(&ctx.gfx)
                .unwrap(),
            psx: true,
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
        let sky_image = Image::new_canvas_image(ctx, ImageFormat::Bgra8UnormSrgb, 320, 240, 1);
        let mut sky_cam = self.camera;
        sky_cam.camera.position = Vec3::ZERO;
        let mut canvas3d = Canvas3d::from_image(ctx, &mut sky_cam, sky_image.clone(), Color::BLACK);
        canvas3d.set_sampler(Sampler::nearest_clamp());
        canvas3d.set_shader(self.custom_shader.clone());
        canvas3d.draw(ctx, &self.skybox, DrawParam3d::default());
        canvas3d.finish(ctx)?;
        let canvas_image = Image::new_canvas_image(ctx, ImageFormat::Bgra8UnormSrgb, 320, 240, 1);
        let mut canvas3d = Canvas3d::from_image(
            ctx,
            &mut self.camera,
            canvas_image.clone(),
            Color::new(0.0, 0.0, 0.0, 0.0),
        );
        canvas3d.set_sampler(Sampler::nearest_clamp());
        if self.psx {
            canvas3d.set_shader(self.psx_shader.clone());
        } else {
            canvas3d.set_shader(self.custom_shader.clone());
        }
        for model in self.models.iter() {
            canvas3d.draw(ctx, model, DrawParam3d::default());
        }
        canvas3d.finish(ctx)?;
        let mut camera = Camera3dBundle::default();
        camera.projection.znear = 0.002;
        camera.projection.fovy = 70.0_f32.to_radians();
        let canvas_image_two =
            Image::new_canvas_image(ctx, ImageFormat::Bgra8UnormSrgb, 320, 240, 1);
        let mut canvas3d = Canvas3d::from_image(
            ctx,
            &mut camera,
            canvas_image_two.clone(),
            Color::new(0.0, 0.0, 0.0, 0.0),
        );
        canvas3d.set_sampler(Sampler::nearest_clamp());
        if self.psx {
            canvas3d.set_shader(self.psx_shader.clone());
        } else {
            canvas3d.set_shader(self.custom_shader.clone());
        }
        for model in self.no_view_models.iter() {
            canvas3d.draw(ctx, model, DrawParam3d::default());
        }
        canvas3d.finish(ctx)?;
        let mut canvas = graphics::Canvas::from_frame(ctx, None);

        // Do ggez drawing
        canvas.set_sampler(Sampler::nearest_clamp());
        let params = DrawParam::new().dest(Vec2::new(0.0, 0.0)).scale(Vec2::new(
            ctx.gfx.drawable_size().0 / 320.0,
            ctx.gfx.drawable_size().1 / 240.0,
        ));
        canvas.draw(&sky_image, params);
        canvas.draw(&canvas_image, params);
        canvas.draw(&canvas_image_two, params);
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
