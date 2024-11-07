use std::collections::{HashMap, HashSet};
use std::error::Error;
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime};
use crate::read_csv::{read_csv_file, ElectricUsage};

pub(crate) fn get_ac_energy_scaling() -> HashMap<String, f32> {
    let mut scaling = HashMap::new();
    scaling.insert("January".to_string(), 327.0);
    scaling.insert("February".to_string(), 352.0);
    scaling.insert("March".to_string(), 560.0);
    scaling.insert("April".to_string(), 630.0);
    scaling.insert("May".to_string(), 688.0);
    scaling.insert("June".to_string(), 747.0);
    scaling.insert("July".to_string(), 757.0);
    scaling.insert("August".to_string(), 724.0);
    scaling.insert("September".to_string(), 610.0);
    scaling.insert("October".to_string(), 504.0);
    scaling.insert("November".to_string(), 391.0);
    scaling.insert("December".to_string(), 330.0);
    scaling
}

// Add extra usage for EV charging every day at 19:00 - 9kWh before October 26, 2024
pub fn add_ev_charging(usage_data: &mut Vec<ElectricUsage>) {
    for record in usage_data.iter_mut() {
        if (record.start_time == "19:00" ) && record.date < "2024-10-26".to_string() {
            record.import_kwh += 9.0;
        }
    }
}

pub fn impute_hour_for_hour() -> Result<Vec<ElectricUsage>, Box<dyn Error>> {

    // Read the export dataset (June to July) for scaling export_kwh
    let export_kwh_usage_ref = read_csv_file("/Users/ludirehak/Downloads/solar panels/pge data/pge_electric_usage_interval_data_Service 1_1_2024-06-10_to_2024-07-11.csv")?;

    // Read the import dataset (April to May) for copying import_kwh
    let import_kwh_usage_ref = read_csv_file("/Users/ludirehak/Downloads/solar panels/pge data/pge_electric_usage_interval_data_Service 1_1_2024-04-15_to_2024-05-10.csv")?;

    // Get the scaling factors for each month
    let scaling_factors = get_ac_energy_scaling();
    let mut imputed_data = Vec::new();

    let start_date = "2023-11-02 00:00:00";
    let end_date = "2024-04-14 23:59:59";
    
    // Parse start and end datetimes for the imputation range (including time)
    let start_time = NaiveDateTime::parse_from_str(start_date, "%Y-%m-%d %H:%M:%S")
        .expect("Failed to parse start datetime");
    let end_time = NaiveDateTime::parse_from_str(end_date, "%Y-%m-%d %H:%M:%S")
        .expect("Failed to parse end datetime");

    let mut current_time = start_time;
    let mut import_data_idx = 0;  // Track index for import dataset

    // Iterate hour-by-hour from start_time to end_time
    while current_time <= end_time {
        let month_name = get_month_name(current_time.month());

        // Get the scaling factor for the export kWh based on the month
        let scaling_factor = scaling_factors.get(&month_name)
            .expect("Failed to get scaling factor for the month");

        // Ensure we don't run out of data in the import dataset set
        if import_data_idx >= import_kwh_usage_ref.len() {
            return Err("Insufficient import dataset data for import_kwh".into());
        }

        // Copy one record from import data for import_kwh
        let import_record = &import_kwh_usage_ref[import_data_idx];

        // Copy the corresponding record from the export data for export_kwh
        let export_record = &export_kwh_usage_ref[import_data_idx % export_kwh_usage_ref.len()]; // Loop through export dataset if shorter

        // Scale the export_kwh from the export dataset
        let scaled_export_kwh = export_record.export_kwh * (scaling_factor / 747.0); // June as baseline

        // Copy import_kwh from the import dataset without scaling
        let copied_import_kwh = import_record.import_kwh;

        // Set the start time and end time for the imputed record based on the current hour
        let start_time_str = current_time.format("%H:%M").to_string(); // e.g., "00:00"
        let end_time_str = (current_time + Duration::minutes(59)).format("%H:%M").to_string(); // e.g., "00:59"

        // Create an imputed record, overwriting the date and times
        imputed_data.push(ElectricUsage {
            r#type: "".to_string(),  // Type is copied from the export dataset
            date: current_time.date().format("%Y-%m-%d").to_string(),  // Overwrite the date
            start_time: start_time_str.clone(),  // Set the start time for the hour
            end_time: end_time_str.clone(),      // Set the end time for the hour
            import_kwh: copied_import_kwh,        // Copy import_kwh from import dataset
            export_kwh: scaled_export_kwh,        // Scale export_kwh from export dataset
            cost: "".to_string(),      // Copy cost from the export dataset
        });

        // Move to the next hour
        current_time += Duration::hours(1);

        // Move the index forward for every hour, looping over available data
        import_data_idx = (import_data_idx + 1) % import_kwh_usage_ref.len();
    }

    Ok(imputed_data)
}


fn get_month_name(month: u32) -> String {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
        .to_string()
}
