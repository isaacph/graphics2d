use bytemuck::Pod;

#[repr(C)]
#[derive(Clone, Copy, Pod, Debug)]
pub struct Mat4 {
    pub data: [[f32; 4]; 4],
}
unsafe impl bytemuck::Zeroable for Mat4 {}

#[repr(C)]
#[derive(Clone, Copy, Pod, Debug)]
pub struct Vec2 {
    pub data: [f32; 2],
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Debug)]
pub struct DVec2 {
    pub x: f32,
    pub y: f32,
}
unsafe impl bytemuck::Zeroable for Vec2 {}
unsafe impl bytemuck::Zeroable for DVec2 {}

#[repr(C)]
#[derive(Clone, Copy, Pod, Debug)]
pub struct Vec3 {
    pub data: [f32; 3],
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Debug)]
pub struct DVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
unsafe impl bytemuck::Zeroable for Vec3 {}
unsafe impl bytemuck::Zeroable for DVec3 {}

#[repr(C)]
#[derive(Clone, Copy, Pod, Debug)]
pub struct Vec4 {
    pub data: [f32; 4],
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Debug)]
pub struct DVec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
unsafe impl bytemuck::Zeroable for Vec4 {}
unsafe impl bytemuck::Zeroable for DVec4 {}

pub trait MultiplyMat4 {
    fn mul(&self, other: &Mat4) -> Mat4;
}

pub trait MultiplyVec4 {
    fn mul(&self, other: &Vec4) -> Vec4;
}

impl Mat4 {
    pub fn identity() -> Self {
        Self {
            data: [[1.0, 0.0, 0.0, 0.0],
                   [0.0, 1.0, 0.0, 0.0],
                   [0.0, 0.0, 1.0, 0.0],
                   [0.0, 0.0, 0.0, 1.0]]
        }
    }

    pub fn ortho(size: winit::dpi::PhysicalSize<u32>) -> Self {
        let w: f32 = size.width as f32;
        let h: f32 = size.height as f32;
        Self {
            data: [[ 1.0 / w * 2.0,  0.0,            0.0,    0.0],
                   [ 0.0,           -1.0 / h * 2.0,  0.0,    0.0],
                   [ 0.0,            0.0,            1.0,    0.0],
                   [-1.0,            1.0,            0.0,    1.0]]
        }
    }

    pub fn box2d(pos: Vec2, size: Vec2) -> Self {
        return Self::translate2d(pos) * Self::scale2d(size);
    }

    pub fn box2d_rot(pos: Vec2, size: Vec2, rotation: f32) -> Self {
        return Self::translate2d(pos) * Self::rotate2d(rotation) * Self::scale2d(size);
    }

    pub fn scale(scale: f32) -> Self {
        let sca = scale;
        Self {
            data: [[sca, 0.0, 0.0, 0.0],
                   [0.0, sca, 0.0, 0.0],
                   [0.0, 0.0, sca, 0.0],
                   [0.0, 0.0, 0.0, 1.0]]
        }
    }

    pub fn scale2d(scale: Vec2) -> Self {
        let s = scale;
        Self {
            data: [[s.x, 0.0, 0.0, 0.0],
                   [0.0, s.y, 0.0, 0.0],
                   [0.0, 0.0, 1.0, 0.0],
                   [0.0, 0.0, 0.0, 1.0]]
        }
    }

    pub fn translate2d(movement: Vec2) -> Self {
        let v = movement;
        Self {
            data: [[1.0, 0.0, 0.0, 0.0],
                   [0.0, 1.0, 0.0, 0.0],
                   [0.0, 0.0, 1.0, 0.0],
                   [v.x, v.y, 0.0, 1.0]]
        }
    }

    pub fn translate3d(movement: Vec3) -> Self {
        let v = movement;
        Self {
            data: [[1.0, 0.0, 0.0, 0.0],
                   [0.0, 1.0, 0.0, 0.0],
                   [0.0, 0.0, 1.0, 0.0],
                   [v.x, v.y, v.z, 1.0]]
        }
    }

