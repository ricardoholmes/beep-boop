mod app;
mod audio_visualiser;

use app::App;
use cpal::traits::{DeviceTrait, HostTrait};
use std::io::{self, BufRead, Write};
use color_eyre::Result;

fn main() -> Result<()> {
    let host = cpal::default_host();

    for (i, d) in host.input_devices().unwrap().enumerate() {
        println!("{i}: {}", d.name().unwrap());
    }
    print!("\nDevice ID: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;

    let device_id = input.trim_end().parse().unwrap();

    let input_device = host.input_devices().expect("no input device available").skip(device_id).next().unwrap();

    println!("\nInput device: {}", input_device.name().unwrap());

    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::new(input_device).run(terminal);
    ratatui::restore();
    app_result
}
