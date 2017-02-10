use std;
use nalgebra as na;
use glium;

pub struct FlyCam {
    roll: f32,
    pitch: f32,
    yaw: f32,
    position: na::Point3<f32>,
}
impl FlyCam {
    pub fn new() -> Self {
        FlyCam {
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
            position: na::Point3::new(0.0, 0.0, 0.0),
        }
    }
    pub fn rotation(&self) -> na::Rotation3<f32> {
        na::Rotation3::new(na::Vector3::new(0.0, self.yaw, 0.0))
        *
        na::Rotation3::new(na::Vector3::new(self.pitch, 0.0, 0.0))
        *
        na::Rotation3::new(na::Vector3::new(0.0, 0.0, self.roll))
        //na::Rotation3::from_euler_angles(self.roll, self.pitch, self.yaw)
    }
    pub fn rotation64(&self) -> na::Rotation3<f64> {
        na::Rotation3::new(na::Vector3::new(0.0, self.yaw as f64, 0.0))
        *
        na::Rotation3::new(na::Vector3::new(self.pitch as f64, 0.0, 0.0))
        *
        na::Rotation3::new(na::Vector3::new(0.0, 0.0, self.roll as f64))
        //na::Rotation3::from_euler_angles(self.roll, self.pitch, self.yaw)
    }
    pub fn inv_rotation(&self) -> na::Rotation3<f32> {
        na::Rotation3::new(na::Vector3::new(0.0, 0.0, self.roll))
        *
        na::Rotation3::new(na::Vector3::new(self.pitch, 0.0, 0.0))
        *
        na::Rotation3::new(na::Vector3::new(0.0, self.yaw, 0.0))
    }
    pub fn inv_rotation64(&self) -> na::Rotation3<f64> {
        na::Rotation3::new(na::Vector3::new(0.0, 0.0, self.roll as f64))
        *
        na::Rotation3::new(na::Vector3::new(self.pitch as f64, 0.0, 0.0))
        *
        na::Rotation3::new(na::Vector3::new(0.0, self.yaw as f64, 0.0))
    }
    pub fn isometry(&self) -> na::Isometry3<f32> {
        na::Isometry3::from_rotation_matrix(*self.position.as_vector(), self.rotation())
    }
    pub fn isometry64(&self) -> na::Isometry3<f64> {
        na::Isometry3::from_rotation_matrix(
            <na::Vector3<f64> as na::Cast<na::Vector3<f32>>>::from(*self.position.as_vector()),
            self.rotation64()
        )
    }
    pub fn rotate(&mut self, yaw: f32, pitch: f32, roll: f32) {
        self.yaw += yaw;
        while self.yaw < -(std::f64::consts::PI as f32) {
            self.yaw += (std::f64::consts::PI * 2.0) as f32;
        }
        while self.yaw > (std::f64::consts::PI as f32) {
            self.yaw -= (std::f64::consts::PI * 2.0) as f32;
        }
        self.pitch += pitch;
        if self.pitch < -((std::f64::consts::PI * 0.5) as f32) {
            self.pitch = -((std::f64::consts::PI * 0.5) as f32);
        }
        if self.pitch > ((std::f64::consts::PI * 0.5) as f32) {
            self.pitch = (std::f64::consts::PI * 0.5) as f32;
        }
        self.roll += roll;
        if self.roll < -((std::f64::consts::PI * 0.5) as f32) {
            self.roll = -((std::f64::consts::PI * 0.5) as f32);
        }
        if self.roll > ((std::f64::consts::PI * 0.5) as f32) {
            self.roll = (std::f64::consts::PI * 0.5) as f32;
        }
    }
    pub fn translate(&mut self, translation: na::Vector3<f32>) {
        self.position += self.rotation() * translation;
    }
}

