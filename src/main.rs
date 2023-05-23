use std::fs::File;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use time::OffsetDateTime;

use nvml_wrapper::{error::NvmlError, Nvml, Device};
use nvml::{NvmlTopology, NvmlMeasurement};

use rapl_probes::{perf_event, EnergyProbe};

pub mod nvml;

fn current_datetime_str() -> String {
    let t = OffsetDateTime::now_local().unwrap();
    t.format(&time::format_description::well_known::Rfc3339).unwrap()
}

const PERIOD: Duration = Duration::from_secs(1);

fn main() -> anyhow::Result<()> {
    // sonde NVML
    let nvml = nvml_wrapper::Nvml::init().expect("failed to init nvml, please check your driver");
    let mut nvidia = nvml::NvmlTopology::new(&nvml).expect("something was wrong");
    let intial_time = current_datetime_str();

    // sonde RAPL
    let rapl_cpus = rapl_probes::cpus_to_monitor()?;
    let binding = rapl_probes::perf_event::all_power_events()?;
    let rapl_events: Vec<&perf_event::PowerEvent> = binding.iter().collect();
    let mut rapl = rapl_probes::perf_event::PerfEventProbe::new(&rapl_cpus, &rapl_events)?;
    let mut rapl_measurements = rapl_probes::EnergyMeasurements::new(rapl_cpus.len());

    // création des fichiers
    let nvml_results_path = &format!("results/{intial_time}-nvml.csv");
    let rapl_results_path = &format!("results/{intial_time}-rapl.csv");

    let mut nvml_file = File::create(nvml_results_path).expect(&format!("failed to open file {nvml_results_path}"));
    let mut rapl_file = File::create(rapl_results_path).expect(&format!("failed to open file {rapl_results_path}"));

    // boucle de mesure et construction du CSV au fur et à mesure
    print_csv_header_gpu(&mut nvml_file)?;
    loop {
        let timestamp = SystemTime::now();
        
        // nvml
        let m_gpu = nvidia.fetch_latest_measurement().expect("where are my measurements ?!");
        
        // rapl
        rapl.read_consumed_energy(&mut rapl_measurements)?;
        
        // écriture nvml
        println!("Got: {m_gpu:?}");
        write_csv_gpu(&m_gpu, &timestamp, &mut nvml_file)?;
        nvml_file.flush()?;
        
        // écriture rapl
        // write_csv_cpu(measures, &timestamp, file)
        todo!();
         
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
fn print_csv_header_gpu(file: &mut File) -> io::Result<()> {
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

fn print_csv_header_cpu(file: &mut File) -> io::Result<()> {
    let header = [
        "timestamp",
        "domain",
        "energy_consumption_since_previous_measurement_milliJ",
    ].join(";");
    file.write_all(header.as_bytes())?;
    file.write_all("\n".as_bytes())?;
    Ok(())
}

fn write_csv_gpu(measures: &[NvmlMeasurement], timestamp: &SystemTime, file: &mut File) -> io::Result<()> {
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

fn write_csv_cpu(measures: &[NvmlMeasurement], timestamp: &SystemTime, file: &mut File) -> io::Result<()> {
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