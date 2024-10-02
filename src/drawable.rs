use crossterm::style::{ContentStyle, Stylize};

use crate::{
    canvas::Canvas,
    entities::{Bullet, Enemy, EntityStatus, Fuel, Player},
};

pub trait Drawable {
    fn draw(&self, sc: &mut Canvas);
}

impl Drawable for Enemy {
    fn draw(&self, sc: &mut Canvas) {
        match self.status {
            EntityStatus::Alive => {
                sc.draw_styled_char(self, '☠', ContentStyle::new().black().on_blue());
            }
            EntityStatus::DeadBody => {
                sc.draw_styled(self, '☢'.red().on_blue());
            }
            EntityStatus::Dead => {}
        };
    }
}

impl Drawable for Fuel {
    fn draw(&self, sc: &mut Canvas) {
        match self.status {
            EntityStatus::Alive => {
                sc.draw_styled_char(self, '❤', ContentStyle::new().yellow().on_blue());
            }
            EntityStatus::DeadBody => {
                sc.draw_styled(self, '❂'.yellow().on_blue());
            }
            EntityStatus::Dead => {}
        };
    }
}

impl Drawable for Bullet {
    fn draw(&self, sc: &mut Canvas) {
        sc.draw_styled_char(self, '⇈', ContentStyle::new().cyan().on_blue())
            .draw_styled_char(
                (self.location.c, self.location.l - 1),
                '↟',
                ContentStyle::new().cyan().on_blue(),
            );
    }
}

impl Drawable for Player {
    fn draw(&self, sc: &mut Canvas) {
        sc.draw_styled(self, '▲'.white().on_blue());
    }
}
