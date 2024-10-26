use rand::{thread_rng, Rng};
use ratatui::{
    layout::Direction,
    style::{Color, Style},
    widgets::{Bar, BarChart, BarGroup, Widget}
};

pub fn get_wave() -> Vec::<u8> {
    let mut rng = thread_rng();
    (0..24).map(|_| rng.gen_range(50..90)).collect()
}

pub fn get_visualiser(wave: &[u8]) -> impl Widget + '_ {
    vertical_bars(wave)
}

/// Create a vertical bar chart from the wave data.
fn vertical_bars(wave: &[u8]) -> BarChart {
    let bars: Vec<Bar> = wave
        .iter()
        .map(|amplitude| vertical_bar(amplitude))
        .collect();

    BarChart::default()
        .data(BarGroup::default().bars(&bars))
        .bar_width(5)
}

fn vertical_bar(amplitude: &u8) -> Bar {
    Bar::default()
        .value(u64::from(*amplitude))
        .style(amplitude_style(*amplitude))
        .text_value(String::new())
}

/// Create a horizontal bar chart from the wave data.
fn horizontal_barchart(wave: &[u8]) -> BarChart {
    let bars: Vec<Bar> = wave
        .iter()
        .map(|amplitude| horizontal_bar(amplitude))
        .collect();

    BarChart::default()
        .data(BarGroup::default().bars(&bars))
        .bar_width(1)
        .bar_gap(0)
        .direction(Direction::Horizontal)
}

fn horizontal_bar(amplitude: &u8) -> Bar {
    let style = amplitude_style(*amplitude);
    Bar::default()
        .value(u64::from(*amplitude))
        .style(style)
        .text_value(String::new())
}

fn amplitude_style(value: u8) -> Style {
    let color = Color::Rgb(66, 134, 189);
    Style::new().fg(color)
}
