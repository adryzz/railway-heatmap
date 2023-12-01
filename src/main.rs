use std::sync::atomic::AtomicUsize;

use geo::{HaversineDistance, Point, Rect};
use rayon::prelude::*;

use crate::stations::STATIONS;

mod stations;
fn main() {
    let topleft = Point::new(47.8, 6.0);
    let bottomright = Point::new(36.0, 19.0);

    let rect = Rect::new(topleft, bottomright);

    let step = 0.01; // degrees

    let value_scale = 1.0;

    let iterator = RectGridIterator::new(rect, step);

    let size = iterator.image_size();
    let total = size.0 * size.1;

    let mut vec = vec![0u32; total];

    let count = AtomicUsize::new(0);

    vec.par_iter_mut().enumerate().for_each(|(i, val)| {
        if let Some(p) = iterator.get_coord_at_index(i) {
            let c = count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let perc = (c as f64 / total as f64) * 100.0;
            if (c & 2047) == 0 {
                println!("{}/{}, {:.3}%", c + 1, total, perc);
            }
            let distance = find_closest_point(&p, &STATIONS);

            let pre_val = (distance.1 * value_scale) as u16;
            let value = fast_heatmap::get_color(u16::MAX - pre_val);

            *val = value;
        }
    });

    image::save_buffer(
        "image.png",
        convert(&vec[..]),
        size.0 as u32,
        size.1 as u32,
        image::ColorType::Rgba8,
    )
    .unwrap();
}

pub fn convert<'a>(data: &'a [u32]) -> &'a [u8] {
    unsafe { &mut std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 4) }
}

#[derive(Debug, Clone, Copy)]
struct RectGridIterator {
    inner: Rect,
    step: f64,
    next: usize,
    size: (usize, usize),
}

impl RectGridIterator {
    pub fn new(rect: Rect, step: f64) -> Self {
        let mut x_c = rect.min().x;
        let mut y_c = rect.min().y;
        let mut x = 0;
        let mut y = 0;

        while x_c <= rect.max().x {
            x += 1;
            x_c += step;
        }

        while y_c <= rect.max().y {
            y += 1;
            y_c += step;
        }

        Self {
            inner: rect,
            step,
            next: 0,
            size: (x, y),
        }
    }

    pub fn image_size(&self) -> (usize, usize) {
        self.size
    }

    pub fn get_coord_at_index(&self, index: usize) -> Option<Point> {
        if index >= (self.size.0 * self.size.1) {
            return None;
        }

        let y = index / self.size.0;

        let x = index % self.size.0;

        let x_c = self.inner.min().x;
        let y_c = self.inner.min().y;

        let x_o = self.step * x as f64;

        let y_o = self.step * y as f64;

        Some(Point::new(x_c + x_o, y_c + y_o))
    }
}

impl Iterator for RectGridIterator {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.get_coord_at_index(self.next);

        self.next += 1;

        ret
    }
}

fn find_closest_point(target: &Point, points: &[(f64, f64)]) -> (usize, f64) {
    let mut min_distance = f64::MAX;
    let mut closest_index = 0;

    for (index, point) in points.iter().enumerate() {
        let distance = target.haversine_distance(&Point::new(point.0, point.1));
        if distance < min_distance {
            min_distance = distance;
            closest_index = index;
        }
    }

    (closest_index, min_distance)
}
