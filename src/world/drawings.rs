use std::{
    borrow::Borrow,
    io::{Stdout, Write},
    thread,
    time::Duration,
};

use crossterm::{
    event::{poll, read},
    style::Stylize,
};

use crate::{
    drawable::Drawable,
    entities::{DeathCause, PlayerStatus},
    game::Game,
    stout_ext::StdoutExt,
    World,
};

pub struct NotificationDrawing {
    max_c: u16,
    max_l: u16,
    message: String,
}

impl NotificationDrawing {
    pub fn new(max_c: u16, max_l: u16, message: String) -> Self {
        Self {
            max_c,
            max_l,
            message,
        }
    }
}

impl Drawable for NotificationDrawing {
    fn draw(&self, sc: &mut crate::canvas::Canvas) {
        let message_len = self.message.len();
        let line_1 = format!("╔═{}═╗", "═".repeat(message_len));
        let line_2 = format!("║ {} ║", self.message);
        let line_3 = format!("╚═{}═╝", "═".repeat(message_len));

        let message_len_offset = (message_len / 2) as u16 + 2;
        sc.draw_line(
            (self.max_c / 2 - message_len_offset, self.max_l / 2 - 1),
            line_1,
        )
        .draw_line(
            (self.max_c / 2 - message_len_offset, self.max_l / 2),
            line_2,
        )
        .draw_line(
            (self.max_c / 2 - message_len_offset, self.max_l / 2 + 1),
            line_3,
        );
    }
}

impl<'g> World<'g> {
    pub fn notification(&self, message: impl Into<String>) -> NotificationDrawing {
        NotificationDrawing::new(self.maxc, self.maxl, message.into())
    }

    pub fn draw_on_canvas(&mut self) {
        self.canvas.clear_all();

        // draw the map
        self.canvas.draw(&self.map);

        self.canvas
            .draw_line(2, format!(" Score: {} ", self.player.score))
            .draw_line((2, 3), format!(" Fuel: {} ", self.player.gas / 100))
            .draw_line((2, 4), format!(" Enemies: {} ", self.enemies.len()))
            .draw_line((2, 5), format!(" Traveled: {} ", self.player.traveled))
            // .draw_line((2, 6), format!(" (dbg) Events: {} ", self.events.len()))
            .draw_line(
                (2, 6),
                format!(" (dbg) Timers: {} ", self.timers.borrow().len()),
            );

        // draw fuel
        for fuel in self.fuels.iter() {
            self.canvas.draw(fuel);
        }

        // draw enemies
        for enemy in self.enemies.iter() {
            self.canvas.draw(enemy);
        }

        // draw bullet
        for bullet in &self.bullets {
            self.canvas.draw(bullet);
        }

        // draw the player
        self.canvas.draw(&self.player);

        for (_, drawing) in self.custom_drawings.iter() {
            let drawing: &dyn Drawable = drawing.borrow();
            drawing.draw(&mut self.canvas);
        }
    }

    pub fn pause_screen(&mut self) {
        self.canvas.draw(&self.notification("Game Paused!"));
    }
}

