#![windows_subsystem = "windows"]

extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;

#[macro_use]
extern crate imgui;
extern crate imgui_gfx_renderer;

#[macro_use]
extern crate enum_primitive;


use imgui::*;

mod support_gfx;
mod philosophers;

const CLEAR_COLOR: [f32; 4] = [0.4, 0.5, 0.6, 1.0];

fn main() {
    let names = [
        "Judith Butler",
        "Gilles Deleuze",
        "Karl Marx",
        "Emma Goldman",
        "Michel Foucault"
    ];

    let dining_philosophers = philosophers::DiningPhilosophers::new(names.len());

    support_gfx::run("Dining Philosophers".to_owned(), CLEAR_COLOR, |ui: &Ui| -> bool {
            ui.window(im_str!("Philosophers"))
                .size((320.0, 130.0), ImGuiCond::FirstUseEver)
                .position((60.0, 60.0), ImGuiCond::FirstUseEver)
                .build(|| {
                    ui.columns(2, im_str!("Philosophers"), true);
                    for i in 0..names.len() {
                        ui.text(names[i]);
                    }
                    ui.next_column();
                    for i in 0..names.len() {
                        ui.text(dining_philosophers.get_state(i).to_string());
                    }
                });
            ui.window(im_str!("Forks"))
                .size((300.0, 130.0), ImGuiCond::FirstUseEver)
                .position((390.0, 60.0), ImGuiCond::FirstUseEver)
                .build(|| {
                    ui.columns(2, im_str!("Forks"), true);
                    for i in 0..names.len() {
                        ui.text(i.to_string());
                    }
                    ui.next_column();
                    for i in 0..names.len() {
                        match dining_philosophers.get_fork(i) {
                        None => ui.text(im_str!("Available")),
                        Some(owner_index) => ui.text(names[owner_index]),
                        }
                    }
                });

            ui.window(im_str!("Config"))
                .size((630.0, 200.0), ImGuiCond::FirstUseEver)
                .position((60.0, 210.0), ImGuiCond::FirstUseEver)
                .build(|| {
                    ui.slider_int(im_str!("Fork Pick Up Time"), unsafe {&mut philosophers::TIME_FORK_PICKUP}, 1, 10000).build();
                    ui.slider_int(im_str!("Fork Put Down Time"), unsafe {&mut philosophers::TIME_FORK_PUTDOWN}, 1, 10000).build();
                    ui.slider_int(im_str!("Eating Time"), unsafe {&mut philosophers::TIME_EATING}, 1, 10000).build();
                    ui.slider_int(im_str!("Thinking Time"), unsafe {&mut philosophers::TIME_THINKING}, 1, 10000).build();
                    ui.columns(4, im_str!("Buttons"), false);
                    if ui.button(im_str!("Fast"), (ui.get_column_width(0), 20.0)) {
                        unsafe {
                            philosophers::TIME_FORK_PICKUP = 1;
                            philosophers::TIME_FORK_PUTDOWN = 1;
                            philosophers::TIME_EATING = 1;
                            philosophers::TIME_THINKING = 1;
                        }
                    }
                    ui.next_column();
                    if ui.button(im_str!("Medium"), (ui.get_column_width(1), 20.0)) {
                        unsafe {
                            philosophers::TIME_FORK_PICKUP = 500;
                            philosophers::TIME_FORK_PUTDOWN = 500;
                            philosophers::TIME_EATING = 3000;
                            philosophers::TIME_THINKING = 6000;
                        }
                    }
                    ui.next_column();
                    if ui.button(im_str!("Slow"), (ui.get_column_width(2), 20.0)) {
                        unsafe {
                            philosophers::TIME_FORK_PICKUP = 5000;
                            philosophers::TIME_FORK_PUTDOWN = 5000;
                            philosophers::TIME_EATING = 10000;
                            philosophers::TIME_THINKING = 10000;
                        }
                    }
                });

            true
        }
    );

    dining_philosophers.wait();
}
