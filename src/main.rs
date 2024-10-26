use std::time::{Duration, Instant};
use std::fs::File;
use std::io::{self, Read};
use std::thread;

fn parse_time(input: &str) -> Result<u64, String> {
    let parts: Vec<&str> = input.split(':').collect();
    if parts.len() != 2 {
        return Err("Invalid format".to_string());
    }

    let minutes: u64 = parts[0]
        .parse()
        .map_err(|_| "Invalid minutes".to_string())?;
    
    let seconds_parts: Vec<&str> = parts[1].split('.').collect();
    if seconds_parts.len() != 2 {
        return Err("Invalid seconds format".to_string());
    }

    let seconds: u64 = seconds_parts[0]
        .parse()
        .map_err(|_| "Invalid seconds".to_string())?;

    // Convert everything to seconds
    let total_seconds = (minutes * 60) + seconds;
    Ok(total_seconds)
}

// Gets the MOST RECENT lyric at the given time
fn get_lyric_at_time(lyrics: &Vec<[String; 2]>, time: u64) -> Option<String> {
    let mut lyric = None;
    for entry in lyrics {
        let parsed_time = match parse_time(&entry[0]) {
            Ok(parsed_time) => parsed_time,
            Err(_) => continue,
        };

        if parsed_time <= time {
            lyric = Some(entry[1].clone());
        } else {
            break;
        }
    }

    lyric
}

fn main() {
    let mut lyrics_arr: Vec<[String; 2]> = Vec::new();

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

    // Process the contents
    if let Some(start) = contents.find("[00:00.00]") {
        let trimmed_contents = &contents[start + "[00:00.00]".len()..];

        // Split into lines and then into [timestamp, lyric] pairs
        let lyrics_2d: Vec<[String; 2]> = trimmed_contents
            .lines()
            .filter_map(|line| {
                let mut parts = line.splitn(2, ']');
                let timestamp = parts.next()?.trim_start_matches('[').to_string();
                let lyric = parts.next()?.trim().to_string();
                Some([timestamp, lyric])
            })
            .collect();

        // Print the 2D array
        for entry in &lyrics_2d {
            println!("{:?}", entry);
        }
        // Set lyrics_arr
        for entry in &lyrics_2d {
            lyrics_arr.push([entry[0].clone(), entry[1].clone()]);
        }

    } else {
        println!("Marker [00:00.00] not found.");
    }


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
            println!("{}", lyrics_arr[curr_lyric_index][0]);

            match parse_time(&lyrics_arr[curr_lyric_index][0]) {
                Ok(parsed_time) => {
                    println!("Current: {}, Parsed: {}", current_time, parsed_time);
                    if current_time > parsed_time {
                        // println!("Current time is greater than the parsed time.");
                        println!("Current Lyric: {}", lyrics_arr[curr_lyric_index][1]);
                        curr_lyric_index += 1;
                    } else {
                        // println!("Curr<=Parsed");
                    }
                }
                Err(err) => {
                    println!("Error parsing time: {}", err);
                }
            }

            // if current_time > parse_time(lyrics_arr[currLyricIndex][0]) {
            //     println!("Current Lyric: {}", lyrics_arr[currLyricIndex][1]);
            //     currLyricIndex += 1;
            // }


            thread::sleep(Duration::from_secs(1));
        }
    });

    thread::sleep(Duration::from_secs(240));
}