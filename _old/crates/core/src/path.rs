#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathPoint {
    pub x: f64,
    pub y: f64,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub id: u32,
    pub points: Vec<PathPoint>,
}

impl Path {
    pub fn new(id: u32, points: Vec<PathPoint>) -> Self {
        Self { id, points }
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    pub fn circle(id: u32, cx: f64, cy: f64, radius: f64, samples: usize) -> Self {
        let mut pts = Vec::with_capacity(samples);
        for i in 0..samples {
            let t = (i as f64) / (samples as f64) * std::f64::consts::TAU;
            pts.push(PathPoint {
                x: cx + radius * t.cos(),
                y: cy + radius * t.sin(),
            });
        }
        Self { id, points: pts }
    }

    pub fn resample(&self, new_len: usize) -> Self {
        if self.points.is_empty() || new_len == 0 {
            return Self { id: self.id, points: Vec::new() };
        }
        if self.points.len() == 1 {
            return Self { id: self.id, points: vec![self.points[0]; new_len] };
        }
        let mut pts = Vec::with_capacity(new_len);
        for i in 0..new_len {
            let t = (i as f64) * ((self.points.len() - 1) as f64) / ((new_len - 1) as f64);
            let idx = t.floor() as usize;
            let frac = t - (idx as f64);
            if idx + 1 >= self.points.len() {
                pts.push(self.points[self.points.len() - 1]);
            } else {
                let a = self.points[idx];
                let b = self.points[idx + 1];
                pts.push(PathPoint {
                    x: a.x + (b.x - a.x) * frac,
                    y: a.y + (b.y - a.y) * frac,
                });
            }
        }
        Self { id: self.id, points: pts }
    }
}