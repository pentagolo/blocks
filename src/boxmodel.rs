use glium;
use nalgebra as na;
use boxtree;
use std::io::{BufRead, Seek};
use std::borrow::Cow;
use image;

/// Vertex of the vertex buffer.
#[derive(Copy, Clone)]
pub struct Vertex {
    /// Position betwwen [0, 0, 0] and [1, 1, 1].
    pub position: [f32; 3],
    /// Normal of the vertex.
    pub normal: [f32; 3],
    /// Face index between 0 and 5 this vertex belongs to.
    pub face: u32,
    /// Texture coordinate.
    pub tex_coord: [f32; 2],
}
implement_vertex!(Vertex, position, normal, face, tex_coord);
/// Type of the vertex buffer.
pub type VertexBuffer = glium::VertexBuffer<Vertex>;

/// Instance of an instance buffer.
#[derive(Copy, Clone)]
pub struct Instance {
    /// The position of the box in the world.
    pub box_pos: [u32; 3],
    /// The type of the box.
    pub box_type: u32,
}
implement_vertex!(Instance, box_pos, box_type);
/// Type of the instance buffer.
pub type InstanceBuffer = glium::VertexBuffer<Instance>;

/// Texture which stores indices of a tile for each box type and face.
pub type BoxTypeFaceTileMapTex = glium::texture::Texture1d;
pub fn box_type_face_tile_map_tex_from_array<F: glium::backend::Facade>(facade: &F, box_type_face_tile_map: &[u16]) -> BoxTypeFaceTileMapTex {
    /*let image = glium::texture::RawImage1d {
        data: Cow::Borrowed(box_type_face_tile_map),
        width: box_type_face_tile_map.len() as u32,
        format: glium::texture::ClientFormat::U16,
    };*/
    BoxTypeFaceTileMapTex::new(facade, box_type_face_tile_map).unwrap()
}

/// The tile color texture array is an 2d texture array. Each texture is filled with the tiles colors.
pub type TileColorTexArray = glium::texture::Texture2dArray;
pub fn tile_color_tex_array_from_images<F: glium::backend::Facade>(facade: &F, images: &[image::RgbaImage]) -> TileColorTexArray {
    let images = images.iter().map(|image| {
        let dimensions = image.dimensions();
        glium::texture::RawImage2d::from_raw_rgba_reversed(image.to_vec(), dimensions)
    }).collect::<Vec<_>>();
    TileColorTexArray::new(facade, images).unwrap()
}

pub struct Tiles {
    pub box_type_face_tile_map_tex: BoxTypeFaceTileMapTex,
    pub tile_color_tex_array: TileColorTexArray,
}
impl Tiles {
    pub fn new(box_type_face_tile_map_tex: BoxTypeFaceTileMapTex, tile_color_tex_array: TileColorTexArray) -> Self {
        Tiles {
            box_type_face_tile_map_tex: box_type_face_tile_map_tex,
            tile_color_tex_array: tile_color_tex_array,
        }
    }
    /*pub fn load<F: glium::backend::Facade, R: BufRead + Seek>(facade: &F, read: R) -> Self {
        let image = image::load(read, image::PNG).unwrap().to_rgba();
        let image_dimensions = image.dimensions();
        let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dimensions);
        let texture = glium::texture::Texture2d::new(facade, image).unwrap();
        Texture { texture: texture }
    }*/
}

