use std::{
    borrow::Borrow,
    io::{Stdout, Write},
    thread,
    time::Duration,
};

use crossterm::{
    event::{poll, read},
    style::{ContentStyle, Stylize},
};

use crate::{
    drawable::Drawable,
    entities::{DeathCause, PlayerStatus},
    game::Game,
    stout_ext::StdoutExt,
    World,
};

pub struct PopupDrawing {
    max_c: u16,
    max_l: u16,
    message: String,
}

impl PopupDrawing {
    pub fn new(max_c: u16, max_l: u16, message: String) -> Self {
        Self {
            max_c,
            max_l,
            message,
        }
    }
}

impl Drawable for PopupDrawing {
    fn draw(&self, sc: &mut crate::canvas::Canvas) {
        let message_len = self.message.len();
        let line_0 = format!("    {}    ", " ".repeat(message_len));
        let line_1 = format!("  ╔═{}═╗  ", "═".repeat(message_len));
        let line_2 = format!("  ║ {} ║  ", self.message);
        let line_3 = format!("  ╚═{}═╝  ", "═".repeat(message_len));
        let line_4 = format!("    {}    ", " ".repeat(message_len));

        let message_len_offset = (message_len / 2) as u16 + 4;
        let col = self.max_c / 2 - message_len_offset;
        let center_l = self.max_l / 2;
        sc.draw_line((col, center_l - 2), line_0)
            .draw_line((col, center_l - 1), line_1)
            .draw_line((col, center_l), line_2)
            .draw_line((col, center_l + 1), line_3)
            .draw_line((col, center_l + 2), line_4);
    }
}

impl<'g> World<'g> {
    pub fn popup(&self, message: impl Into<String>) -> PopupDrawing {
        PopupDrawing::new(self.maxc, self.maxl, message.into())
    }

    pub fn draw_on_canvas(&mut self) {
        self.canvas.clear_all();

        // draw the map
        self.canvas.draw(&self.map);

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
        self.canvas.draw(&self.popup("Game Paused!"));
    }
}

impl<'g> Game<'g> {
    pub fn clear_screen<'a>(
        &'a self,
        stdout: &'a mut Stdout,
    ) -> Result<&mut Stdout, std::io::Error> {
        stdout.clear_all()
    }

    pub fn draw_status(&self) {
        let events = self.events_len();
        let mut world = self.world.borrow_mut();
        let score = world.player.score;
        let gas = world.player.gas / 100;
        let enemies = world.enemies.len();
        let traveled = world.player.traveled;
        let timers = world.timers.borrow().len();
        let elapsed_time = world.elapsed_time;

        world
            .canvas
            .draw_line(2, format!(" Score: {} ", score))
            .draw_line((2, 3), format!(" Fuel: {} ", gas / 100))
            .draw_line((2, 4), format!(" Time: {}s ", elapsed_time))
            .draw_line((2, 5), format!(" Enemies: {} ", enemies))
            .draw_line((2, 6), format!(" Traveled: {} ", traveled))
            .draw_line((2, 7), format!(" (dbg) Events: {} ", events))
            .draw_line((2, 8), format!(" (dbg) Timers: {} ", timers));
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
