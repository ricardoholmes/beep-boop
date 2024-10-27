mod app;
mod lyrics;
mod audio_visualiser;

use app::App;
use color_eyre::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    sound_file: std::path::PathBuf,
    lrc_file: std::path::PathBuf,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    // let host = cpal::default_host();

    // for (i, d) in host.output_devices().unwrap().enumerate() {
    //     println!("{i}: {}", d.name().unwrap());
    // }
    // print!("\nDevice ID: ");
    // stdout().flush()?;

    // let mut input = String::new();
    // stdin().lock().read_line(&mut input)?;

    // let device_id = input.trim_end().parse().unwrap();

    // let output_device = host.output_devices()?.skip(device_id).next().expect("Invalid device");

    // println!("\nOutput device: {}", output_device.name().unwrap());

    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::new(args.sound_file, args.lrc_file).run(terminal);
    ratatui::restore();
    app_result
}
