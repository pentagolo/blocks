//extern crate nalgebra as na;
use nalgebra as na;
use nalgebra::Norm;
use std::ops::{Deref, DerefMut};
use std::mem::transmute;

/// Maximal depth of the tree.
/// The maximal dimension of the world is (CHUNK_SIDE_LEN^MAX_DEPTH)^3=(16^8)^3=(2^32)^3 boxes.
pub const MAX_DEPTH: u8 = 8;

/// Second logarithm of a side length of a chunk.
/// The chunk of a node is made of CHUNK_SIDE_LEN^3=16^3=4096 16-Bit integer values, which specify
/// the nodes children.
/// The maximal dimension of the world is (CHUNK_SIDE_LEN^MAX_DEPTH)^3=(16^8)^3=(2^32)^3 boxes.
pub const CHUNK_SIDE_LEN_LOG2: u8 = 4;
/// Side length of a chunk.
/// The chunk of a node is made of CHUNK_SIDE_LEN^3=16^3=4096 16-Bit integer values, which specify
/// the nodes children.
/// The maximal dimension of the world is (CHUNK_SIDE_LEN^MAX_DEPTH)^3=(16^8)^3=(2^32)^3 boxes.
pub const CHUNK_SIDE_LEN: u8 = 1 << CHUNK_SIDE_LEN_LOG2;
/// Mask for a chunks side index.
pub const CHUNK_SIDE_MASK: u32 = (CHUNK_SIDE_LEN - 1) as u32;
/// Length of a chunk = CHUNK_SIDE_LEN^3.
pub const CHUNK_LEN: u16 = (CHUNK_SIDE_LEN as u16) * (CHUNK_SIDE_LEN as u16) * (CHUNK_SIDE_LEN as u16);

/// Index of a bit in a child value, which specifies whether the child is an other node or a leaf.
pub const NODE_BIT: u8 = 15;
/// Mask of a bit in a child value, which specifies whether the child is an other node or a leaf.
pub const NODE_BIT_MASK: u16 = 1 << NODE_BIT;

/// Number of bits in a child value, which specify the index to an other node.
pub const NODE_INDEX_BITS: u8 = 15;
/// Mask of bits in a child value, which specify the index to an other node.
pub const NODE_INDEX_BIT_MASK: u16 = (1 << NODE_INDEX_BITS) - 1;

/// Index of a bit in a nodes child value, which specifies whether the child is part of the surface
/// (not surrounded by solid boxes). Only used if the child is a leaf.
pub const SURFACE_BIT: u8 = 14;
/// Mask of a bit in a nodes child value, which specifies whether the child is part of the surface
// (not surrounded by solid boxes). Only used if the child is a leaf.
pub const SURFACE_BIT_MASK: u16 = 1 << SURFACE_BIT;

/// Index of a bit in a nodes child value which specifies whether the node is solid
/// (intransparent). Only used if the child is a leaf.
pub const SOLID_BIT: u8 = 13;
/// Mask of a bit in a nodes child value which specifies whether the node is is solid
/// (intransparent). Only used if the child is a leaf.
pub const SOLID_BIT_MASK: u16 = 1 << SOLID_BIT;

/// Number of bits in a child value, which specify the box.
pub const BOX_SPEC_BITS: u8 = 13;
/// Mask of bits in a child value, which specify the box.
pub const BOX_SPEC_BIT_MASK: u16 = (1 << BOX_SPEC_BITS) - 1;

/*
/// Index of a bit in a nodes child value which specifies whether the node is compressed or static.
pub const COMPRESSED_BIT: u8 = 12;
/// Mask of a bit in a nodes child value which specifies whether the node is compressed or static.
pub const COMPRESSED_BIT_MASK: u16 = 1 << COMPRESSED_BIT;
*/

