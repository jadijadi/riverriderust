use crate::{
    canvas::Canvas,
    entities::{Bullet, Enemy, EntityStatus, Fuel, Player},
};

pub trait Drawable {
    fn draw(&self, sc: &mut Canvas);
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
    fn draw(&self, sc: &mut Canvas) {
        (self.drawer)(sc)
    }
}

impl Drawable for Enemy {
    fn draw(&self, sc: &mut Canvas) {
        match self.status {
            EntityStatus::Alive => {
                sc.draw_char(self, 'E');
            }
            EntityStatus::DeadBody => {
                sc.draw_char(self, 'X');
            }
            EntityStatus::Dead => {}
        };
    }
}

impl Drawable for Fuel {
    fn draw(&self, sc: &mut Canvas) {
        match self.status {
            EntityStatus::Alive => {
                sc.draw_char(self, 'F');
            }
            EntityStatus::DeadBody => {
                sc.draw_char(self, '$');
            }
            EntityStatus::Dead => {}
        };
    }
}

impl Drawable for Bullet {
    fn draw(&self, sc: &mut Canvas) {
        sc.draw_char(self, '|')
            .draw_char((self.location.c, self.location.l - 1), '^');
    }
}

impl Drawable for Player {
    fn draw(&self, sc: &mut Canvas) {
        sc.draw_char(self, 'P');
    }
}
