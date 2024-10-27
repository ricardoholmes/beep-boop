pub fn parse_time(input: &str) -> Result<u64, String> {
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
pub fn get_lyric_at_time(lyrics: &Vec<(String, String)>, time: u64) -> Option<String> {
    let mut lyric = None;
    for (entry_time, entry_text) in lyrics {
        let parsed_time = match parse_time(entry_time) {
            Ok(parsed_time) => parsed_time,
            Err(_) => continue,
        };

        if parsed_time <= time {
            lyric = Some(entry_text.clone());
        } else {
            break;
        }
    }

    lyric
}

pub fn parse_lrc_file(contents: String) -> Vec<(String, String)> {
    let mut lyrics_arr: Vec<(String, String)> = Vec::new();

    // Process the contents
    if let Some(start) = contents.find("[00:00.00]") {
        let trimmed_contents = &contents[start + "[00:00.00]".len()..];

        // Split into lines and then into [timestamp, lyric] pairs
        lyrics_arr = trimmed_contents
            .lines()
            .filter_map(|line| {
                let mut parts = line.splitn(2, ']');
                let timestamp = parts.next()?.trim_start_matches('[').to_string();
                let lyric = parts.next()?.trim().to_string();
                Some((timestamp, lyric))
            })
            .collect();

        // Print the 2D array
        for entry in &lyrics_arr {
            // println!("{:?}", entry);
        }
    }
    else {
        panic!("Marker [00:00.00] not found.");
    }

    lyrics_arr
}