/// Child of node.
#[derive(Copy, Clone)]
pub struct Child {
    pub value: u16,
}
impl Child {
    pub fn new(value: u16) -> Self {
        Child { value: value }
    }
    pub fn node_from_index(index: u16) -> Self {
        Child::new(
            NODE_BIT_MASK |
            (index & NODE_INDEX_BIT_MASK)
        )
    }
    pub fn leaf_from_surface_solid_box_spec(surface: bool, solid: bool, spec: u16) -> Self {
        Child::new(
            (if surface { SURFACE_BIT_MASK } else { 0 }) |
            (if solid { SOLID_BIT_MASK } else { 0 }) |
            (spec & BOX_SPEC_BIT_MASK)
        )
    }
    pub fn leaf_from_surface_ext_spec(surface: bool, spec: u16) -> Self {
        Child::new(
            (if surface { SURFACE_BIT_MASK } else { 0 }) |
            (spec & (BOX_SPEC_BIT_MASK | SOLID_BIT_MASK))
        )
    }
    pub fn void() -> Self {
        Child::leaf_from_surface_ext_spec(false, 0)
    }
    pub fn is_void(&self) -> bool {
        self.value == 0
    }
    pub fn is_node(&self) -> bool {
        (self.value & NODE_BIT_MASK) != 0
    }
    pub fn set_node(&mut self, node: bool) {
        if node {
            self.value |= NODE_BIT_MASK;
        } else {
            self.value &= !NODE_BIT_MASK;
        }
    }
    pub fn is_surface(&self) -> bool {
        (self.value & SURFACE_BIT_MASK) != 0
    }
    pub fn set_surface(&mut self, surface: bool) {
        if surface {
            self.value |= SURFACE_BIT_MASK;
        } else {
            self.value &= !SURFACE_BIT_MASK;
        }
    }
    pub fn is_solid(&self) -> bool {
        (self.value & SOLID_BIT_MASK) != 0
    }
    pub fn set_solid(&mut self, solid: bool) {
        if solid {
            self.value |= SOLID_BIT_MASK;
        } else {
            self.value &= !SOLID_BIT_MASK;
        }
    }
    pub fn box_spec(&self) -> u16 {
        self.value & BOX_SPEC_BIT_MASK
    }
    pub fn set_box_spec_unmasked(&mut self, box_spec: u16) {
        self.value = (self.value & !BOX_SPEC_BIT_MASK) | box_spec;
    }
    pub fn set_box_spec(&mut self, box_spec: u16) {
        unsafe { self.set_box_spec_unmasked(box_spec & BOX_SPEC_BIT_MASK); }
    }
    pub fn ext_spec(&self) -> u16 {
        self.value & (BOX_SPEC_BIT_MASK | SOLID_BIT_MASK)
    }
    pub fn set_ext_spec_unmasked(&mut self, ext_spec: u16) {
        self.value = (self.value & !(BOX_SPEC_BIT_MASK | SOLID_BIT_MASK)) | ext_spec;
    }
    pub fn set_ext_spec(&mut self, ext_spec: u16) {
        unsafe { self.set_ext_spec_unmasked(ext_spec & (BOX_SPEC_BIT_MASK | SOLID_BIT_MASK)); }
    }
    pub fn node_index(&self) -> u16 {
        self.value & NODE_INDEX_BIT_MASK
    }
    pub fn set_node_index_unmasked(&mut self, node_index: u16) {
        self.value = (self.value & !NODE_INDEX_BIT_MASK) | node_index;
    }
    pub fn set_node_index(&mut self, node_index: u16) {
        unsafe { self.set_node_index_unmasked(node_index & NODE_INDEX_BIT_MASK); }
    }
    pub unsafe fn as_leaf(&self) -> &Leaf {
        transmute::<&Self, &Leaf>(self)
    }
    pub unsafe fn as_leaf_mut(&mut self) -> &mut Leaf {
        transmute::<&mut Self, &mut Leaf>(self)
    }
    pub unsafe fn as_hidden(&self) -> &HiddenLeaf {
        transmute::<&Self, &HiddenLeaf>(self)
    }
    pub unsafe fn as_hidden_mut(&mut self) -> &mut HiddenLeaf {
        transmute::<&mut Self, &mut HiddenLeaf>(self)
    }
}
impl Deref for Child {
    type Target = u16;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl DerefMut for Child {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

/// Leaf wrapper of a child of a node.
#[derive(Copy, Clone)]
pub struct Leaf {
    child_: Child,
}
impl Leaf {
    pub unsafe fn new(child: Child) -> Self {
        Leaf { child_: child }
    }
    pub fn void() -> Self {
        unsafe { Leaf::new(Child::void()) }
    }
    pub fn from_solid_box_spec(solid: bool, spec: u16) -> Self {
        unsafe { Leaf::new(Child::leaf_from_surface_solid_box_spec(false, solid, spec)) }
    }
    pub fn from_ext_spec(spec: u16) -> Self {
        unsafe { Leaf::new(Child::leaf_from_surface_ext_spec(false, spec)) }
    }
    pub fn set_solid(&mut self, solid: bool) {
        self.child_.set_solid(solid);
    }
    pub fn set_box_spec(&mut self, box_spec: u16) {
        self.child_.set_box_spec(box_spec);
    }
    pub fn set_ext_spec(&mut self, ext_spec: u16) {
        self.child_.set_ext_spec(ext_spec);
    }
    pub fn as_child(&self) -> &Child {
        &self.child_
    }
    pub unsafe fn as_child_mut(&mut self) -> &mut Child {
        &mut self.child_
    }
}
impl Deref for Leaf {
    type Target = Child;
    fn deref(&self) -> &Self::Target {
        &self.child_
    }
}

#[derive(Copy, Clone)]
pub struct HiddenLeaf {
    child_: Child,
}
impl HiddenLeaf {
    pub unsafe fn new(child: Child) -> Self {
        HiddenLeaf { child_: child }
    }
    pub fn from_box_spec(spec: u16) -> Self {
        unsafe { HiddenLeaf::new(Child::leaf_from_surface_solid_box_spec(false, true, spec)) }
    }
    pub fn set_box_spec(&mut self, box_spec: u16) {
        self.child_.set_box_spec(box_spec);
    }
    pub fn as_child(&self) -> &Child {
        &self.child_
    }
    pub unsafe fn as_child_mut(&mut self) -> &mut Child {
        &mut self.child_
    }
}
impl Deref for HiddenLeaf {
    type Target = Child;
    fn deref(&self) -> &Self::Target {
        &self.child_
    }
}

/// A chunk is an array of child values of a node. A chunk and an info form a node. They are stored
/// in direfferent arrays in the tree, but share the same indices.
/// A chunk is made of one 16-Bit integer value per child and CHUNK_SIDE_LEN^3=16^3=4096 children
/// per node, where the LEAF_BIT decides whether the child is either a 15-Bit index to an other
/// node or whether it is a leaf with a read-only bit (SURFACE_BIT) indicating whether the leaf is
/// part of the surface (for rendering), a predefined bit (SOLID_BIT) specifying whether the leaf
/// is not partial transparent and a 13-Bit user defined box specifier. Zero is the void box
/// specifier. When the space a node would represent in the world is only made of the same boxes
/// (including void), no chunks and infos will be allocated for this node and its children. Instead
/// the parent node will only contain a leaf child value for this node.
#[derive(Copy)]
pub struct Chunk<C: Copy> {
    pub children: [C; CHUNK_LEN as usize],
}
impl Chunk<Child> {
    pub unsafe fn as_leaf(&self) -> &Chunk<Leaf> {
        transmute::<&Self, &Chunk<Leaf>>(self)
    }
    pub unsafe fn as_leaf_mut(&mut self) -> &mut Chunk<Leaf> {
        transmute::<&mut Self, &mut Chunk<Leaf>>(self)
    }
    pub unsafe fn as_hidden_leaf(&self) -> &Chunk<HiddenLeaf> {
        transmute::<&Self, &Chunk<HiddenLeaf>>(self)
    }
    pub unsafe fn as_hidden_leaf_mut(&mut self) -> &mut Chunk<HiddenLeaf> {
        transmute::<&mut Self, &mut Chunk<HiddenLeaf>>(self)
    }
}
impl Chunk<Leaf> {
    pub fn as_child(&self) -> &Chunk<Child> {
        unsafe { transmute::<&Self, &Chunk<Child>>(self) }
    }
    pub unsafe fn as_child_mut(&mut self) -> &Chunk<Child> {
        transmute::<&mut Self, &mut Chunk<Child>>(self)
    }
}
impl Chunk<HiddenLeaf> {
    pub fn as_child(&self) -> &Chunk<Leaf> {
        unsafe { transmute::<&Self, &Chunk<Leaf>>(self) }
    }
    pub unsafe fn as_child_mut(&mut self) -> &mut Chunk<Child> {
        transmute::<&mut Self, &mut Chunk<Child>>(self)
    }
}
impl<C: Copy> Clone for Chunk<C> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<C: Copy> Deref for Chunk<C> {
    type Target = [C; CHUNK_LEN as usize];
    fn deref(&self) -> &Self::Target { &self.children }
}
impl<C: Copy> DerefMut for Chunk<C> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.children }
}

