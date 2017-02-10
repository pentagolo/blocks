use nalgebra as na;

pub struct WorldPos {
    box_pos: BoxPos,
    sub_pos: SubPos,
}

pub struct AABB {
    min: WorldPos,
    max: WorldPos,
}

pub trait Node {
    fn aabb
}

struct Node {
    parent_: *Node,
    next_: *Node,
    prev_: *Node,
    quaternion_: na::Quaternion,
}

struct scene {

}
