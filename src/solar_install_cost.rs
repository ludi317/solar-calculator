use chrono::{Datelike, NaiveDateTime, Timelike};
use crate::read_csv::ElectricUsage;

// Constants for costs and rates
const COST_PER_KWH: f32 = 0.45;       // Cost per kWh from the grid
const CREDIT_PER_KWH: f32 = 0.04;     // Credit per kWh sent to the grid; based on pg&e export pricing
const INSTALL_COST_PER_PANEL: f32 = 1300.0;  // Cost per new panel (for additional capacity)
const BATTERY_INSTALL_COST_PER_KWH: f32 = 1000.0;  // Cost per kWh of battery installed

// Function to calculate the total cost based on inputs
// a = Desired solar capacity compared to current
// b = Battery capacity in kWh
pub(crate) fn annual_cost(usage_data: &Vec<ElectricUsage>, a: f32, b: f32) -> f32 {

    // Solar panel parameters
    let r = 0.03; // Effective inflation-adjusted yearly cost

    let panel_install_cost = cost_of_getting_to_capacity(a, 26, false);

    let mut total_cost: f32 = 0.0;
    let mut battery_charge = 0.0;  // Start with the initial battery charge

    // Add initial installation costs (panels + batteries)
    total_cost += r * panel_install_cost;
    total_cost += r * b * BATTERY_INSTALL_COST_PER_KWH;

    // Iterate through each hour of the usage data
    for record in usage_data {
        let production = a * record.export_kwh;
        let consumption = record.import_kwh;

        let net_production = production - consumption;  // di

        // If there is excess consumption and the battery is empty
        if net_production < 0.0 {
            let energy_needed = -net_production;
            if battery_charge >= energy_needed {
                // Use battery charge to meet the consumption
                battery_charge -= energy_needed;
            } else {
                // Grid consumption
                let grid_consumption = energy_needed - battery_charge;
                total_cost += grid_consumption * COST_PER_KWH;  // Pay for this consumption
                battery_charge = 0.0;  // Battery fully drained
            }
        } else {
            // Excess production
            let excess_production = net_production;

            // Charge the battery first
            let charge_room = b - battery_charge;
            if excess_production <= charge_room {
                battery_charge += excess_production;  // Fill the battery
            } else {
                // Fill the battery and send the rest to the grid
                battery_charge = b;
                let to_grid = excess_production - charge_room;
                total_cost -= to_grid * CREDIT_PER_KWH;  // Earn credit for sending energy to the grid
            }
        }
    }

    total_cost
}


const PUT_BACK_OLD_PANELS_COST: f32 = 5000.0;  // Cost to move old panels (for up to 10 panels)
const JUNK_OLD_PANELS_COST: f32 = 1500.0;  // Cost to junk 10 old panels; 2500
const NEW_PANEL_COST_PER_UNIT: f32 = 1400.;  // Cost per new panel
const NEW_PANEL_EFFICIENCY_FACTOR: f32 = 2.5; // New panels are 3x more efficient than old panels

// Function to calculate the cost of getting to solar capacity a
pub(crate) fn cost_of_getting_to_capacity(a: f32, old_panels: usize, verbose: bool) -> f32 {
    let old_panels = old_panels as f32;

    // Calculate the number of new panels needed, adjusted by the efficiency factor
    let new_panels_needed_ifjunk = ((a * old_panels - 16.0) / NEW_PANEL_EFFICIENCY_FACTOR).ceil();
    let new_panels_cost_ifjunk = new_panels_needed_ifjunk * NEW_PANEL_COST_PER_UNIT;

    if a <= 1.0 {
        // For a <= 1.0, either salvage the old panels or junk and replace with new
        let move_and_salvage_cost = PUT_BACK_OLD_PANELS_COST; // Salvaging just requires moving the old panels back
        let junk_and_new_cost = JUNK_OLD_PANELS_COST + new_panels_cost_ifjunk; // Junking and replacing with new panels

        // Return the cheaper option between moving old panels or junking and installing new panels
        if move_and_salvage_cost < junk_and_new_cost {
            if verbose {
                println!("Moving old panels back to reach capacity");
            }
            move_and_salvage_cost
        } else {
            if verbose {
                println!("Junking old panels and installing new panels to reach capacity");
            }
            junk_and_new_cost
        }
    } else {
        // For a > 1.0, calculate the cost of using a combination of old and new panels

        // Option 1: Move old panels back and install new panels to reach the required capacity
        // Account for the efficiency of new panels
        let additional_capacity_needed = (a - 1.0) * old_panels;
        let new_panels_for_additional_capacity = (additional_capacity_needed / NEW_PANEL_EFFICIENCY_FACTOR).ceil();
        let move_old_and_add_new_cost = PUT_BACK_OLD_PANELS_COST + (new_panels_for_additional_capacity * NEW_PANEL_COST_PER_UNIT);

        // Option 2: Junk the old panels and install all new panels to reach the capacity
        let junk_old_and_install_new_cost = JUNK_OLD_PANELS_COST + new_panels_cost_ifjunk;

        // Return the cheaper option between moving old back and adding new, or junking and installing all new
        if move_old_and_add_new_cost < junk_old_and_install_new_cost {
            if verbose {
                println!("Moving old panels back and installing new panels to reach capacity");
                println!("Number of new panels needed: {}", new_panels_for_additional_capacity);
            }
            move_old_and_add_new_cost
        } else {
            if verbose {
                println!("Junking old panels and installing new panels to reach capacity");
                println!("Number of new panels needed: {}", new_panels_needed_ifjunk);
            }
            junk_old_and_install_new_cost
        }
    }
}

// https://www.pge.com/tariffs/assets/pdf/tariffbook/ELEC_SCHEDS_E-ELEC.pdf, page 2
fn cost_per_kWh(kWh: f32, start_time: NaiveDateTime) -> f32 {

    const SUMMER_IDX: usize = 0;
    const WINTER_IDX: usize = 1;

    const PEAK: usize = 0;
    const PART_PEAK: usize = 1;
    const OFF_PEAK: usize = 2;

    let energy_rates: Vec<Vec<f32>> = vec![
        vec![0.616, 0.454, 0.397], // Summer rates: [PEAK, PART_PEAK, OFF_PEAK]
        vec![0.384, 0.362, 0.348]  // Winter rates: [PEAK, PART_PEAK, OFF_PEAK]
    ];

    let row_idx =  match start_time.month() {
        6..=9 => SUMMER_IDX,
        _ => WINTER_IDX,
    };

    let col_idx = match start_time.hour() {
        16..=20 => PEAK,
        15 | 21..=23 => PART_PEAK,
        _ => OFF_PEAK,
    };

    energy_rates[row_idx][col_idx] * kWh
}