/// Chunks where all children are hidden may be compressed and decompressed.
pub trait Compression {
    /// Compress a chunk to a leaf.
    fn compress(&mut self, pos: na::Vector3<u32>, depth: u8, chunk: &Chunk<HiddenLeaf>) -> Option<HiddenLeaf>;
    /// Decompress a leaf into a chunk.
    fn decompress(&mut self, pos: na::Vector3<u32>, depth: u8, leaf: HiddenLeaf, chunk: &mut Chunk<HiddenLeaf>);
}

/// Additional information about a node. A chunk and an info form a node. They are stored in
/// direfferent arrays in the tree, but share the same indices.
#[derive(Clone)]
pub struct Info {
    /// Index of the parent node, 0 if root.
    pub parent_index: u16,
    /// Number of void children.
    pub num_void_children: u16,
    /// Number of children which are solid.
    pub num_solid_children: u16,
    /// Number of values which have the visible flag set in the corresponding chunk.
    pub num_surface_children: u16,
}
/// An octree like structer with CHUNK_LEN^3=16^3=4096 entries per node.
/// On its leafs it stores box types.
/// The maximal dimension of the world is (CHUNK_LEN^MAX_DEPTH)^3=(16^8)^3=(2^32)^3 boxes.
pub struct Tree<C: Compression + ?Sized> {
    chunks_: Vec<Chunk<Child>>,
    infos_: Vec<Info>,
    depth_: u8,
    first_free_node_: u16,
    num_free_nodes_: u16,
    compression_: C,
}
impl<C: Compression> Tree<C> {
    /// Creates a new empty tree.
    pub fn new(depth: u8, max_nodes: u16, compression: C) -> Self {
        unsafe {
            // Check for valid range of depth.
            if depth < 2 || depth > MAX_DEPTH {
                panic!(
                    "depth must be in range {} - {}, but {} was specified",
                    2,
                    MAX_DEPTH,
                    depth
                );
            }
            // Check for valid range of max_nodes.
            if max_nodes < (depth as u16) || max_nodes > (1 << NODE_INDEX_BITS) {
                panic!(
                    "max_nodes must be in range (depth={}) - {}, but {} was specified",
                    depth,
                    (1 << NODE_INDEX_BITS),
                    max_nodes
                );
            }
            // Create chunks.
            let mut chunks = Vec::new();
            chunks.resize(max_nodes as usize, Chunk {
                children: [Child::void(); CHUNK_LEN as usize]
            } );
            for i in 1..(max_nodes - 1) {
                chunks.get_unchecked_mut(i as usize).get_unchecked_mut(0).value = (i + 1) as u16;
            }
            chunks.get_unchecked_mut((max_nodes - 1) as usize).get_unchecked_mut(0).value = 0;
            // Create infos.
            let mut infos = Vec::new();
            infos.resize(max_nodes as usize, Info {
                parent_index: 0,
                num_void_children: CHUNK_LEN,
                num_solid_children: 0,
                num_surface_children: 0,
            });
            // Create and return the tree.
            Tree {
                chunks_: chunks,
                infos_: infos,
                depth_: depth,
                first_free_node_: 1,
                num_free_nodes_: max_nodes - 1,
                compression_: compression,
            }
        }
    }
    pub fn chunks(&self) -> &[Chunk<Child>] {
        &self.chunks_
    }
    pub unsafe fn chunks_mut(&mut self) -> &mut [Chunk<Child>] {
        &mut self.chunks_
    }
    pub fn infos(&self) -> &[Info] {
        &self.infos_
    }
    pub unsafe fn infos_mut(&mut self) -> &mut [Info] {
        &mut self.infos_
    }
    pub fn depth(&self) -> &u8 {
        &self.depth_
    }
    pub fn first_free_node(&self) -> &u16 {
        &self.first_free_node_
    }
    pub unsafe fn first_free_node_mut(&mut self) -> &mut u16 {
        &mut self.first_free_node_
    }
    pub fn num_free_nodes(&self) -> &u16 {
        &self.num_free_nodes_
    }
    pub unsafe fn num_free_nodes_mut(&mut self) -> &mut u16 {
        &mut self.num_free_nodes_
    }
    /// Get the type of a box at a specific position.
    pub fn get_at_pos(&self, mut pos: na::Vector3<u32>) -> Leaf {
        unsafe {
            pos = {
                let init_rotate = (self.depth_ * CHUNK_SIDE_LEN_LOG2) as u32;
                na::Vector3::new(
                    pos.x.rotate_right(init_rotate),
                    pos.y.rotate_right(init_rotate),
                    pos.z.rotate_right(init_rotate)
                )
            };
            let mut node_index: u16 = 0;
            let mut depth = self.depth_;
            loop {
                pos = {
                    const CHUNK_SIDE_LEN_LOG2_U32: u32 = CHUNK_SIDE_LEN_LOG2 as u32;
                    na::Vector3::new(
                        pos.x.rotate_left(CHUNK_SIDE_LEN_LOG2_U32),
                        pos.y.rotate_left(CHUNK_SIDE_LEN_LOG2_U32),
                        pos.z.rotate_left(CHUNK_SIDE_LEN_LOG2_U32),
                    )
                };
                depth -= 1;
                let index = (
                    ((pos.x & CHUNK_SIDE_MASK) as u16) << (0 * CHUNK_SIDE_LEN_LOG2)
                    |
                    ((pos.y & CHUNK_SIDE_MASK) as u16) << (1 * CHUNK_SIDE_LEN_LOG2)
                    |
                    ((pos.z & CHUNK_SIDE_MASK) as u16) << (2 * CHUNK_SIDE_LEN_LOG2)
                );
                let child = *self.chunks_.get_unchecked(node_index as usize).get_unchecked(index as usize);
                if !child.is_node() {
                    if child.ext_spec() != 0 && depth != 0 {
                        // Handle compressed or static
                        panic!("not jet implemented");
                    }
                    return Leaf::new(child);
                }
                node_index = child.node_index();
            }
        }
    }
    pub unsafe fn neighbor_mut(&mut self, index: u16, mut pos: na::Vector3<u32>, dir: na::Vector3<u32>) -> &mut Child {
        let mask = 1u32.wrapping_shl((self.depth_ * CHUNK_SIDE_LEN_LOG2) as u32).wrapping_sub(1);
        pos.x = pos.x.wrapping_add(dir.x) & mask;
        pos.y = pos.y.wrapping_add(dir.y) & mask;
        pos.z = pos.z.wrapping_add(dir.z) & mask;
        pos = {
            let init_rotate = (self.depth_ * CHUNK_SIDE_LEN_LOG2) as u32;
            na::Vector3::new(
                pos.x.rotate_right(init_rotate),
                pos.y.rotate_right(init_rotate),
                pos.z.rotate_right(init_rotate)
            )
        };
        let mut node_index: u16 = 0;
        let mut depth = self.depth_;
        loop {
            pos = {
                const CHUNK_SIDE_LEN_LOG2_U32: u32 = CHUNK_SIDE_LEN_LOG2 as u32;
                na::Vector3::new(
                    pos.x.rotate_left(CHUNK_SIDE_LEN_LOG2_U32),
                    pos.y.rotate_left(CHUNK_SIDE_LEN_LOG2_U32),
                    pos.z.rotate_left(CHUNK_SIDE_LEN_LOG2_U32),
                )
            };
            depth -= 1;
            let index = (
                ((pos.x & CHUNK_SIDE_MASK) as u16) << (0 * CHUNK_SIDE_LEN_LOG2)
                |
                ((pos.y & CHUNK_SIDE_MASK) as u16) << (1 * CHUNK_SIDE_LEN_LOG2)
                |
                ((pos.z & CHUNK_SIDE_MASK) as u16) << (2 * CHUNK_SIDE_LEN_LOG2)
            );
            let child = *self.chunks_.get_unchecked(node_index as usize).get_unchecked(index as usize);
            if !child.is_node() {
                if child.ext_spec() != 0 && depth != 0 {
                    // Handle compressed or static
                    panic!("not jet implemented");
                }
                return self.chunks_.get_unchecked_mut(node_index as usize).get_unchecked_mut(index as usize)
            }
            node_index = child.node_index();
        }
    }
    pub fn set_non_void_at_pos(&mut self, mut pos: na::Vector3<u32>, new_leaf: Leaf) -> bool {
        unsafe {
            if new_leaf.is_void() {
                panic!("expected non void");
            }
            pos = {
                let init_rotate = (self.depth_ * CHUNK_SIDE_LEN_LOG2) as u32;
                na::Vector3::new(
                    pos.x.rotate_right(init_rotate),
                    pos.y.rotate_right(init_rotate),
                    pos.z.rotate_right(init_rotate)
                )
            };
            let mut chunk: u16 = 0;
            let mut depth = self.depth_;
            loop {
                pos = {
                    const CHUNK_SIDE_LEN_LOG2_U32: u32 = CHUNK_SIDE_LEN_LOG2 as u32;
                    na::Vector3::new(
                        pos.x.rotate_left(CHUNK_SIDE_LEN_LOG2_U32),
                        pos.y.rotate_left(CHUNK_SIDE_LEN_LOG2_U32),
                        pos.z.rotate_left(CHUNK_SIDE_LEN_LOG2_U32),
                    )
                };
                depth -= 1;
                let index = (
                    ((pos.x & CHUNK_SIDE_MASK) as u16) << (0 * CHUNK_SIDE_LEN_LOG2)
                    |
                    ((pos.y & CHUNK_SIDE_MASK) as u16) << (1 * CHUNK_SIDE_LEN_LOG2)
                    |
                    ((pos.z & CHUNK_SIDE_MASK) as u16) << (2 * CHUNK_SIDE_LEN_LOG2)
                );
                if depth == 0 {
                    let orig_leaf = {
                        let leaf = self.chunks_.get_unchecked_mut(chunk as usize).get_unchecked_mut(index as usize);
                        let orig_leaf = *leaf;
                        *leaf = *new_leaf.as_child();
                        orig_leaf
                    };
                    if orig_leaf.is_solid() == new_leaf.is_solid() {
                        // TODO: Debug! Just remove
                        panic!("not jet supported {} {} {:b} {:b}", chunk, pos, orig_leaf.value, new_leaf.value);
                        self.chunks_.get_unchecked_mut(chunk as usize).get_unchecked_mut(index as usize).set_surface(orig_leaf.is_surface());
                    } else if !new_leaf.is_solid() {
                        // TODO: Debug! Just remove
                        panic!("usage of transparent data not jet supported");
                        for z in 0..5u32 {
                            for y in 0..5u32 {
                                for x in 0..5u32 {
                                    let d = na::Vector3::new(x.wrapping_sub(2), y.wrapping_sub(2), z.wrapping_sub(2));
                                    let n = self.neighbor_mut(index, pos, d);
                                    if !n.is_void() {
                                        n.set_surface(true);
                                    }
                                }
                            }
                        }
                    } else {
                        let mut flags = [0u32; 5];
                        for z in 0..5u32 {
                            let mut i = 0;
                            for y in 0..5u32 {
                                for x in 0..5u32 {
                                    let d = na::Vector3::new(x.wrapping_sub(2), y.wrapping_sub(2), z.wrapping_sub(2));
                                    if !self.neighbor_mut(index, pos, d).is_solid() {
                                        flags[z as usize] |= 1u32 << i;
                                    }
                                    i += 1;
                                }
                            }
                        }
                        for z in 0..3u32 {
                            let mut i: u8 = 0;
                            for y in 0..3u32 {
                                for x in 0..3u32 {
                                    let d = na::Vector3::new(x.wrapping_sub(1), y.wrapping_sub(1), z.wrapping_sub(1));
                                    let n = self.neighbor_mut(index, pos, d);
                                    let surface = (
                                        (
                                            flags[(z + 0) as usize] |
                                            flags[(z + 1) as usize] |
                                            flags[(z + 2) as usize]
                                        ) &
                                        (0b0100011100010u32 << i)
                                    ) != 0;
                                    if !n.is_void() {
                                        n.set_surface(surface);
                                    }
                                    i += 1;
                                }
                                i += 2;
                            }
                        }
                    }
                    if orig_leaf.is_void() {
                        let info = self.infos_.get_unchecked_mut(chunk as usize);
                        info.num_void_children += 1;
                    }
                    return true;
                } else {
                    let mut child = *self.chunks_.get_unchecked_mut(chunk as usize).get_unchecked_mut(index as usize);
                    if child.is_void() {
                        let new_node = self.first_free_node_;
                        if new_node == 0 {
                            // TODO: reverse previus inserted nodes
                            return false;
                        }
                        self.num_free_nodes_ -= 1;
                        self.first_free_node_ = self.chunks_.get_unchecked(new_node as usize).get_unchecked(0).value;
                        *self.chunks_.get_unchecked_mut(new_node as usize).get_unchecked_mut(0) = Child::void();
                        child = Child::node_from_index(new_node);
                        *self.chunks_.get_unchecked_mut(chunk as usize).get_unchecked_mut(index as usize) = child;
                        self.infos_.get_unchecked_mut(new_node as usize).parent_index = chunk;
                    }
                    chunk = child.node_index();
                }
            }
        }
    }
    pub fn set_void_at_pos(&mut self, pos: na::Vector3<u32>) {
        panic!("not jet implemented");
        // TODO
    }
    pub fn set_at_pos(&mut self, pos: na::Vector3<u32>, new_leaf: Leaf) -> bool {
        if new_leaf.is_void() {
            self.set_void_at_pos(pos);
            true
        } else {
            self.set_non_void_at_pos(pos, new_leaf)
        }
    }

