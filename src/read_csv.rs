
use std::error::Error;
use std::fs::{read_dir, DirEntry};
use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader};
use csv::ReaderBuilder;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct ElectricUsage {
    #[serde(rename = "TYPE")]
    pub r#type: String,
    #[serde(rename = "DATE")]
    pub date: String,
    #[serde(rename = "START TIME")]
    pub start_time: String,
    #[serde(rename = "END TIME")]
    pub end_time: String,
    #[serde(rename = "IMPORT (kWh)")]
    pub import_kwh: f32,
    #[serde(rename = "EXPORT (kWh)")]
    pub export_kwh: f32,
    #[serde(rename = "COST")]
    pub cost: String,
}

pub fn read_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<ElectricUsage>, Box<dyn Error>> {
    let file = File::open(&path)?;
    let reader = BufReader::new(file);

    // Skip lines until we find the header (TYPE, DATE, etc.)
    let mut lines = reader.lines().skip_while(|line| {
        match line {
            Ok(ref l) => !l.starts_with("TYPE,DATE"), // Skip until we find the header
            _ => true, // Skip any error lines
        }
    });

    // Read the actual CSV data, starting from the header
    lines.next(); // Skip the header line
    let remaining_lines: Vec<String> = lines.filter_map(|line| line.ok()).collect();

    let binding = remaining_lines.join("\n");
    let mut rdr = ReaderBuilder::new()
        .has_headers(false) // We already handled the header
        .from_reader(binding.as_bytes());


    let mut data = Vec::new();

    // Handle deserialization with possible missing fields
    for result in rdr.deserialize() {
        match result {
            Ok(record) => data.push(record),
            Err(e) => eprintln!("Error deserializing record: {}", e), // Log the error but continue
        }
    }

    Ok(data)
}


pub fn read_all_csv_files_in_directory() -> Result<Vec<ElectricUsage>, Box<dyn Error>> {

    let dir_path = "/Users/ludirehak/Downloads/solar panels/pge data";
    let mut all_data: Vec<ElectricUsage> = Vec::new();

    // Read directory and collect entries
    let mut entries: Vec<DirEntry> = read_dir(dir_path)?
        .filter_map(|entry| entry.ok())
        .collect();
    // Sort entries alphabetically by their file name
    entries.sort_by_key(|entry| entry.path());

    // Process each CSV file
    for entry in entries {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("csv") {
            // Print the file path
            println!("Reading file: {:?}", path.display());

            // Read the CSV file and extend the all_data vector with its contents
            let data = read_csv_file(&path)?;

            // while last date of data equals first date of all_data, skip
            if !all_data.is_empty() {
                while all_data.last().unwrap().date == data.first().unwrap().date {
                    all_data.pop();
                }
            }

            all_data.extend(data); // Append the data from this file
        }
    }
    Ok(all_data)
}
