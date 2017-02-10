#[macro_use]
extern crate glium;
pub extern crate nalgebra;
use nalgebra::Transformation;
use nalgebra::ToHomogeneous;
use nalgebra::Norm;
extern crate image;

use nalgebra as na;

pub mod boxtree;
pub mod boxmodel;
pub mod camera;

use std::time::{Duration, Instant, SystemTime};

fn main() {
    let mut max_dist: f32 = 100.0;

    struct Compression;
    impl boxtree::Compression for Compression {
        fn compress(&mut self, pos: na::Vector3<u32>, depth: u8, chunk: &boxtree::Chunk<boxtree::HiddenLeaf>) -> Option<boxtree::HiddenLeaf> {
            panic!("not jet implemented");
        }
        fn decompress(&mut self, pos: na::Vector3<u32>, depth: u8, leaf: boxtree::HiddenLeaf, chunk: &mut boxtree::Chunk<boxtree::HiddenLeaf>) {
            panic!("not jet implemented");
        }
    }
    let mut box_tree = boxtree::Tree::new(7, 1 << boxtree::NODE_INDEX_BITS, Compression);

    use std::io::Cursor;
    let image = image::load(Cursor::new(&include_bytes!("test.png")[..]), image::PNG).unwrap().to_rgba();
    let image_dimensions = image.dimensions();
    for x in 0..image_dimensions.0 {
        for y in 0..image_dimensions.1 {
            let pixel = image.get_pixel(x, y);
            let grey = (pixel.data[0] as u16) + (pixel.data[1] as u16) + (pixel.data[2] as u16);
            let height = grey / 32;
            for h in 0..(height+1) {
                if !box_tree.set_at_pos(na::Vector3::new(x, h as u32, y), boxtree::Leaf::from_solid_box_spec(true, grey / 3)) {
                    panic!("Cannot add box, not enaugh chunks available.");
                }
            }
        }
    }

    let mut fly_cam = camera::FlyCam::new();
    fly_cam.translate(na::Vector3::new(0.0, 0.0, 10.0));
    let mut fly_cam_controller = camera::FlyCamController::new();

    use glium::{DisplayBuild, Surface};
    let display = glium::glutin::WindowBuilder::new().with_depth_buffer(24).build_glium().unwrap();

    let box_model = boxmodel::Model::new(&display);
    let tiles = boxmodel::Tiles::new(
        boxmodel::box_type_face_tile_map_tex_from_array(&display, &[
            0, 0, 0, 0, 0, 0
        ]),
        boxmodel::tile_color_tex_array_from_images(&display, &[
            image::load(Cursor::new(&include_bytes!("boxes2.png")[..]), image::PNG).unwrap().to_rgba()
        ])
    );

    let persp_mat: nalgebra::PerspectiveMatrix3<f32> = nalgebra::PerspectiveMatrix3::new(
        {
            let d = display.get_framebuffer_dimensions();
            d.0 as f32 / d.1 as f32
        },
        (std::f64::consts::PI as f32) * 0.5,
        0.01,
        2.0 * max_dist,
    );
    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            .. Default::default()
        },
        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullCounterClockwise,
        .. Default::default()
    };

    let mut time = Instant::now();
    let mut elapsed = Duration::new(0, 0);
    let mut frames: usize = 0;
    loop {
        let mut per_instance = {
            let mut data: Vec<boxmodel::Instance> = Vec::new();
            let isometry = fly_cam.isometry64();
            let origin = isometry * na::Point3::new(0.0f64, 0.0, 0.0);
            let d = display.get_framebuffer_dimensions();
            let d = (d.0 as f64) / (d.1 as f64);
            let mut planes: [na::Vector3<f64>; 4] = [
                na::Vector3::new( 1.0/d,  0.0, -1.0),
                na::Vector3::new(-1.0/d,  0.0, -1.0),
                na::Vector3::new( 0.0  ,  1.0, -1.0),
                na::Vector3::new( 0.0  , -1.0, -1.0)
            ];
            for mut p in planes.iter_mut() {
                *p = isometry * p.normalize();
            }
            box_tree.cast_view(origin, planes, max_dist as f64, &mut |box_pos: na::Vector3<u32>, leaf| {
                data.push(
                    boxmodel::Instance {
                        box_pos: [box_pos.x, box_pos.y, box_pos.z],
                        box_type: 0/*leaf.box_spec() as u32*/,
                    }
                );
            });
            glium::vertex::VertexBuffer::new(&display, &data).unwrap()
        };

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);
        box_model.draw(
            &mut target,
            (*persp_mat.as_matrix() * fly_cam.isometry().inverse_transformation().to_homogeneous()).as_ref(),
            &per_instance,
            &tiles,
            &params
        );
        target.finish().unwrap();

        for ev in display.poll_events() {
            fly_cam_controller.process_event(&ev, &mut fly_cam);
            match ev {
                glium::glutin::Event::Closed => return,
                glium::glutin::Event::KeyboardInput(glium::glutin::ElementState::Pressed, _, Some(glium::glutin::VirtualKeyCode::I)) => {
                    max_dist *= 0.5;
                },
                glium::glutin::Event::KeyboardInput(glium::glutin::ElementState::Pressed, _, Some(glium::glutin::VirtualKeyCode::O)) => {
                    max_dist *= 2.0;
                },
                _ => {},
            }
        }
        fly_cam_controller.update(0.01, &mut fly_cam);

        frames += 1;
        let mut e = time.elapsed() - elapsed;
        while e.as_secs() >= 1 {
            println!("MD: {} FPS: {}", max_dist as u32, frames);
            frames = 0;
            e -= Duration::new(1, 0);
            elapsed += Duration::new(1, 0);
        }
    }
}