impl<'g> Game<'g> {
    pub fn clear_screen<'a>(
        &'a self,
        stdout: &'a mut Stdout,
    ) -> Result<&mut Stdout, std::io::Error> {
        stdout.clear_all()
    }

    pub fn welcome_screen(&self, stdout: &mut Stdout) -> Result<(), std::io::Error> {
        let world = &self.world.borrow();

        let welcome_msg: &str = "██████╗ ██╗██╗   ██╗███████╗██████╗ ██████╗  █████╗ ██╗██████╗     ██████╗ ██╗   ██╗███████╗████████╗\n\r██╔══██╗██║██║   ██║██╔════╝██╔══██╗██╔══██╗██╔══██╗██║██╔══██╗    ██╔══██╗██║   ██║██╔════╝╚══██╔══╝\n\r██████╔╝██║██║   ██║█████╗  ██████╔╝██████╔╝███████║██║██║  ██║    ██████╔╝██║   ██║███████╗   ██║   \n\r██╔══██╗██║╚██╗ ██╔╝██╔══╝  ██╔══██╗██╔══██╗██╔══██║██║██║  ██║    ██╔══██╗██║   ██║╚════██║   ██║   \n\r██║  ██║██║ ╚████╔╝ ███████╗██║  ██║██║  ██║██║  ██║██║██████╔╝    ██║  ██║╚██████╔╝███████║   ██║   \n\r╚═╝  ╚═╝╚═╝  ╚═══╝  ╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝╚═════╝     ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝   \n";
        self.clear_screen(stdout)?;

        if world.maxc > 100 {
            stdout.draw((0, 2), welcome_msg)?;
        } else {
            stdout.draw((0, 2), "RiverRaid Rust")?;
        }

        stdout.draw((2, world.maxl - 2), "Press any key to continue...")?;
        stdout.flush()?;

        loop {
            if poll(Duration::from_millis(0)).unwrap() {
                read()?;
                break;
            }
        }
        self.clear_screen(stdout)?;

        Ok(())
    }

    pub fn goodbye_screen(&self, stdout: &mut Stdout) -> Result<(), std::io::Error> {
        let world = &self.world.borrow();

        let goodbye_msg1: &str = " ██████╗  ██████╗  ██████╗ ██████╗      ██████╗  █████╗ ███╗   ███╗███████╗██╗\n\r██╔════╝ ██╔═══██╗██╔═══██╗██╔══██╗    ██╔════╝ ██╔══██╗████╗ ████║██╔════╝██║\n\r██║  ███╗██║   ██║██║   ██║██║  ██║    ██║  ███╗███████║██╔████╔██║█████╗  ██║\n\r██║   ██║██║   ██║██║   ██║██║  ██║    ██║   ██║██╔══██║██║╚██╔╝██║██╔══╝  ╚═╝\n\r╚██████╔╝╚██████╔╝╚██████╔╝██████╔╝    ╚██████╔╝██║  ██║██║ ╚═╝ ██║███████╗██╗\n\r ╚═════╝  ╚═════╝  ╚═════╝ ╚═════╝      ╚═════╝ ╚═╝  ╚═╝╚═╝     ╚═╝╚══════╝╚═╝\n";
        let goodbye_msg2: &str = "████████╗██╗  ██╗ █████╗ ███╗   ██╗██╗  ██╗███████╗\n\r╚══██╔══╝██║  ██║██╔══██╗████╗  ██║██║ ██╔╝██╔════╝\n\r   ██║   ███████║███████║██╔██╗ ██║█████╔╝ ███████╗\n\r   ██║   ██╔══██║██╔══██║██║╚██╗██║██╔═██╗ ╚════██║\n\r   ██║   ██║  ██║██║  ██║██║ ╚████║██║  ██╗███████║██╗\n\r   ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝╚══════╝╚═╝\n";

        self.clear_screen(stdout)?
            .draw((0, 2), goodbye_msg1)?
            .draw((0, 10), goodbye_msg2)?;

        stdout.move_cursor((2, world.maxl - 5))?;
        if let PlayerStatus::Dead(cause) = &world.player.status {
            match cause {
                DeathCause::Ground => {
                    if world.maxc > 91 {
                        stdout.print("\r█▄█ █▀█ █░█   █▀▀ █▀█ ▄▀█ █▀ █░█ █▀▀ █▀▄   █ █▄░█   ▀█▀ █░█ █▀▀   █▀▀ █▀█ █▀█ █░█ █▄░█ █▀▄ ░\n\r░█░ █▄█ █▄█   █▄▄ █▀▄ █▀█ ▄█ █▀█ ██▄ █▄▀   █ █░▀█   ░█░ █▀█ ██▄   █▄█ █▀▄ █▄█ █▄█ █░▀█ █▄▀ ▄\n\r")?;
                    } else {
                        stdout.print("You crashed in the ground.")?;
                    }
                }
                DeathCause::Enemy => {
                    if world.maxc > 72 {
                        stdout.print("\r▄▀█ █▄░█   █▀▀ █▄░█ █▀▀ █▀▄▀█ █▄█   █▄▀ █ █░░ █░░ █▀▀ █▀▄   █▄█ █▀█ █░█ ░\n\r█▀█ █░▀█   ██▄ █░▀█ ██▄ █░▀░█ ░█░   █░█ █ █▄▄ █▄▄ ██▄ █▄▀   ░█░ █▄█ █▄█ ▄\n\r")?;
                    } else {
                        stdout.print("An enemy killed you.")?;
                    }
                }
                DeathCause::Fuel => {
                    if world.maxc > 69 {
                        stdout.print("\r█▄█ █▀█ █░█   █▀█ ▄▀█ █▄░█   █▀█ █░█ ▀█▀   █▀█ █▀▀   █▀▀ █░█ █▀▀ █░░ ░\n\r░█░ █▄█ █▄█   █▀▄ █▀█ █░▀█   █▄█ █▄█ ░█░   █▄█ █▀░   █▀░ █▄█ ██▄ █▄▄ ▄\n\r")?;
                    } else {
                        stdout.print("You ran out of fuel.")?;
                    }
                }
            }
        } else {
            // Quit
            if world.player.status != PlayerStatus::Quit {
                unreachable!("Undead player has no death cause!")
            }
        }

        stdout.move_cursor((2, world.maxl - 2))?;
        thread::sleep(Duration::from_millis(2000));
        stdout.print("Press any key to continue...")?;
        stdout.flush()?;
        loop {
            if poll(Duration::from_millis(0)).unwrap() {
                read()?;
                break;
            }
        }

        self.clear_screen(stdout)?;
        Ok(())
    }
}
