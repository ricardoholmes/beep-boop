use audioviz::spectrum::Frequency;
use rand::{thread_rng, Rng};
use ratatui::{
    layout::Direction,
    style::{Color, Style},
    widgets::{Bar, BarChart, BarGroup, Widget}
};

use crate::app::Config;

pub fn get_wave() -> Vec::<u8> {
    let mut rng = thread_rng();
    (0..24).map(|_| rng.gen_range(50..90)).collect()
}

pub fn get_visualiser<'a>(config: &'a Config, wave_data: &'a [u64]) -> impl Widget + 'a  {

    if config.is_horizontal {
        horizontal_barchart(wave_data)
    } else {
        vertical_barchart(wave_data)
    }
}

/// Create a vertical bar chart from the wave data.
fn vertical_barchart(wave: &[u64]) -> BarChart {
    let bars: Vec<Bar> = wave
        .iter()
        .map(vertical_bar)
        .collect();

    BarChart::default()
        .data(BarGroup::default().bars(&bars))
        .bar_width(5)
}

fn vertical_bar(amplitude: &u64) -> Bar {
    let style = amplitude_style(*amplitude);
    Bar::default()
        .value(*amplitude)
        .style(style)
        .text_value(String::new())
}

/// Create a horizontal bar chart from the wave data.
fn horizontal_barchart(wave: &[u64]) -> BarChart {
    let bars: Vec<Bar> = wave
        .iter().take(20)
        .map(horizontal_bar)
        .collect();

    BarChart::default()
        .data(BarGroup::default().bars(&bars))
        .bar_width(1)
        .bar_gap(0)
        .direction(Direction::Horizontal)
}

fn horizontal_bar(amplitude: &u64) -> Bar {
    let style = amplitude_style(*amplitude);
    Bar::default()
        .value(*amplitude)
        .style(style)
        .text_value(String::new())
}

fn amplitude_style(_value: u64) -> Style {
    let color = Color::Rgb(66, 134, 189);
    Style::new().fg(color)
}
