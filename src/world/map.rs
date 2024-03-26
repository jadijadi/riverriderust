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

    pub fn from_map(map: &Map, rng: &mut ThreadRng) -> Self {
        use Ordering::*;

        match map.river_mode {
            RiverMode::Random {
                min_width,
                max_width,
                max_center_diff,
            } => {
                let mut river = RiverPart::new(
                    rng.gen_range(min_width..max_width),
                    rng.gen_range(0..map.max_c),
                );

                // Adjust newly generated center_c to be not so far
                let front_center_c = map.front().unwrap().center_c;
                if river.center_c.abs_diff(front_center_c) > max_center_diff {
                    river.center_c = match river.center_c.cmp(&front_center_c) {
                        Less => front_center_c - max_center_diff,
                        Greater => front_center_c + max_center_diff,
                        _ => unreachable!(),
                    }
                }

                river
            }
            RiverMode::ConstWidth {
                width,
                max_center_diff,
            } => {
                let mut river = RiverPart::new(width, rng.gen_range(0..map.max_c));

                // Adjust newly generated center_c to be not so far
                let front_center_c = map.front().unwrap().center_c;
                if river.center_c.abs_diff(front_center_c) > max_center_diff {
                    river.center_c = match river.center_c.cmp(&front_center_c) {
                        Less => front_center_c - max_center_diff,
                        Greater => front_center_c + max_center_diff,
                        _ => unreachable!(),
                    }
                }

                river
            }
            RiverMode::ConstCenter {
                center_c,
                min_width,
                max_width,
            } => RiverPart::new(rng.gen_range(min_width..max_width), center_c),
            RiverMode::ConstWidthAndCenter { width, center_c } => RiverPart::new(width, center_c),
        }
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum RiverMode {
    Random {
        min_width: u16,
        max_width: u16,
        max_center_diff: u16,
    },
    ConstWidth {
        width: u16,
        max_center_diff: u16,
    },
    ConstCenter {
        center_c: u16,
        min_width: u16,
        max_width: u16,
    },
    ConstWidthAndCenter {
        width: u16,
        center_c: u16,
    },
}

pub struct Map {
    max_c: u16,
    max_l: u16,
    river_mode: RiverMode,
    river_mode_default: RiverMode,
    river_parts: VecDeque<RiverPart>,
    next_point: u16,
    change_rate: u16,
    target_river: RiverPart,
}

impl Drawable for Map {
    fn draw(&self, sc: &mut crate::canvas::Canvas) {
        for (line, part) in self.river_parts.iter().enumerate() {
            let border_range = self.river_borders(part);
            let (left_b, right_b) = (border_range.start, border_range.end);

            let line: u16 = line as u16;
            sc.draw_line((0, line), "+".repeat(left_b.into()))
                .draw_line((right_b, line), "+".repeat((self.max_c - right_b) as usize));
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
        let river_mode = RiverMode::Random {
            min_width,
            max_width,
            max_center_diff,
        };
        Self {
            max_c,
            max_l,
            next_point: max_l,
            river_parts: (0..max_l)
                .map(|_| RiverPart::new(max_width, max_c / 2))
                .collect(),
            change_rate,
            river_mode: river_mode.clone(),
            river_mode_default: river_mode,
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
        RiverPart::from_map(self, rng)
    }

    pub fn river_borders_index(&self, line: usize) -> std::ops::Range<u16> {
        self.river_borders(&self.river_parts[line])
    }

    pub fn river_borders(&self, river: &RiverPart) -> std::ops::Range<u16> {
        let offset = river.width / 2;
        let left_border = river.center_c.checked_sub(offset).unwrap_or(0);
        let right_border = river.center_c + offset;

        left_border..if right_border >= self.max_c {
            self.max_c
        } else {
            right_border
        }
    }

    pub fn update(&mut self, rng: &mut ThreadRng) {
        if self.next_point <= self.change_rate {
            self.target_river = self.generate_new_target(rng);
            self.next_point = self.max_l;
        }

        self.river_parts.pop_back();
        self.river_parts.push_front(self.decide_next());
        self.next_point = self.next_point.checked_sub(self.change_rate).unwrap_or(0);
    }

    pub fn change_river_mode(&mut self, mode: RiverMode) {
        self.river_mode = mode;
    }

    pub fn restore_river_mode(&mut self) {
        self.river_mode = self.river_mode_default.clone();
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
