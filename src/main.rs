mod lyrics;

use std::time::{Duration, Instant};
use std::fs::File;
use std::io::Read;
use std::thread;

use lyrics::{parse_lrc_file, parse_time, get_lyric_at_time};


fn main() {
    // Read File
    let mut file = match File::open("lrc/Linkin Park - Heavy Is the Crown.lrc") {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Error opening file: {}", error);
            return;
        }
    };

    let mut contents = String::new();
    if let Err(error) = file.read_to_string(&mut contents) {
        eprintln!("Error reading file: {}", error);
        return;
    }

    let lyrics_arr = parse_lrc_file(contents);

    // Start Timer
    let now = Instant::now();
    let mut curr_lyric_index = 0;

    println!("LYRIC AT GIVEN TIME: {}", get_lyric_at_time(&lyrics_arr, 120).unwrap());


    thread::spawn(move || {
        loop {
            // Compare time with last 'indexed + 1' lyric time
            // If time is greater than last 'indexed + 1' lyric time, print the lyric
            // Move index up by 1

            let current_time = now.elapsed().as_secs();
            println!("{}", lyrics_arr[curr_lyric_index].0);

            match parse_time(&lyrics_arr[curr_lyric_index].0) {
                Ok(parsed_time) => {
                    println!("Current: {}, Parsed: {}", current_time, parsed_time);
                    if current_time > parsed_time {
                        // println!("Current time is greater than the parsed time.");
                        println!("Current Lyric: {}", lyrics_arr[curr_lyric_index].1);
                        curr_lyric_index += 1;
                    } else {
                        // println!("Curr<=Parsed");
                    }
                }
                Err(err) => {
                    println!("Error parsing time: {}", err);
                }
            }

            // if current_time > parse_time(lyrics_arr[currLyricIndex].0) {
            //     println!("Current Lyric: {}", lyrics_arr[currLyricIndex].1);
            //     currLyricIndex += 1;
            // }


            thread::sleep(Duration::from_secs(1));
        }
    });

    thread::sleep(Duration::from_secs(240));
}