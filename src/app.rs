use crate::{audio_visualiser, lyrics::{self, get_lyric_at_time}};

use audio_visualiser::get_visualiser;
use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind}, layout::{Constraint, Layout}, style::{Color, Stylize}, text::ToText, widgets::Gauge, DefaultTerminal, Frame
};
use rodio::{Decoder, OutputStream, Source};

use std::{cmp::Ordering, fs::{self, File}, io::BufReader, path::PathBuf, thread::{self, JoinHandle}, time::{Duration, Instant}};

use audioviz::spectrum::{config::ProcessorConfig, Frequency};


use audioviz::spectrum;

pub struct Config {
    pub is_horizontal: bool,
    is_stereo: bool,
}

pub struct App {
    _audio_thread: JoinHandle<()>,
    start: Instant,
    lrc_parsed: Vec<(String, String)>,
    sound_file: PathBuf,
    should_exit: bool,
    processor_config: ProcessorConfig,
    config: Config,
    frame_num: u64,
    prev_wave_data: Option<Vec<u64>>,
}

impl App {
    pub fn new(sound_file: PathBuf, lrc_file: PathBuf) -> Self {
        let lrc_contents = fs::read_to_string(lrc_file).unwrap();
        let lrc_parsed = lyrics::parse_lrc_file(lrc_contents);

        let snd_file = sound_file.clone();
        let audio_thread = thread::spawn(move || {
            // Get an output stream handle to the default physical sound device
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            // Load a sound from a file, using a path relative to Cargo.toml
            let file = BufReader::new(File::open(snd_file).unwrap());
            // Decode that sound file into a source
            let source = Decoder::new(file).unwrap();

            let total_duration = source.total_duration().unwrap();

            // Play the sound directly on the device
            let _ = stream_handle.play_raw(source.convert_samples());
            
            let start = Instant::now();
            while start.elapsed() < total_duration { }
        });

        let viz_config = ProcessorConfig::default();
        // viz_config.sampling_rate = config.sample_rate.0;

        Self {
            _audio_thread: audio_thread,
            start: Instant::now(),
            lrc_parsed,
            sound_file,
            should_exit: false,
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
        let [title, lyrics_area, visualiser_area, progress_area, stuff] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .spacing(1)
            .areas(frame.area());

        let start = self.start.elapsed().max(Duration::from_millis(50)) - Duration::from_millis(50);
        let dur = self.start.elapsed() - start;
        let decoder = Decoder::new(BufReader::new(File::open(self.sound_file.clone()).unwrap())).unwrap();
        let total_dur= decoder.total_duration().unwrap_or_default();
        let data = decoder.convert_samples().skip_duration(start).take_duration(dur);
        let buf: Vec<f32> = data.collect();

        let mut processor = spectrum::processor::Processor::from_raw_data(self.processor_config.clone(), buf.clone());
        processor.raw_to_freq_buffer();
        let mut wave = processor.freq_buffer.clone();

        let mut wave_data = if wave.is_empty() {
            // self.prev_wave_data.clone().unwrap_or((0..21).map(|_| 0).collect())
            (0..21).map(|_| 0).collect()
        } else {
            wave.sort_by(|x, y| x.freq.total_cmp(&y.freq));
            let wave = wave.iter();
            let mut wave_data = vec![];
            for low_bound in (0..20_000).step_by(100) {
                let high_bound = low_bound + 100;
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
            wave_data
        };

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

        let mut x = vec![];
        for low_bound in (0..20_000).step_by(1000) {
            let high_bound = low_bound + 1000;
            x.push(processor.freq_buffer.clone().iter().filter(|f| f.freq >= low_bound as f32 && f.freq < high_bound as f32).collect::<Vec<&Frequency>>().len())
        }

        let ma = wave_data.iter().max().unwrap();
        let mi = wave_data.iter().min().unwrap();
        frame.render_widget(format!("{} | {mi} <-> {ma} | {} | {} | {:.01}%", self.frame_num, buf.len(), wave_data.len(), (self.start.elapsed().as_secs_f64() / total_dur.as_secs_f64()) * 100.0), stuff);

        let current_time = format!("{:02}:{:02}", self.start.elapsed().as_secs() / 60, self.start.elapsed().as_secs() % 60);
        let total_time = format!("{:02}:{:02}", total_dur.as_secs() / 60, total_dur.as_secs() % 60);
        let [progress_l, progress_m, progress_r] = Layout::horizontal([
            Constraint::Percentage(3),
            Constraint::Fill(1),
            Constraint::Percentage(3),
        ]).areas(progress_area);
        frame.render_widget(current_time.to_text().centered(), progress_l);
        frame.render_widget(total_time.to_text().centered(), progress_r);

        let progress_line = Gauge::default()
            .gauge_style(Color::Rgb(66, 134, 189))
            .ratio((self.start.elapsed().as_secs_f64() / total_dur.as_secs_f64()).min(1.0));
        frame.render_widget(progress_line, progress_m);

        frame.render_widget("BEEP BOOP".bold().into_centered_line(), title);
        let lyric = get_lyric_at_time(&self.lrc_parsed, self.start.elapsed().as_secs()).unwrap_or_default();
        frame.render_widget(lyric.to_text().centered(), lyrics_area);

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
