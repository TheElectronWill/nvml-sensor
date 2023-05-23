pub mod nvml;

use nvml_wrapper::{error::NvmlError, Nvml, Device};
use nvml::{NvmlTopology, NvmlMeasurement};
use std::fs::File;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use time::OffsetDateTime;

// TODO
// https://docs.rs/nvml-wrapper/latest/nvml_wrapper/device/struct.Device.html#method.process_utilization_stats

fn current_datetime_str() -> String {
    let t = OffsetDateTime::now_local().unwrap();
    t.format(&time::format_description::well_known::Rfc3339).unwrap()
}

const PERIOD: Duration = Duration::from_secs(1);

fn main() -> io::Result<()> {
    let nvml = nvml_wrapper::Nvml::init().expect("failed to init nvml, please check your driver");
    let mut nvidia = nvml::NvmlTopology::new(&nvml).expect("something was wrong");
    let intial_time = current_datetime_str();

    let nvml_results_path = &format!("results/{intial_time}-nvml.csv");
    let rapl_results_path = &format!("results/{intial_time}-rapl.csv");

    let mut nvml_file = File::create(nvml_results_path).expect(&format!("failed to open file {nvml_results_path}"));
    let mut rapl_file = File::create(rapl_results_path).expect(&format!("failed to open file {rapl_results_path}"));

    print_csv_header(&mut nvml_file)?;
    loop {
        let timestamp = SystemTime::now();
        let m = nvidia.fetch_latest_measurement().expect("where are my measurements ?!");
        println!("Got: {m:?}");
        write_csv(&m, &timestamp, &mut nvml_file)?;
        nvml_file.flush()?;
        std::thread::sleep(PERIOD);
    }
}

/// ## CSV columns
/// - timestamp
/// - device_index
/// - energy_consumption_since_previous_measurement_milliJ
/// - instantaneous_power_milliW
/// - global_utilization_percent
/// - global_memory_percent
fn print_csv_header(file: &mut File) -> io::Result<()> {
    let header = [
        "timestamp",
        "device_index",
        "energy_consumption_since_previous_measurement_milliJ",
        "instantaneous_power_milliW",
        "global_utilization_percent",
        "global_memory_percent"
    ].join(";");
    file.write_all(header.as_bytes())?;
    file.write_all("\n".as_bytes())?;
    Ok(())
}

fn write_csv(measures: &[NvmlMeasurement], timestamp: &SystemTime, file: &mut File) -> io::Result<()> {
    let timestamp_unix = timestamp.duration_since(UNIX_EPOCH).expect("time went backwards ?!").as_millis();
    for measure in measures {
        let energy_consumption = measure.consumption_millij;
        let device_index = measure.device_index;
        let inst_power = measure.instantaneous_power;
        let gpu_percent = measure.utilization.gpu;
        let memory_percent = measure.utilization.memory;
        let line = format!("{timestamp_unix};{device_index};{energy_consumption};{inst_power};{gpu_percent};{memory_percent}\n");
        file.write_all(line.as_bytes())?;
    }
    Ok(())
}

/*
[NvmlMeasurement { 
    
    device_index: 0, 
    consumption_millij: 93920559, 
    instantaneous_power: 8612, 
    utilization: Utilization { gpu: 0, memory: 0 }, 
    util_decoder: UtilizationInfo { utilization: 0, sampling_period: 167000 }, 
    util_encoder: UtilizationInfo { utilization: 0, sampling_period: 167000 }, c
    compute_processes: [], 
    graphic_processes: [] }
    
    , 
    
NvmlMeasurement
*/