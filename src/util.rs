use cgmath::{Vector2, Vector3, vec2, Array};

pub trait PartialOrdMinMax<P: PartialOrd> {
    fn partial_max(self) -> Option<P>;
    fn partial_min(self) -> Option<P>;
}

impl<T, P> PartialOrdMinMax<P> for T
            where P: PartialOrd,
                  T: Iterator<Item = P> {
    fn partial_max(self) -> Option<P> {
        use std::cmp::Ordering::*;
        self.fold(None, |cur, next| {
            match cur {
                None => Some(next),
                Some(cur) => match cur.partial_cmp(&next) {
                    None => None,
                    Some(Less) => Some(next),
                    Some(Equal) | Some(Greater) => Some(cur),
                },
            }
        })
    }

    fn partial_min(self) -> Option<P> {
        use std::cmp::Ordering::*;
        self.fold(None, |cur, next| {
            match cur {
                None => Some(next),
                Some(cur) => match cur.partial_cmp(&next) {
                    None => None,
                    Some(Greater) => Some(next),
                    Some(Equal) | Some(Less) => Some(cur),
                },
            }
        })
    }
}

pub fn char_bytes(c: char) -> String {
    let mut b = [0u8; 4];
    let len = c.encode_utf8(&mut b).len();
    let mut s = String::new();
    for byte in b.iter().take(len) {
        s += &(byte.to_string() + " ");
    }
    s
}

pub trait WordSpaceIterable<'a> {
    fn words_spaces(self) -> WordSpaceIterator<'a>;
}
impl<'a> WordSpaceIterable<'a> for std::str::Chars<'a> {
    fn words_spaces(self) -> WordSpaceIterator<'a> {
        WordSpaceIterator {
            chars: self,
            current: None,
        }
    }
}

pub struct WordSpaceIterator<'a> {
    chars: std::str::Chars<'a>,
    current: Option<char>
}

pub fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\t' || c == '\n'
}

impl <'a> Iterator for WordSpaceIterator<'a> {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        let start = {
            if let Some(current) = self.current {
                current
            } else if let Some(next) = self.chars.next() {
                next
            } else {
                self.current = None;
                return None
            }
        };
        self.current = None;
        let whitespace = is_whitespace(start);
        let mut word = start.to_string();
        for c in &mut self.chars {
            if is_whitespace(c) == whitespace {
                word.push(c);
            } else {
                // preserve the unused character (reason why we can't use take_while)
                self.current = Some(c);
                break;
            }
        }

        Some(word)
    }
}

pub fn clampi(val: i32, start: i32, end: i32) -> i32 {
    if end < start {
        return start
    }
    if val < start {
        start
    } else if val > end {
        end
    } else {
        val
    }
}

pub struct BoundingBox {
    pub center: Vector2<f32>,
    pub scale: Vector2<f32>,
}

impl BoundingBox {
    pub fn new(center: Vector2<f32>, scale: Vector2<f32>) -> Self {
        Self { center, scale }
    }
    pub fn contains(&self, point: &Vector2<f32>) -> bool {
        self.center.x - self.scale.x / 2.0 <= point.x && point.x <= self.center.x + self.scale.x / 2.0 &&
        self.center.y - self.scale.y / 2.0 <= point.y && point.y <= self.center.y + self.scale.y / 2.0
    }
}

pub fn world_to_world_screen_space(v: Vector3<f32>) -> Vector2<f32> {
    vec2(v.x, v.y - v.z)
}

pub trait Depth {
    fn depth(&self) -> f32;
}

// merges two sorted lists into a bigger sorted list
pub fn merge_depth<T: Depth>(mut av: Vec<T>, mut bv: Vec<T>) -> Vec<T> {
    if av.is_empty() {
        return bv;
    } else if bv.is_empty() {
        return av;
    }
    let mut res = vec![];
    let mut ai = av.drain(0..av.len()).peekable();
    let mut bi = bv.drain(0..bv.len()).peekable();
    while let (Some(a), Some(b)) = (ai.peek(), bi.peek()) {
        if a.depth() < b.depth() {
            res.push(ai.next().unwrap());
        } else if a.depth() > b.depth() {
            res.push(bi.next().unwrap());
        } else {
            res.push(ai.next().unwrap());
            res.push(bi.next().unwrap());
        }
    }
    res.extend(ai);
    res.extend(bi);
    res
}
