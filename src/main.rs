use cpal::{Data, Sample, SampleFormat, FromSample};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() {
    let host = cpal::default_host();
    let input_device = host.default_input_device().expect("no input device available");
    let output_device = host.default_output_device().expect("no output device available");

    println!("Input device: {}", input_device.name().unwrap());
    println!("Output device: {}", output_device.name().unwrap());

    // Configuration of audio STREAM
    let mut supported_configs_range = input_device.supported_input_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range.next()
        .expect("no supported config?!")
        .with_max_sample_rate();
    println!("Supported config: {:?}", supported_config);
    println!("Supported config.config: {:?}", supported_config.config());

    let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let sample_format = supported_config.sample_format();
    let config = supported_config.clone().into();



    let stream: cpal::Stream = match sample_format {
        SampleFormat::F32 => input_device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
            // react to stream events and read or write stream data here.
            // 'data' should be &Data for input streams
                println!("EEE: {:?}", data);
            },
            err_fn, None),
        SampleFormat::U8 => input_device.build_input_stream(
            &config,
            move |data: &[u8], _: &cpal::InputCallbackInfo| {
                // react to stream events and read or write stream data here.
                println!("Something happening");
            },
            err_fn, None),
        sample_format => panic!("Unsupported sample format '{sample_format}'")
    }.unwrap();

    stream.play().unwrap();
    // Run for 3 seconds before closing.
    println!("Playing for 3 seconds... ");
    std::thread::sleep(std::time::Duration::from_secs(3));
    drop(stream);
    println!("Done!");


}
