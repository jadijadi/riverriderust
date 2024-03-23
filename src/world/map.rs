use std::{cmp::Ordering, collections::VecDeque};

use rand::{rngs::ThreadRng, Rng};

use crate::drawable::Drawable;

#[derive(Clone)]
pub struct RiverPart {
    width: u16,
    center_c: u16,
}

impl RiverPart {
    pub fn new(width: u16, center_c: u16) -> Self {
        Self { width, center_c }
    }
}

pub struct Map {
    max_c: u16,
    max_l: u16,
    min_width: u16,
    max_width: u16,
    river_parts: VecDeque<RiverPart>,
    next_point: u16,
    change_rate: u16,
    max_center_diff: u16,
    target_river: RiverPart,
}

impl Drawable for Map {
    fn draw(&self, sc: &mut crate::canvas::Canvas) {
        for (line, part) in self.river_parts.iter().enumerate() {
            let (left_border, right_border) = self.river_borders(part);

            let line: u16 = line as u16;
            sc.draw_line((0, line), "+".repeat(left_border.into()))
                .draw_line(
                    (right_border, line),
                    "+".repeat((self.max_c - right_border) as usize),
                );
        }
    }
}

impl Map {
    pub fn new(
        max_c: u16,
        max_l: u16,
        min_width: u16,
        max_width: u16,
        change_rate: u16,
        max_center_diff: u16,
    ) -> Self {
        Self {
            max_c,
            max_l,
            min_width,
            max_width,
            next_point: max_l,
            river_parts: (0..max_l)
                .map(|_| RiverPart::new(max_width, max_c / 2))
                .collect(),
            change_rate,
            max_center_diff,
            target_river: RiverPart::new(max_width, max_c / 2),
        }
    }

    fn decide_next(&self) -> RiverPart {
        if let Some(river) = self.front() {
            let (current_center_c, current_width) = (river.center_c, river.width);

            let new_center_c = (current_center_c as f32)
                + (self.target_river.center_c as f32 - current_center_c as f32) * 0.1;
            let new_width = (current_width as f32)
                + (self.target_river.width as f32 - current_width as f32) * 0.1;

            RiverPart::new(new_width as u16, new_center_c as u16)
        } else {
            unreachable!("Map can't get empty.")
        }
    }

    fn generate_new_target(&self, rng: &mut ThreadRng) -> RiverPart {
        let mut river = RiverPart::new(
            rng.gen_range(self.min_width..self.max_width),
            rng.gen_range(0..self.max_c),
        );

        let front_center_c = self.front().unwrap().center_c;
        if river.center_c.abs_diff(front_center_c) > self.max_center_diff {
            river.center_c = match river.center_c.cmp(&front_center_c) {
                Ordering::Less => front_center_c - self.max_center_diff,
                Ordering::Greater => front_center_c + self.max_center_diff,
                _ => unreachable!(),
            }
        }

        river
    }

    pub fn river_borders_index(&self, line: usize) -> (u16, u16) {
        self.river_borders(&self.river_parts[line])
    }

    pub fn river_borders(&self, river: &RiverPart) -> (u16, u16) {
        let offset = river.width / 2;
        let left_border = river.center_c.checked_sub(offset).unwrap_or(0);
        let right_border = river.center_c + offset;

        (
            left_border,
            if right_border >= self.max_c {
                self.max_c
            } else {
                right_border
            },
        )
    }

    pub fn update(&mut self, rng: &mut ThreadRng) {
        if self.next_point <= self.change_rate {
            self.target_river = self.generate_new_target(rng);
            self.next_point = self.max_l;
        }

        let next = self.decide_next();
        self.river_parts.pop_back();
        self.river_parts.push_front(next);
        self.next_point = self.next_point.checked_sub(self.change_rate).unwrap_or(0);
    }

    pub fn front(&self) -> Option<&RiverPart> {
        self.river_parts.front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        let v: VecDeque<u16> = (0..10).collect();
        println!("{v:?}");
        println!("front {:?}", v.front());
        println!("back {:?}", v.back())
    }
}