    pub fn cast_view<Callback: FnMut(na::Vector3<u32>, Child)>(
        &self,
        origin: na::Point3<f64>, planes: [na::Vector3<f64>; 4], dist: f64,
        callback: &mut Callback
    ) {
        unsafe {
            // (x-ox)*px + (y-oy)*py + (z-oz)*pz > 0
            let mut deltas = [na::Vector3::new(0i64, 0, 0); 5];
            let mut dists = [0i64; 5];
            for i in 0..4 {
                deltas[i].x = (planes[i].x * ((1 << 28) as f64)).ceil() as i64;
                deltas[i].y = (planes[i].y * ((1 << 28) as f64)).ceil() as i64;
                deltas[i].z = (planes[i].z * ((1 << 28) as f64)).ceil() as i64;
                dists[i] = (
                    (-origin.x * (deltas[i].x as f64)).ceil() as i64
                    +
                    (-origin.y * (deltas[i].y as f64)).ceil() as i64
                    +
                    (-origin.z * (deltas[i].z as f64)).ceil() as i64
                );
                deltas[i].x <<= (self.depth_ - 1) * CHUNK_SIDE_LEN_LOG2;
                deltas[i].y <<= (self.depth_ - 1) * CHUNK_SIDE_LEN_LOG2;
                deltas[i].z <<= (self.depth_ - 1) * CHUNK_SIDE_LEN_LOG2;
            }
            let mut plane4 = na::Vector3::new(0.0f64, 0.0, 0.0);
            for i in 0..4 {
                plane4 -= planes[i];
            }
            plane4 = plane4.normalize();
            deltas[4].x = (plane4.x * ((1 << 28) as f64)).ceil() as i64;
            deltas[4].y = (plane4.y * ((1 << 28) as f64)).ceil() as i64;
            deltas[4].z = (plane4.z * ((1 << 28) as f64)).ceil() as i64;
            dists[4] = (
                (-origin.x * (deltas[4].x as f64)).ceil() as i64
                +
                (-origin.y * (deltas[4].y as f64)).ceil() as i64
                +
                (-origin.z * (deltas[4].z as f64)).ceil() as i64
                +
                ((dist * ((1 << 28) as f64)).ceil() as i64)
            );
            deltas[4].x <<= (self.depth_ - 1) * CHUNK_SIDE_LEN_LOG2;
            deltas[4].y <<= (self.depth_ - 1) * CHUNK_SIDE_LEN_LOG2;
            deltas[4].z <<= (self.depth_ - 1) * CHUNK_SIDE_LEN_LOG2;

            for i in 0..5 {
                if deltas[i].x > 0 { dists[i] += deltas[i].x; }
                if deltas[i].y > 0 { dists[i] += deltas[i].y; }
                if deltas[i].z > 0 { dists[i] += deltas[i].z; }
            }

            let mut depth: u8 = (self.depth_ as u8) - 1;
            let mut chunk: u16 = 0;
            let mut pos = na::Vector3::new(0u32, 0, 0);
            let mut index: u16 = 0;
            loop {
                if dists[0] >= 0 && dists[1] >= 0 && dists[2] >= 0 && dists[3] >= 0 && dists[4] >= 0 {
                    let child = *self.chunks_.get_unchecked(chunk as usize).get_unchecked(index as usize);
                    if child.is_node() {
                        depth -= 1;
                        chunk = child.node_index();
                        index = 0;
                        pos.x *= CHUNK_SIDE_LEN as u32;
                        pos.y *= CHUNK_SIDE_LEN as u32;
                        pos.z *= CHUNK_SIDE_LEN as u32;
                        for i in 0..5 {
                            if deltas[i].x > 0 { dists[i] -= deltas[i].x; }
                            if deltas[i].y > 0 { dists[i] -= deltas[i].y; }
                            if deltas[i].z > 0 { dists[i] -= deltas[i].z; }
                        }
                        for i in 0..5 {
                            deltas[i].x /= CHUNK_SIDE_LEN as i64;
                            deltas[i].y /= CHUNK_SIDE_LEN as i64;
                            deltas[i].z /= CHUNK_SIDE_LEN as i64;
                        }
                        for i in 0..5 {
                            if deltas[i].x > 0 { dists[i] += deltas[i].x; }
                            if deltas[i].y > 0 { dists[i] += deltas[i].y; }
                            if deltas[i].z > 0 { dists[i] += deltas[i].z; }
                        }
                        continue;
                    } else if child.is_surface() {
                        callback(pos, child);
                    }
                }
                loop {
                    for i in 0..5 {
                        dists[i] += deltas[i].x;
                    }
                    if (pos.x % (CHUNK_SIDE_LEN as u32)) != ((CHUNK_SIDE_LEN - 1) as u32) {
                        pos.x += 1;
                    } else {
                        pos.x = (pos.x / (CHUNK_SIDE_LEN as u32)) * (CHUNK_SIDE_LEN as u32);
                        for i in 0..5 {
                            dists[i] -= deltas[i].x * (CHUNK_SIDE_LEN as i64);
                        }
                        for i in 0..5 {
                            dists[i] += deltas[i].y;
                        }
                        if (pos.y % (CHUNK_SIDE_LEN as u32)) != ((CHUNK_SIDE_LEN - 1) as u32) {
                            pos.y += 1;
                        } else {
                            pos.y = (pos.y / (CHUNK_SIDE_LEN as u32)) * (CHUNK_SIDE_LEN as u32);
                            for i in 0..5 {
                                dists[i] -= deltas[i].y * (CHUNK_SIDE_LEN as i64);
                            }
                            for i in 0..5 {
                                dists[i] += deltas[i].z;
                            }
                            if (pos.z % (CHUNK_SIDE_LEN as u32)) != ((CHUNK_SIDE_LEN - 1) as u32) {
                                pos.z += 1;
                            } else {
                                depth += 1;
                                if depth == (self.depth_ as u8) {
                                    return;
                                }
                                pos.z = (pos.z / (CHUNK_SIDE_LEN as u32)) * (CHUNK_SIDE_LEN as u32);
                                for i in 0..5 {
                                    dists[i] -= deltas[i].z * (CHUNK_SIDE_LEN as i64);
                                }
                                chunk = self.infos_.get_unchecked(chunk as usize).parent_index;
                                pos.x /= CHUNK_SIDE_LEN as u32;
                                pos.y /= CHUNK_SIDE_LEN as u32;
                                pos.z /= CHUNK_SIDE_LEN as u32;
                                for i in 0..5 {
                                    if deltas[i].x > 0 { dists[i] -= deltas[i].x; }
                                    if deltas[i].y > 0 { dists[i] -= deltas[i].y; }
                                    if deltas[i].z > 0 { dists[i] -= deltas[i].z; }
                                }
                                for i in 0..5 {
                                    deltas[i].x *= CHUNK_SIDE_LEN as i64;
                                    deltas[i].y *= CHUNK_SIDE_LEN as i64;
                                    deltas[i].z *= CHUNK_SIDE_LEN as i64;
                                }
                                for i in 0..5 {
                                    if deltas[i].x > 0 { dists[i] += deltas[i].x; }
                                    if deltas[i].y > 0 { dists[i] += deltas[i].y; }
                                    if deltas[i].z > 0 { dists[i] += deltas[i].z; }
                                }
                                index = (
                                    ((pos.x % (CHUNK_SIDE_LEN as u32)) * 1)
                                    |
                                    ((pos.y % (CHUNK_SIDE_LEN as u32)) * (CHUNK_SIDE_LEN as u32))
                                    |
                                    ((pos.z % (CHUNK_SIDE_LEN as u32)) * (((CHUNK_SIDE_LEN as u16)*(CHUNK_SIDE_LEN as u16)) as u32))
                                ) as u16;
                                continue;
                            }
                        }
                    }
                    index += 1;
                    break;
                }
            }
        }
    }
}
