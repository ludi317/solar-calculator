mod impute;
mod read_csv;
mod impute_check;
mod solar_install_cost;

use std::error::Error;

use crate::read_csv::{read_all_csv_files_in_directory};
use crate::impute::{add_ev_charging, impute_hour_for_hour};
use crate::solar_install_cost::cost_of_getting_to_capacity;

fn main() -> Result<(), Box<dyn Error>> {

    // Call the function to read all CSV files in the directory
    let pge_data = read_all_csv_files_in_directory()?;

    // find which day had the maximum export_kwh
    let max_day = pge_data.iter().max_by(|a, b| a.export_kwh.partial_cmp(&b.export_kwh).unwrap()).unwrap();

    println!("Max day: {:?}", max_day);

    // Perform the imputation by scaling export_kwh and copying import_kwh
    let mut all_data = impute_hour_for_hour()?;

    // append all_data with pge_data
    all_data.extend(pge_data);

    // add extra usage to account for EV charging
    add_ev_charging(&mut all_data);

    // sum all import_kwh
    let total_import_kwh: f32 = all_data.iter().map(|x| x.import_kwh).sum();
    println!("Total import_kwh: {:.2}", total_import_kwh);

    assert_eq!(all_data.len(), 366 * 24);

    // Run the cost calculation
    let mut a: f32 = 16./26.;
    let mut min_tot_cost = std::f32::MAX;
    let mut min_a = -1.0;
    let mut min_b = -1;
    while a <= 90./26. {
        for b in (0..=30).step_by(5) {
            let total_cost = solar_install_cost::annual_cost(&all_data, a, b as f32);
            if total_cost < min_tot_cost {
                min_tot_cost = total_cost;
                min_a = a;
                min_b = b;
            }
            // println!("Total annual cost for a = {}, b = {}: ${:.2}", a, b, total_cost);
        }
        a += 2./26.;
    }

    println!("Minimum total annual cost: ${:.2} for a = {}, b = {}", min_tot_cost, min_a, min_b);
    cost_of_getting_to_capacity(min_a, 26, true);


    Ok(())
}
