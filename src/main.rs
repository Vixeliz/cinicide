use ggez::conf::NumSamples;
use ggez::graphics::Model;
use ggez::graphics::{
    Camera3d, Canvas3d, DrawParam, DrawParam3d, ImageFormat, Sampler, Transform3d,
};
use ggez::graphics::{Image, Shader};
use ggez::input::keyboard::KeyCode;
use ggez::{
    event,
    glam::*,
    graphics::{self, Color},
    Context, GameResult,
};
use std::{env, path};

pub struct ModelPos {
    transform: Transform3d,
    model: Model,
}
impl ModelPos {
    fn new(model: Model, transform: Transform3d) -> Self {
        Self { model, transform }
    }
}

impl ggez::graphics::Drawable3d for ModelPos {
    fn draw(&self, canvas: &mut Canvas3d, _: impl Into<DrawParam3d>) {
        canvas.draw(
            &self.model,
            // DrawParam3d::default()
            DrawParam3d::default()
                .transform(self.transform)
                .offset(self.model.to_aabb().unwrap_or_default().center),
        )
    }
}

struct MainState {
    camera: Camera3d,
    models: Vec<ModelPos>,
    no_view_models: Vec<ModelPos>,
    psx: bool,
    psx_shader: Shader,
    custom_shader: Shader,
    skybox: ModelPos,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut camera = Camera3d::default();
        camera.transform.yaw = 0.0;
        camera.transform.pitch = 0.0;
        camera.projection.zfar = 1000.0;
        let rot = Quat::from_euler(
            EulerRot::YZX,
            -80.0_f32.to_radians(),
            0.0,
            10.0_f32.to_radians(),
        );
        let tree_gun = ModelPos::new(
            Model::from_path(ctx, "/tree_gun.glb", None)?,
            Transform3d::Values {
                pos: Vec3::new(4.75, -1.25, 2.0).into(),
                rotation: rot.into(),
                scale: Vec3::splat(3.0).into(),
                offset: None,
                pivot: None,
            },
        );
        let cin_gun = ModelPos::new(
            Model::from_path(ctx, "/cinicide_gun.glb", None)?,
            Transform3d::Values {
                pos: Vec3::new(10.0, 5.0, -10.0).into(),
                rotation: Quat::IDENTITY.into(),
                scale: Vec3::splat(10.0).into(),
                offset: None,
                pivot: None,
            },
        );
        let skybox = ModelPos::new(
            Model::from_path(ctx, "/skybox.gltf", None)?,
            Transform3d::Values {
                pos: Vec3::ZERO.into(),
                rotation: Quat::IDENTITY.into(),
                scale: Vec3::splat(100.0).into(),
                offset: None,
                pivot: None,
            },
        );
        ggez::input::mouse::set_cursor_hidden(ctx, true);
        ggez::input::mouse::set_cursor_grabbed(ctx, true)?;

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
        // set_cursor_grabbed(ctx, true)?;
        let k_ctx = &ctx.keyboard.clone();
        let (yaw_sin, yaw_cos) = self.camera.transform.yaw.sin_cos();
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
            self.camera.transform.position.y += 10.0 * dt;
        }
        if k_ctx.is_key_pressed(KeyCode::C) {
            self.camera.transform.position.y -= 10.0 * dt;
        }
        if k_ctx.is_key_pressed(KeyCode::W) {
            self.camera.transform = self.camera.transform.translate(forward);
        }
        if k_ctx.is_key_just_pressed(KeyCode::K) {
            self.psx = !self.psx;
        }
        if k_ctx.is_key_pressed(KeyCode::S) {
            self.camera.transform = self.camera.transform.translate(-forward);
        }
        if k_ctx.is_key_pressed(KeyCode::D) {
            self.camera.transform = self.camera.transform.translate(right);
        }
        if k_ctx.is_key_pressed(KeyCode::A) {
            self.camera.transform = self.camera.transform.translate(-right);
        }
        if k_ctx.is_key_pressed(KeyCode::Right) {
            self.camera.transform.yaw += 1.0_f32.to_radians() * dt * 50.0;
        }
        if k_ctx.is_key_pressed(KeyCode::Left) {
            self.camera.transform.yaw -= 1.0_f32.to_radians() * dt * 50.0;
        }
        if k_ctx.is_key_pressed(KeyCode::Up) {
            self.camera.transform.pitch += 1.0_f32.to_radians() * dt * 50.0;
        }
        if k_ctx.is_key_pressed(KeyCode::Down) {
            self.camera.transform.pitch -= 1.0_f32.to_radians() * dt * 50.0;
        }
        let mouse_delta = ctx.mouse.raw_delta();
        let speed = 0.5;
        let mouse_delta_y = mouse_delta.y as f32 * speed * dt * -1.0;
        let mouse_delta_x = mouse_delta.x as f32 * speed * dt;
        self.camera.transform.yaw += mouse_delta_x;
        self.camera.transform.pitch += mouse_delta_y;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let sky_image = Image::new_canvas_image(ctx, ImageFormat::Bgra8UnormSrgb, 320, 240, 1);
        let mut sky_cam = self.camera.clone();
        sky_cam.transform = sky_cam.transform.position(Vec3::ZERO);
        let mut canvas3d = Canvas3d::from_image(ctx, sky_image.clone(), Color::BLACK);
        canvas3d.set_projection(sky_cam.to_matrix());
        canvas3d.set_sampler(Sampler::nearest_clamp());
        canvas3d.set_shader(&self.custom_shader);
        canvas3d.draw(&self.skybox, DrawParam3d::default());
        canvas3d.finish(ctx)?;
        let canvas_image = Image::new_canvas_image(ctx, ImageFormat::Bgra8UnormSrgb, 320, 240, 1);
        let mut canvas3d =
            Canvas3d::from_image(ctx, canvas_image.clone(), Color::new(0.0, 0.0, 0.0, 0.0));
        canvas3d.set_shader(&self.custom_shader);
        canvas3d.set_projection(self.camera.to_matrix());
        canvas3d.set_sampler(Sampler::nearest_clamp());
        if self.psx {
            canvas3d.set_shader(&self.psx_shader);
        } else {
            canvas3d.set_shader(&self.custom_shader);
        }
        for model in self.no_view_models.iter() {
            canvas3d.draw(
                model,
                DrawParam3d::default().pivot(model.model.to_aabb().unwrap().center),
            );
        }
        for model in self.models.iter() {
            canvas3d.draw(model, DrawParam3d::default());
        }
        canvas3d.finish(ctx)?;
        let mut camera = Camera3d::default();
        camera.projection.znear = 0.002;
        camera.projection.fovy = 70.0_f32.to_radians();
        let canvas_image_two =
            Image::new_canvas_image(ctx, ImageFormat::Bgra8UnormSrgb, 320, 240, 1);
        let mut canvas3d = Canvas3d::from_image(
            ctx,
            canvas_image_two.clone(),
            Color::new(0.0, 0.0, 0.0, 0.0),
        );
        canvas3d.set_projection(camera.to_matrix());
        canvas3d.set_sampler(Sampler::nearest_clamp());
        if self.psx {
            canvas3d.set_shader(&self.psx_shader);
        } else {
            canvas3d.set_shader(&self.custom_shader);
        }
        for model in self.no_view_models.iter() {
            canvas3d.draw(model, DrawParam3d::default());
        }
        canvas3d.finish(ctx)?;
        let mut canvas = graphics::Canvas::from_frame(ctx, None);

        // Do ggez drawing
        canvas.set_sampler(Sampler::nearest_clamp());
        let params = DrawParam::default()
            .dest(Vec2::new(0.0, 0.0))
            .scale(Vec2::new(
                ctx.gfx.drawable_size().0 / 320.0,
                ctx.gfx.drawable_size().1 / 240.0,
            ));
        canvas.draw(&sky_image, params);
        canvas.draw(&canvas_image, params);
        canvas.draw(&canvas_image_two, params);
        let dest_point1 = Vec2::new(10.0, 210.0);
        let dest_center = Vec2::new(
            ctx.gfx.drawable_size().0 / 2.0,
            ctx.gfx.drawable_size().1 / 2.0,
        );
        let dest_point2 = Vec2::new(10.0, 250.0);
        let crosshair = Image::from_path(ctx, "/crosshair.png")?;
        canvas.draw(
            &crosshair,
            DrawParam::default()
                // .offset(Vec2::new(
                //     crosshair.width() as f32 / 2.0,
                //     crosshair.height() as f32 / 2.0,
                // ))
                .dest(dest_center), // .scale(Vec2::splat(10.0)),
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

        canvas.draw(
            &graphics::Text::new(format!("{}", ctx.time.fps())),
            dest_point1,
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

    let cb = ggez::ContextBuilder::new("cinicide", "vixeliz")
        .window_mode(ggez::conf::WindowMode::default().resizable(true))
        .window_setup(ggez::conf::WindowSetup {
            title: "Cinicide".to_owned(),
            samples: NumSamples::One,
            vsync: false,
            icon: "".to_owned(),
            srgb: true,
        })
        .add_resource_path(resource_dir);

    let (mut ctx, events_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, events_loop, state)
}