    pub fn rotate2d(angle: f32) -> Self {
        let c_t = f32::cos(angle);
        let s_t = f32::sin(angle);
        Self {
            data: [[c_t,-s_t, 0.0, 0.0],
                   [s_t, c_t, 0.0, 0.0],
                   [0.0, 0.0, 1.0, 0.0],
                   [0.0, 0.0, 0.0, 1.0]]
        }
    }
}

impl std::ops::Mul for &Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Self) -> Self::Output {
        let a = &self.data;
        let b = &rhs.data;
        return Mat4 {
            data: [[a[0][0] * b[0][0] + a[1][0] * b[0][1] + a[2][0] * b[0][2] + a[3][0] * b[0][3],
                    a[0][1] * b[0][0] + a[1][1] * b[0][1] + a[2][1] * b[0][2] + a[3][1] * b[0][3],
                    a[0][2] * b[0][0] + a[1][2] * b[0][1] + a[2][2] * b[0][2] + a[3][2] * b[0][3],
                    a[0][3] * b[0][0] + a[1][3] * b[0][1] + a[2][3] * b[0][2] + a[3][3] * b[0][3]],

                   [a[0][0] * b[1][0] + a[1][0] * b[1][1] + a[2][0] * b[1][2] + a[3][0] * b[1][3],
                    a[0][1] * b[1][0] + a[1][1] * b[1][1] + a[2][1] * b[1][2] + a[3][1] * b[1][3],
                    a[0][2] * b[1][0] + a[1][2] * b[1][1] + a[2][2] * b[1][2] + a[3][2] * b[1][3],
                    a[0][3] * b[1][0] + a[1][3] * b[1][1] + a[2][3] * b[1][2] + a[3][3] * b[1][3]],

                   [a[0][0] * b[2][0] + a[1][0] * b[2][1] + a[2][0] * b[2][2] + a[3][0] * b[2][3],
                    a[0][1] * b[2][0] + a[1][1] * b[2][1] + a[2][1] * b[2][2] + a[3][1] * b[2][3],
                    a[0][2] * b[2][0] + a[1][2] * b[2][1] + a[2][2] * b[2][2] + a[3][2] * b[2][3],
                    a[0][3] * b[2][0] + a[1][3] * b[2][1] + a[2][3] * b[2][2] + a[3][3] * b[2][3]],

                   [a[0][0] * b[3][0] + a[1][0] * b[3][1] + a[2][0] * b[3][2] + a[3][0] * b[3][3],
                    a[0][1] * b[3][0] + a[1][1] * b[3][1] + a[2][1] * b[3][2] + a[3][1] * b[3][3],
                    a[0][2] * b[3][0] + a[1][2] * b[3][1] + a[2][2] * b[3][2] + a[3][2] * b[3][3],
                    a[0][3] * b[3][0] + a[1][3] * b[3][1] + a[2][3] * b[3][2] + a[3][3] * b[3][3]],
            ]
        }
    }
}
impl std::ops::Mul for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Self) -> Self::Output {
        &self * &rhs
    }
}

impl std::ops::Rem<&Vec4> for &Mat4 {
    type Output = Vec4;

    fn rem(self, other: &Vec4) -> Vec4 {
        let a = &self.data;
        let b = &[other.data];
        return vec4(a[0][0] * b[0][0] + a[1][0] * b[0][1] + a[2][0] * b[0][2] + a[3][0] * b[0][3],
                    a[0][1] * b[0][0] + a[1][1] * b[0][1] + a[2][1] * b[0][2] + a[3][1] * b[0][3],
                    a[0][2] * b[0][0] + a[1][2] * b[0][1] + a[2][2] * b[0][2] + a[3][2] * b[0][3],
                    a[0][3] * b[0][0] + a[1][3] * b[0][1] + a[2][3] * b[0][2] + a[3][3] * b[0][3]);
    }
}
impl std::ops::Rem<Vec4> for Mat4 {
    type Output = Vec4;

    fn rem(self, rhs: Vec4) -> Vec4 {
        &self % &rhs
    }
}

impl Vec4 {
    pub fn zero() -> Vec4 {
        Self {
            data: [0.0, 0.0, 0.0, 0.0],
        }
    }
    pub fn identity() -> Vec4 {
        Self {
            data: [1.0, 1.0, 1.0, 1.0], 
        }
    }
}

