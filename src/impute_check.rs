use std::collections::HashSet;
use chrono::{Duration, NaiveDate};
use crate::read_csv::ElectricUsage;

// Generate all expected hours for the date range
pub(crate) fn generate_expected_hours(start_date: &str, end_date: &str) -> HashSet<(String, String)> {
    let mut expected_hours = HashSet::new();

    // Parse the start and end dates
    let start = NaiveDate::parse_from_str(start_date, "%Y-%m-%d").expect("Invalid start date");
    let end = NaiveDate::parse_from_str(end_date, "%Y-%m-%d").expect("Invalid end date");

    let mut current_date = start;
    while current_date <= end {
        // For each hour of the day (00:00 to 23:00)
        for hour in 0..24 {
            let start_time = format!("{:02}:00", hour);
            let end_time = format!("{:02}:59", hour);
            expected_hours.insert((
                current_date.format("%Y-%m-%d").to_string(),  // Date in "YYYY-MM-DD" format
                start_time,                                  // Start time for the hour
            ));
        }
        current_date += Duration::days(1);
    }

    expected_hours
}

// Function to find duplicate hours in a Vec<ElectricUsage>
pub fn find_duplicate_hours(usage_data: &Vec<ElectricUsage>) -> Vec<(String, String)> {
    let mut seen_hours = HashSet::new();
    let mut duplicates = Vec::new();

    for record in usage_data {
        let key = (record.date.clone(), record.start_time.clone());

        // Check if the (date, start_time) pair has been seen before
        if !seen_hours.insert(key.clone()) {
            // If it was already seen, it's a duplicate
            duplicates.push(key);
        }
    }

    duplicates
}

// Compare the actual records with the expected hours and find missing ones
pub(crate) fn find_missing_hours(
    usage_data: &Vec<ElectricUsage>,
    expected_hours: &HashSet<(String, String)>
) -> Vec<(String, String)> {
    let mut missing_hours = Vec::new();

    // Collect the actual dates and hours in a HashSet for quick lookup
    let actual_hours: HashSet<(String, String)> = usage_data
        .iter()
        .map(|record| (record.date.clone(), record.start_time.clone()))
        .collect();

    // Compare expected with actual and find missing entries
    for expected in expected_hours {
        if !actual_hours.contains(expected) {
            missing_hours.push(expected.clone());
        }
    }

    missing_hours
}