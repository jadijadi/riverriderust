use crossterm::style::{ContentStyle, Stylize};

use crate::{
    canvas::Canvas,
    entities::{Bullet, Entity, EntityStatus, Player},
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
        match self.entity_type {
            crate::entities::EntityType::Enemy(_) => {
                match self.status {
                    EntityStatus::Alive => {
                        sc.draw_styled_char(self, '⍢', ContentStyle::new().red());
                    }
                    EntityStatus::DeadBody => {
                        sc.draw_styled(self, '✘'.yellow());
                    }
                    EntityStatus::Dead => {}
                };
            }
            crate::entities::EntityType::Fuel(_) => match self.status {
                EntityStatus::Alive => {
                    sc.draw_styled_char(self, '✚', ContentStyle::new().green());
                }
                EntityStatus::DeadBody => {
                    sc.draw_styled(self, '$'.yellow());
                }
                EntityStatus::Dead => {}
            },
        };
    }
}

impl Drawable for Bullet {
    fn draw_on_canvas(&self, sc: &mut Canvas) {
        sc.draw_styled_char(self, '▴', ContentStyle::new().cyan());
    }
}

impl Drawable for Player {
    fn draw_on_canvas(&self, sc: &mut Canvas) {
        sc.draw_char(self, '▲');

        for bullet in self.bullets.iter() {
            sc.draw(bullet);
        }
    }
}