pub struct FlyCamController {
    last_mouse_pos: na::Point2<i32>,
    mouse_rotate_speed: f32,
    key_move_speed: f32,
    shift_down: bool,
    w_down: bool,
    a_down: bool,
    s_down: bool,
    d_down: bool,
    ctrl_down: bool,
    space_down: bool,
    left_mouse_button_down: bool,
    right_mouse_button_down: bool,
}
impl FlyCamController {
    pub fn new() -> Self {
        FlyCamController {
            last_mouse_pos: na::Point2::new(0, 0),
            mouse_rotate_speed: 0.01,
            key_move_speed: 10.0,
            shift_down: false,
            w_down: false,
            a_down: false,
            s_down: false,
            d_down: false,
            ctrl_down: false,
            space_down: false,
            left_mouse_button_down: false,
            right_mouse_button_down: false,
        }
    }
    pub fn process_event(&mut self, event: &glium::glutin::Event, fly_cam: &mut FlyCam) {
        match event {
            &glium::glutin::Event::MouseMoved(x, y) => {
                let mouse_pos = na::Point2::new(x, y);
                let delta_mouse_pos = mouse_pos - self.last_mouse_pos;
                self.last_mouse_pos = mouse_pos;
                if self.left_mouse_button_down {
                    let delta = self.mouse_rotate_speed * <na::Vector2<f32> as na::Cast<na::Vector2<i32>>>::from(delta_mouse_pos);
                    fly_cam.rotate(-delta.x, -delta.y, 0.0);
                }
            },
            &glium::glutin::Event::KeyboardInput(state, _, Some(code)) => {
                match state {
                    glium::glutin::ElementState::Pressed => {
                        match code {
                            glium::glutin::VirtualKeyCode::W => { self.w_down = true; },
                            glium::glutin::VirtualKeyCode::A => { self.a_down = true; },
                            glium::glutin::VirtualKeyCode::S => { self.s_down = true; },
                            glium::glutin::VirtualKeyCode::D => { self.d_down = true; },
                            glium::glutin::VirtualKeyCode::LShift => { self.shift_down = true; },
                            glium::glutin::VirtualKeyCode::LControl => { self.ctrl_down = true; },
                            glium::glutin::VirtualKeyCode::Space => { self.space_down = true; },
                            _ => {},
                        }
                    },
                    glium::glutin::ElementState::Released => {
                        match code {
                            glium::glutin::VirtualKeyCode::W => { self.w_down = false; },
                            glium::glutin::VirtualKeyCode::A => { self.a_down = false; },
                            glium::glutin::VirtualKeyCode::S => { self.s_down = false; },
                            glium::glutin::VirtualKeyCode::D => { self.d_down = false; },
                            glium::glutin::VirtualKeyCode::LShift => { self.shift_down = false; },
                            glium::glutin::VirtualKeyCode::LControl => { self.ctrl_down = false; },
                            glium::glutin::VirtualKeyCode::Space => { self.space_down = false; },
                            _ => {},
                        }
                    },
                }
            },
            &glium::glutin::Event::MouseInput(state, button) => {
                match state {
                    glium::glutin::ElementState::Pressed => {
                        match button {
                            glium::glutin::MouseButton::Left => { self.left_mouse_button_down = true; },
                            glium::glutin::MouseButton::Right => { self.right_mouse_button_down = true; },
                            _ => {},
                        }
                    },
                    glium::glutin::ElementState::Released => {
                        match button {
                            glium::glutin::MouseButton::Left => { self.left_mouse_button_down = false; },
                            glium::glutin::MouseButton::Right => { self.right_mouse_button_down = false; },
                            _ => {},
                        }
                    }
                }
            }
            _ => {},
        }
    }
    pub fn update(&mut self, delta_time: f32, fly_cam: &mut FlyCam) {
        let mut delta = delta_time * self.key_move_speed;
        if self.shift_down {
            delta *= 10.0;
        }
        if self.s_down { fly_cam.translate(na::Vector3::new(0.0, 0.0,  delta)); }
        if self.w_down { fly_cam.translate(na::Vector3::new(0.0, 0.0, -delta)); }
        if self.a_down { fly_cam.translate(na::Vector3::new(-delta, 0.0, 0.0)); }
        if self.d_down { fly_cam.translate(na::Vector3::new( delta, 0.0, 0.0)); }
        if self.space_down { fly_cam.translate(na::Vector3::new( 0.0,  delta, 0.0)); }
        if self.ctrl_down { fly_cam.translate(na::Vector3::new( 0.0, -delta, 0.0)); }
    }
}
