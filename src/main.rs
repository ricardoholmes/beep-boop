mod app;
mod audio_visualiser;

use std::io::{BufRead, Write};
use std::io;

use audioviz::spectrum::config::ProcessorConfig;
use cpal::SampleFormat;
use cpal::traits::{DeviceTrait, HostTrait};

use ringbuf::{
    traits::{Consumer, Producer, Split},
    HeapRb,
};

use audioviz::spectrum;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    // Configuration of audio STREAM
    let mut supported_configs_range = input_device.supported_input_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range.next()
        .expect("no supported config?!")
        .with_max_sample_rate();
    println!("Supported config: {:?}", supported_config);
    println!("Supported config.config: {:?}", supported_config.config());

    let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let sample_format: SampleFormat = supported_config.sample_format();
    let config: cpal::StreamConfig = supported_config.clone().into();


    // Create a delay in case the input and output devices aren't synced.
    let latency_frames = (5000.0 / 1_000.0) * config.sample_rate.0 as f32;
    let latency_samples = latency_frames as usize * config.channels as usize;
    // The buffer to share samples
    let ring = HeapRb::<f32>::new(latency_samples * 2);
    let (mut producer, mut consumer) = ring.split();

    // Fill the samples with 0.0 equal to the length of the delay.
    for _ in 0..latency_samples {
        // The ring buffer has twice as much space as necessary to add latency here,
        // so this should never fail
        producer.try_push(0.0).unwrap();
    }

    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        let mut output_fell_behind = false;
        for &sample in data {
            // println!("sample: {}", sample);
            if producer.try_push(sample).is_err() {
                output_fell_behind = true;
            }
        }
        if output_fell_behind {
            eprintln!("output stream fell behind: try increasing latency");
        }
    };

    // Build streams.
    println!(
        "Attempting to build both streams with f32 samples and `{:?}`.",
        config
    );
    let input_stream = input_device.build_input_stream(&config, input_data_fn, err_fn, None)?;

    std::thread::sleep(std::time::Duration::from_secs(3));

    let mut viz_config = ProcessorConfig::default();
    viz_config.sampling_rate = config.sample_rate.0;

    let mut processor = spectrum::processor::Processor::from_raw_data(viz_config, consumer.into_iter().collect());
    processor.raw_to_freq_buffer();

    drop(input_stream);
    println!("Done!");
    Ok(())
}