pub struct Model {
    pub program: glium::Program,
    pub vertex_buffer: glium::VertexBuffer<Vertex>,
    //pub index_buffer: glium::IndexBuffer<u8>,
}
impl Model {
    pub fn new<F: glium::backend::Facade>(facade: &F) -> Model {
        let program = {
            let vertex_shader_src = r#"
                #version 150

                uniform sampler1D box_type_face_tile_map_tex;

                in uvec3 box_pos;
                in uint box_type;

                in vec3 position;
                in vec3 normal;
                in uint face;
                in vec2 tex_coord;

                out vec3 v_normal;
                out vec3 v_position;
                out vec3 v_tex_coord;
                out vec3 v_color;

                uniform mat4 matrix;

                void main() {
                    /*vec4 value = texture(map, vec2(
                        box_tex_index % 16,
                        box_tex_index / 16,
                    ));*/
                    if (box_type == uint(0)) {
                        v_color = vec3(1.0, 0.0, 0.0);
                    } else {
                        v_color = vec3(0.0, 0.0, 0.0);
                    }
                    vec4 value = texelFetch(box_type_face_tile_map_tex, int(box_type) * 6 + int(face), 0);
                    v_tex_coord = vec3(tex_coord, value.r);
                    v_normal = transpose(inverse(mat3(matrix))) * normal;
                    gl_Position = matrix * vec4(position + box_pos, 1.0);
                    v_position = gl_Position.xyz / gl_Position.w;
                }
            "#;
            let fragment_shader_src = r#"
                #version 150

                uniform sampler2DArray tile_color_tex_array;

                in vec3 v_normal;
                in vec3 v_position;
                in vec3 v_tex_coord;
                in vec3 v_color;

                out vec4 color;

                const vec3 light = vec3(-1.0, 0.4, 0.9);

                const vec3 ambient_color = vec3(0.5, 0.5, 0.5);
                const vec3 diffuse_color = vec3(0.5, 0.5, 0.5);
                const vec3 specular_color = vec3(0.2, 0.2, 0.2);

                void main() {
                    float diffuse = max(dot(normalize(v_normal), normalize(light)), 0.0);

                    vec3 camera_dir = normalize(-v_position);
                    vec3 half_direction = normalize(normalize(light) + camera_dir);
                    float specular = pow(max(dot(half_direction, normalize(v_normal)), 0.0), 16.0);

                    color = vec4(texture(tile_color_tex_array, v_tex_coord).rgb * (ambient_color + diffuse * diffuse_color + specular * specular_color), 1.0);
                    //color = texture(color_tex, v_tex_coord);
                    //color = vec4(v_color, 1);
                }
            "#;
            glium::Program::from_source(facade, vertex_shader_src, fragment_shader_src, None).unwrap()
        };
        let vertex_buffer = {
            let vecs: Vec<(i8, i8, i8)> = vec!(
                (0, 0, 0), (1, 0, 0), (1, 1, 0),  (1, 1, 0), (0, 1, 0), (0, 0, 0),
                (1, 0, 0), (1, 0, 1), (1, 1, 1),  (1, 1, 1), (1, 1, 0), (1, 0, 0),
                (1, 0, 1), (0, 0, 1), (0, 1, 1),  (0, 1, 1), (1, 1, 1), (1, 0, 1),
                (0, 0, 1), (0, 0, 0), (0, 1, 0),  (0, 1, 0), (0, 1, 1), (0, 0, 1),
                (0, 1, 0), (1, 1, 0), (1, 1, 1),  (1, 1, 1), (0, 1, 1), (0, 1, 0),
                (0, 0, 1), (1, 0, 1), (1, 0, 0),  (1, 0, 0), (0, 0, 0), (0, 0, 1)
            );
            let tex_coords: [(u8, u8); 6] = [
                (0, 0), (1, 0), (1, 1),  (1, 1), (0, 1), (0, 0)
            ];
            let mut vertices = vecs.iter().map(|v| {
                let pos = [v.0 as f32, v.1 as f32, v.2 as f32];
                let norm = [pos[0] - 0.5, pos[1] - 0.5, pos[2] - 0.5];
                Vertex {
                    position: pos,
                    normal: pos,
                    face: 0,
                    tex_coord: [0.0, 0.0],
                }
            }).collect::<Vec<Vertex>>();
            for f in 0..6 {
                let mut acc = [0.0f32, 0.0, 0.0];
                for v in 0..6 {
                    for i in 0..3 {
                        acc[i] += vertices[f*6 + v].normal[i];
                    }
                }
                let len = (acc[0]*acc[0] + acc[1]*acc[1] + acc[2]*acc[2]).sqrt();
                for v in 0..6 {
                    for i in 0..3 {
                        vertices[f*6 + v].normal[i] = acc[i] / len;
                    }
                }
                for v in 0..6 {
                    vertices[f*6 + v].face = f as u32;
                    vertices[f*6 + v].tex_coord = [tex_coords[v].0 as f32, tex_coords[v].1 as f32];
                }
            }
            glium::VertexBuffer::new(facade, &vertices).unwrap()
        };
        Model { program: program, vertex_buffer: vertex_buffer }
    }
    pub fn draw<S: glium::Surface>(&self, target: &mut S, matrix: &[[f32; 4]; 4], instance_buffer: &glium::VertexBuffer<Instance>, tiles: &Tiles, params: &glium::DrawParameters) {
        let uniforms = uniform! {
            matrix: matrix.clone(),
            box_type_face_tile_map_tex: &tiles.box_type_face_tile_map_tex,
            tile_color_tex_array: &tiles.tile_color_tex_array,
        };
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
        target.draw(
            (&self.vertex_buffer, instance_buffer.per_instance().unwrap()),
            &indices,
            &self.program,
            &uniforms,
            params
        ).unwrap();
    }
}