pub fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2 {
        data: [x, y],
    }
}
pub fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3 {
        data: [x, y, z],
    }
}
pub fn vec3a(v: Vec2, z: f32) -> Vec3 {
    Vec3 {
        data: [v.x, v.y, z],
    }
}
pub fn vec3b(x: f32, v: Vec2) -> Vec3 {
    Vec3 {
        data: [x, v.x, v.y],
    }
}
pub fn vec4(x: f32, y: f32, z: f32, w: f32) -> Vec4 {
    Vec4 {
        data: [x, y, z, w],
    }
}
pub fn vec4a(v: Vec2, z: f32, w: f32) -> Vec4 {
    Vec4 {
        data: [v.x, v.y, z, w],
    }
}
pub fn vec4b(x: f32, v: Vec2, w: f32) -> Vec4 {
    Vec4 {
        data: [x, v.x, v.y, w],
    }
}
pub fn vec4c(x: f32, y: f32, v: Vec2) -> Vec4 {
    Vec4 {
        data: [x, y, v.x, v.y],
    }
}
pub fn vec4d(a: Vec2, b: Vec2) -> Vec4 {
    Vec4 {
        data: [a.x, a.y, b.x, b.y],
    }
}
pub fn vec4e(v: Vec3, d: f32) -> Vec4 {
    Vec4 {
        data: [v.x, v.y, v.z, d],
    }
}

impl std::ops::Deref for Vec2 {
    type Target = DVec2;

    fn deref(&self) -> &Self::Target {
        return bytemuck::cast_ref(&self.data);
    }
}
impl std::ops::DerefMut for Vec2 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return bytemuck::cast_mut(&mut self.data);
    }
}

impl std::ops::Deref for Vec3 {
    type Target = DVec3;

    fn deref(&self) -> &Self::Target {
        return bytemuck::cast_ref(&self.data);
    }
}
impl std::ops::DerefMut for Vec3 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return bytemuck::cast_mut(&mut self.data);
    }
}

impl std::ops::Deref for Vec4 {
    type Target = DVec4;

    fn deref(&self) -> &Self::Target {
        return bytemuck::cast_ref(&self.data);
    }
}
impl std::ops::DerefMut for Vec4 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return bytemuck::cast_mut(&mut self.data);
    }
}

impl AsRef<[u8]> for Mat4 {
    fn as_ref(&self) -> &[u8] {
        let m: &[f32; 16] = bytemuck::must_cast_ref(self);
        let m: &[u8; 64] = bytemuck::must_cast_ref(m);
        return m;
    }
}
impl<'a> Into<&'a [u8; 64]> for &'a Mat4 {
    fn into(self) -> &'a [u8; 64] {
        let m: &'a [f32; 16] = bytemuck::must_cast_ref(self);
        return bytemuck::must_cast_ref(m);
    }
}
impl<'a> Into<&'a [u8]> for &'a Mat4 {
    fn into(self) -> &'a [u8] {
        let m: &'a [f32; 16] = bytemuck::must_cast_ref(self);
        let s: &'a [u8; 64] = bytemuck::must_cast_ref(m);
        return s;
    }
}
impl Into<[u8; 64]> for Mat4 {
    fn into(self) -> [u8; 64] {
        let m: [f32; 16] = bytemuck::must_cast(self);
        let s: [u8; 64] = bytemuck::must_cast(m);
        return s;
    }
}

