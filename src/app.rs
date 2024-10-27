use crate::audio_visualiser;

use audio_visualiser::get_visualiser;
use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout},
    style::Stylize,
    DefaultTerminal, Frame,
};
use ringbuf::traits::Consumer;

use std::{cmp::Ordering, time::Duration};

use audioviz::spectrum::config::ProcessorConfig;
use cpal::{Device, SampleFormat, Stream};
use cpal::traits::DeviceTrait;

use ringbuf::{
    traits::{Producer, Split},
    HeapRb,
};

use audioviz::spectrum;

pub struct Config {
    pub is_horizontal: bool,
    is_stereo: bool,
}

pub struct App {
    _input_stream: Stream,
    should_exit: bool,
    get_buffer: Box<dyn FnMut() -> Vec<f32>>,
    processor_config: ProcessorConfig,
    config: Config,
    frame_num: u64,
    prev_wave_data: Option<Vec<u64>>,
}

impl App {
    pub fn new(input_device: Device) -> Self {
        // Configuration of audio STREAM
        let mut supported_configs_range = input_device.supported_input_configs()
            .expect("error while querying configs");
        let supported_config = supported_configs_range.next()
            .expect("no supported config?!")
            .with_max_sample_rate();
        // println!("Supported config: {:?}", supported_config);
        // println!("Supported config.config: {:?}", supported_config.config());

        let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);
        let _sample_format: SampleFormat = supported_config.sample_format();
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

        let get_buffer_fn = move || {
            let mut out = vec![];
            while let Some(x) = consumer.try_pop() {
                out.push(x);
            }
            out
        };

        // Build streams.
        // println!(
        //     "Attempting to build both streams with f32 samples and `{:?}`.",
        //     config
        // );
        let input_stream = input_device.build_input_stream(&config, input_data_fn, err_fn, None).unwrap();

        let mut viz_config = ProcessorConfig::default();
        viz_config.sampling_rate = config.sample_rate.0;

        let get_buffer = Box::new(get_buffer_fn);
        Self {
            _input_stream: input_stream,
            should_exit: false,
            get_buffer,
            processor_config: viz_config,
            config: Config {
                is_horizontal: false,
                is_stereo: false,
            },
            frame_num: 0,
            prev_wave_data: None,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        if let Ok(x) = event::poll(Duration::ZERO) {
            if x == true {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') => self.should_exit = true,
                            KeyCode::Char('r') => self.config.is_horizontal = !self.config.is_horizontal,
                            KeyCode::Char(' ') => self.config.is_stereo = !self.config.is_stereo,
                            _ => (),
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let [title, visualiser_area, stuff] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(1),
            ])
            .spacing(1)
            .areas(frame.area());

        let buf = (self.get_buffer)();
        let mut processor = spectrum::processor::Processor::from_raw_data(self.processor_config.clone(), buf);
        processor.raw_to_freq_buffer();
        let mut wave = processor.freq_buffer.clone();

        let wave_data = if wave.is_empty() {
            self.prev_wave_data.clone().unwrap_or((0..21).map(|_| 0).collect())
        } else {
            wave.sort_by(|x, y| x.freq.total_cmp(&y.freq));
            let wave = wave.iter();
            let mut wave_data = vec![];
            for low_bound in (0..20_0000).step_by(1000) {
                let high_bound = low_bound + 1000;
                let waves_in_range: Vec<u64> = wave
                    .clone()
                    .skip_while(|freq| (freq.freq as i32) < low_bound)
                    .take_while(|freq| (freq.freq as i32) < high_bound)
                    .map(|freq| (freq.volume * 100.0) as u64)
                    .collect();

                let data: u64 = if waves_in_range.len() > 0 {
                    waves_in_range.iter().fold(0, |t, &x| t + x)// / (waves_in_range.len() as u64)
                } else {
                    30
                };
                wave_data.push(data);
            }

            if let Some(old_data) = &self.prev_wave_data {
                for i in 0..old_data.len() {
                    wave_data[i] = match wave_data[i].cmp(&old_data[i]) {
                        Ordering::Equal => wave_data[i],
                        // Ordering::Less => (old_data[i].checked_sub(2).unwrap_or(0)).max(wave_data[i]),
                        // Ordering::Greater => (old_data[i] + 2).min(wave_data[i]),
                        Ordering::Less => old_data[i] - ((old_data[i] - wave_data[i]) / 10),
                        Ordering::Greater => old_data[i] + ((wave_data[i] - old_data[i]) / 10),
                    };
                }
            }
            self.prev_wave_data = Some(wave_data.clone());
            wave_data
        };


        let ma = wave_data.iter().max().unwrap();
        let mi = wave_data.iter().min().unwrap();
        frame.render_widget(format!("{} | {mi} <-> {ma}", self.frame_num), stuff);

        frame.render_widget("BEEP BOOP".bold().into_centered_line(), title);

        if self.config.is_stereo {
            if self.config.is_horizontal {
                let [inner_layout_l, inner_layout_r] = Layout::horizontal([
                    Constraint::Fill(1),
                    Constraint::Fill(1),
                ]).areas(visualiser_area);

                frame.render_widget(get_visualiser(&self.config, &wave_data), inner_layout_l);
                frame.render_widget(get_visualiser(&self.config, &wave_data), inner_layout_r);
            } else {
                let [inner_layout_u, inner_layout_d] = Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Fill(1),
                ]).areas(visualiser_area);

                frame.render_widget(get_visualiser(&self.config, &wave_data), inner_layout_u);
                frame.render_widget(get_visualiser(&self.config, &wave_data), inner_layout_d);
            }
        } else {
            frame.render_widget(get_visualiser(&self.config, &wave_data), visualiser_area);
        }

        self.frame_num += 1;
    }
}
