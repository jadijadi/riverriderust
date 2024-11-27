use crossterm::style::{ContentStyle, Stylize};

use crate::{
    canvas::Canvas,
    entities::{Bullet, Entity, EntityStatus, Player},
    world::map::Map,
};

pub trait Drawable {
    fn draw_on_canvas(&self, canvas: &mut Canvas);
}

pub struct CustomDrawing {
    drawer: Box<dyn Fn(&mut Canvas)>,
}

impl CustomDrawing {
    #[allow(dead_code)]
    pub fn new(drawer: impl Fn(&mut Canvas) + 'static) -> Self {
        Self {
            drawer: Box::new(drawer),
        }
    }
}

impl Drawable for CustomDrawing {
    fn draw_on_canvas(&self, sc: &mut Canvas) {
        (self.drawer)(sc)
    }
}

impl Drawable for Entity {
    fn draw_on_canvas(&self, sc: &mut Canvas) {
        match &self.entity_type {
            crate::entities::EntityType::Enemy(_) => {
                match self.status {
                    EntityStatus::Alive => {
                        sc.draw_styled_char(self, '⍢', ContentStyle::new().red().on_blue());
                    }
                    EntityStatus::DeadBody => {
                        sc.draw_styled(self, '✘'.yellow().on_blue());
                    }
                    EntityStatus::Dead => {}
                };
            }
            crate::entities::EntityType::Fuel(_) => match self.status {
                EntityStatus::Alive => {
                    sc.draw_styled_char(self, '✚', ContentStyle::new().green().on_blue());
                }
                EntityStatus::DeadBody => {
                    sc.draw_styled(self, '$'.yellow().on_blue());
                }
                EntityStatus::Dead => {}
            },
        };
    }
}

impl Drawable for Bullet {
    fn draw_on_canvas(&self, sc: &mut Canvas) {
        sc.draw_styled_char(self, '▴', ContentStyle::new().cyan().on_blue());
    }
}

impl Drawable for Player {
    fn draw_on_canvas(&self, sc: &mut Canvas) {
        sc.draw_styled(self, '▲'.on_blue());

        for bullet in self.bullets.iter() {
            sc.draw(bullet);
        }
    }
}

impl Drawable for Map {
    fn draw_on_canvas(&self, sc: &mut crate::canvas::Canvas) {
        for (line, part) in self.river_parts().iter().enumerate() {
            let border_range = self.river_borders(part);
            let (left_b, right_b) = (border_range.start, border_range.end);

            let line: u16 = line as u16;
            sc.draw_styled((0, line), " ".repeat(left_b.into()).on_green())
                .draw_styled(
                    (left_b, line),
                    " ".repeat((right_b - left_b) as usize).on_blue(),
                )
                .draw_styled(
                    (right_b, line),
                    " ".repeat((self.max_c - right_b) as usize).on_green(),
                );
        }
    }
}