// pub trait PartialOrdMinMax<P: PartialOrd> {
//     fn partial_max(self) -> Option<P>;
//     fn partial_min(self) -> Option<P>;
// }
// 
// impl<T, P> PartialOrdMinMax<P> for T
//             where P: PartialOrd,
//                   T: Iterator<Item = P> {
//     fn partial_max(self) -> Option<P> {
//         use std::cmp::Ordering::*;
//         self.fold(None, |cur, next| {
//             match cur {
//                 None => Some(next),
//                 Some(cur) => match cur.partial_cmp(&next) {
//                     None => None,
//                     Some(Less) => Some(next),
//                     Some(Equal) | Some(Greater) => Some(cur),
//                 },
//             }
//         })
//     }
// 
//     fn partial_min(self) -> Option<P> {
//         use std::cmp::Ordering::*;
//         self.fold(None, |cur, next| {
//             match cur {
//                 None => Some(next),
//                 Some(cur) => match cur.partial_cmp(&next) {
//                     None => None,
//                     Some(Greater) => Some(next),
//                     Some(Equal) | Some(Less) => Some(cur),
//                 },
//             }
//         })
//     }
// }
// 
// pub fn char_bytes(c: char) -> String {
//     let mut b = [0u8; 4];
//     let len = c.encode_utf8(&mut b).len();
//     let mut s = String::new();
//     for byte in b.iter().take(len) {
//         s += &(byte.to_string() + " ");
//     }
//     s
// }
// 
// pub trait WordSpaceIterable<'a> {
//     fn words_spaces(self) -> WordSpaceIterator<'a>;
// }
// impl<'a> WordSpaceIterable<'a> for std::str::Chars<'a> {
//     fn words_spaces(self) -> WordSpaceIterator<'a> {
//         WordSpaceIterator {
//             chars: self,
//             current: None,
//         }
//     }
// }
// 
// pub struct WordSpaceIterator<'a> {
//     chars: std::str::Chars<'a>,
//     current: Option<char>
// }
// 
// pub fn is_whitespace(c: char) -> bool {
//     c == ' ' || c == '\t' || c == '\n'
// }
// 
// impl <'a> Iterator for WordSpaceIterator<'a> {
//     type Item = String;
//     fn next(&mut self) -> Option<Self::Item> {
//         let start = {
//             if let Some(current) = self.current {
//                 current
//             } else if let Some(next) = self.chars.next() {
//                 next
//             } else {
//                 self.current = None;
//                 return None
//             }
//         };
//         self.current = None;
//         let whitespace = is_whitespace(start);
//         let mut word = start.to_string();
//         for c in &mut self.chars {
//             if is_whitespace(c) == whitespace {
//                 word.push(c);
//             } else {
//                 // preserve the unused character (reason why we can't use take_while)
//                 self.current = Some(c);
//                 break;
//             }
//         }
// 
//         Some(word)
//     }
// }
// 
// pub fn clampi(val: i32, start: i32, end: i32) -> i32 {
//     if end < start {
//         return start
//     }
//     if val < start {
//         start
//     } else if val > end {
//         end
//     } else {
//         val
//     }
// }
// 
// pub struct BoundingBox {
//     pub center: Vector2<f32>,
//     pub scale: Vector2<f32>,
// }
// 
// impl BoundingBox {
//     pub fn new(center: Vector2<f32>, scale: Vector2<f32>) -> Self {
//         Self { center, scale }
//     }
//     pub fn contains(&self, point: &Vector2<f32>) -> bool {
//         self.center.x - self.scale.x / 2.0 <= point.x && point.x <= self.center.x + self.scale.x / 2.0 &&
//         self.center.y - self.scale.y / 2.0 <= point.y && point.y <= self.center.y + self.scale.y / 2.0
//     }
// }
// 
// pub fn world_to_world_screen_space(v: Vector3<f32>) -> Vector2<f32> {
//     vec2(v.x, v.y - v.z)
// }
// 
// pub trait Depth {
//     fn depth(&self) -> f32;
// }
// 
// // merges two sorted lists into a bigger sorted list
// pub fn merge_depth<T: Depth>(mut av: Vec<T>, mut bv: Vec<T>) -> Vec<T> {
//     if av.is_empty() {
//         return bv;
//     } else if bv.is_empty() {
//         return av;
//     }
//     let mut res = vec![];
//     let mut ai = av.drain(0..av.len()).peekable();
//     let mut bi = bv.drain(0..bv.len()).peekable();
//     while let (Some(a), Some(b)) = (ai.peek(), bi.peek()) {
//         if a.depth() < b.depth() {
//             res.push(ai.next().unwrap());
//         } else if a.depth() > b.depth() {
//             res.push(bi.next().unwrap());
//         } else {
//             res.push(ai.next().unwrap());
//             res.push(bi.next().unwrap());
//         }
//     }
//     res.extend(ai);
//     res.extend(bi);
//     res
// }
