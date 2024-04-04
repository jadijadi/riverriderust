use std::{
    borrow::Borrow,
    io::{Stdout, Write},
    thread,
    time::Duration,
};

use crossterm::{
    event::{poll, read},
    style::ContentStyle,
};

use crate::{
    entities::{DeathCause, PlayerStatus},
    game::Game,
    utilities::drawable::Drawable,
    utilities::stout_ext::StdoutExt,
    World,
};

pub struct PopupDrawing {
    max_c: u16,
    max_l: u16,
    message: String,
    style: Option<ContentStyle>,
}

impl PopupDrawing {
    pub fn new(
        max_c: u16,
        max_l: u16,
        message: String,
        style: impl Into<Option<ContentStyle>>,
    ) -> Self {
        Self {
            max_c,
            max_l,
            message,
            style: style.into(),
        }
    }
}

impl Drawable for PopupDrawing {
    fn draw_on_canvas(&self, sc: &mut crate::canvas::Canvas) {
        let message_len = self.message.len();
        let line_0 = format!("    {}    ", " ".repeat(message_len));
        let line_1 = format!("  ╔═{}═╗  ", "═".repeat(message_len));
        let line_2 = format!("  ║ {} ║  ", self.message);
        let line_3 = format!("  ╚═{}═╝  ", "═".repeat(message_len));
        let line_4 = format!("    {}    ", " ".repeat(message_len));

        let message_len_offset = (message_len / 2) as u16 + 4;
        let col = self.max_c / 2 - message_len_offset;
        let center_l = self.max_l / 2;
        sc.draw_styled_line((col, center_l - 2), line_0, self.style)
            .draw_styled_line((col, center_l - 1), line_1, self.style)
            .draw_styled_line((col, center_l), line_2, self.style)
            .draw_styled_line((col, center_l + 1), line_3, self.style)
            .draw_styled_line((col, center_l + 2), line_4, self.style);
    }
}

impl<'g> World<'g> {
    pub fn popup(
        &self,
        message: impl Into<String>,
        style: impl Into<Option<ContentStyle>>,
    ) -> PopupDrawing {
        PopupDrawing::new(self.max_c(), self.max_l(), message.into(), style)
    }

    pub fn draw_on_canvas(&mut self) {
        self.canvas.clear_all();

        // draw the map
        self.canvas.draw(&self.map);

        for entity in self.entities.iter() {
            self.canvas.draw(entity);
        }

        self.canvas.draw(&self.player);

        for (_, drawing) in self.custom_drawings.iter() {
            let drawing: &dyn Drawable = drawing.borrow();
            drawing.draw_on_canvas(&mut self.canvas);
        }
    }

    pub fn pause_screen(&mut self) {
        self.canvas.draw(&self.popup("Game Paused!", None));
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
        let player = &world.player;
        let score = player.score;
        let fuel = (player.fuel as f32) / 100.0;
        let enemies = world.enemies().fold(0, |acc, _| acc + 1);
        let traveled = player.traveled;
        let timers = world.timers.borrow().len();
        let elapsed_time = world.elapsed_time;

        world
            .canvas
            .draw_line(2, format!(" Score: {} ", score))
            .draw_line((2, 3), format!(" Fuel: {} ", fuel))
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

        if world.max_c() > 100 {
            stdout.draw((0, 2), welcome_msg)?;
        } else {
            stdout.draw((0, 2), "RiverRaid Rust")?;
        }

        stdout.draw((2, world.max_l() - 2), "Press any key to continue...")?;
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

        stdout.move_cursor((2, world.max_l() - 5))?;
        if let PlayerStatus::Dead(cause) = &world.player.status {
            match cause {
                DeathCause::Ground => {
                    if world.max_c() > 91 {
                        stdout.print("\r█▄█ █▀█ █░█   █▀▀ █▀█ ▄▀█ █▀ █░█ █▀▀ █▀▄   █ █▄░█   ▀█▀ █░█ █▀▀   █▀▀ █▀█ █▀█ █░█ █▄░█ █▀▄ ░\n\r░█░ █▄█ █▄█   █▄▄ █▀▄ █▀█ ▄█ █▀█ ██▄ █▄▀   █ █░▀█   ░█░ █▀█ ██▄   █▄█ █▀▄ █▄█ █▄█ █░▀█ █▄▀ ▄\n\r")?;
                    } else {
                        stdout.print("You crashed in the ground.")?;
                    }
                }
                DeathCause::Enemy => {
                    if world.max_c() > 72 {
                        stdout.print("\r▄▀█ █▄░█   █▀▀ █▄░█ █▀▀ █▀▄▀█ █▄█   █▄▀ █ █░░ █░░ █▀▀ █▀▄   █▄█ █▀█ █░█ ░\n\r█▀█ █░▀█   ██▄ █░▀█ ██▄ █░▀░█ ░█░   █░█ █ █▄▄ █▄▄ ██▄ █▄▀   ░█░ █▄█ █▄█ ▄\n\r")?;
                    } else {
                        stdout.print("An enemy killed you.")?;
                    }
                }
                DeathCause::Fuel => {
                    if world.max_c() > 69 {
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

        stdout.move_cursor((2, world.max_l() - 2))?;
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
