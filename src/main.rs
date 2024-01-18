use std::fs::File;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use sysinfo::{System, SystemExt, CpuExt};
use time::OffsetDateTime;
use std::env;

use nvml::NvmlMeasurement;

use rapl_probes::{perf_event, EnergyProbe};
use clap::Parser;

pub mod nvml;

fn current_datetime_str() -> String {
    let t = OffsetDateTime::now_local().unwrap();
    t.format(&time::format_description::well_known::Rfc3339).unwrap()
}

const GPU_CSV_HEADER: &[&str] = &[
    "timestamp",
    "device_index",
    "energy_consumption_since_previous_measurement_milliJ",
    "instantaneous_power_milliW",
    "global_utilization_percent",
    "global_memory_percent"
];

const CPU_CSV_HEADER: &[&str] = &[
        "timestamp",
        "domain",
        "socket",
        "energy_consumption_since_previous_measurement_milliJ",
];

const SYSINFO_CSV_HEADER: &[&str] = &[
        "timestamp",
        "cpu",
        "utilization_percent",
];

#[derive(Parser)]
#[command(about)]
struct Arguments {
    #[arg(short, long, default_value_t = 1.0)]
    period_seconds: f64,

    #[arg(short, long, default_value_t = String::from("results"))]
    result_dir: String
}

fn main() -> anyhow::Result<()> {
    env::set_var("RUST_BACKTRACE", "1");

    let args = Arguments::parse();
    let period = Duration::from_secs_f64(args.period_seconds);
    if period <= Duration::from_secs_f64(0.1) {
        println!("WARNING: You have chosen a small measurement period, but this tool (like many others) is using the standard sleep function for timing, which is not accurate enough.");
        println!("WARNING: You may experience incorrect/inaccurate delays between each measurement.");
        println!("WARNING (solution): Please use ALUMET instead in order to get a precise timing mechanism. You can ask Guillaume for that :)")
    }

    // sonde NVML
    let nvml = nvml_wrapper::Nvml::init().expect("Failed to init nvml, please check your driver (do you have a desktop/server nvidia GPU?)");
    let mut nvidia = nvml::NvmlTopology::new(&nvml).expect("something was wrong");
    let intial_time = current_datetime_str();

    // sonde RAPL
    let rapl_cpus = rapl_probes::cpus_to_monitor().expect("cpu cores should be available in powercap interface");
    let binding = rapl_probes::perf_event::all_power_events().expect("perf events should be accessible");
    let rapl_events: Vec<&perf_event::PowerEvent> = binding.iter().collect();
    let mut rapl = rapl_probes::perf_event::PerfEventProbe::new(&rapl_cpus, &rapl_events).expect("failed to create rapl probe");

    // SysInfo
    let mut sys = System::new();

    // création des fichiers
    let dir = args.result_dir;
    let nvml_results_path = &format!("{dir}/{intial_time}-nvml.csv");
    let rapl_results_path = &format!("{dir}/{intial_time}-rapl.csv");
    let sysinfo_results_path = &format!("{dir}/{intial_time}-sysinfo.csv");

    let mut nvml_file = File::create(nvml_results_path).expect(&format!("failed to create file {nvml_results_path}"));
    let mut rapl_file = File::create(rapl_results_path).expect(&format!("failed to create file {rapl_results_path}"));
    let mut sysinfo_file = File::create(sysinfo_results_path).expect(&format!("failed to create file {sysinfo_results_path}"));

    // boucle de mesure et construction du CSV au fur et à mesure
    print_csv_header(&mut nvml_file, GPU_CSV_HEADER)?;
    print_csv_header(&mut rapl_file, CPU_CSV_HEADER)?;
    print_csv_header(&mut sysinfo_file, SYSINFO_CSV_HEADER)?;
    loop {
        let timestamp = SystemTime::now();

        // nvml
        let m_gpu = nvidia.fetch_latest_measurement().expect("Where are my NVML measurements >_< ?!");

        // rapl
        rapl.poll().expect("RAPL did not respond appropriately to my request, my heart is broken T_T");
        let rapl_measurements = rapl.measurements();

        // cpu utilization
        sys.refresh_cpu();

        // écriture nvml
        write_csv_gpu(&m_gpu, &timestamp, &mut nvml_file)?;
        nvml_file.flush()?;

        // écriture rapl
        write_csv_cpu(&rapl_measurements, &timestamp, &mut rapl_file)?;
        rapl_file.flush()?;

        // écriture sysinfo
        write_csv_sysinfo(&mut sys, &timestamp, &mut sysinfo_file)?;
        sysinfo_file.flush()?;

        // sleep
        std::thread::sleep(period);
    }
}

fn print_csv_header(file: &mut File, header: &[&str]) -> io::Result<()> {
    file.write_all(header.join(";").as_bytes())?;
    file.write_all("\n".as_bytes())?;
    Ok(())
}

fn write_csv_gpu(measures: &[NvmlMeasurement], timestamp: &SystemTime, file: &mut File) -> io::Result<()> {
    let timestamp_unix = timestamp.duration_since(UNIX_EPOCH).expect("time went backwards ?!").as_millis();
    for measure in measures {
        let energy_consumption = measure.consumption_millij;
        let device_index = measure.device_index;
        let inst_power = measure.instantaneous_power.map(|p| p.to_string()).unwrap_or_else(|| String::from("ø"));
        let gpu_percent = measure.utilization.as_ref().map(|u| u.gpu.to_string()).unwrap_or_else(|| String::from("ø"));
        let memory_percent = measure.utilization.as_ref().map(|u| u.memory.to_string()).unwrap_or_else(|| String::from("ø"));
        let line = format!("{timestamp_unix};{device_index};{energy_consumption};{inst_power};{gpu_percent};{memory_percent}\n");
        file.write_all(line.as_bytes())?;
    }
    Ok(())
}

fn write_csv_cpu(measures: &rapl_probes::EnergyMeasurements, timestamp: &SystemTime, file: &mut File) -> io::Result<()> {
    let timestamp_unix = timestamp.duration_since(UNIX_EPOCH).expect("time went backwards ?!").as_millis();
    for (socket_id, domains) in measures.per_socket.iter().enumerate() {
        for (domain_id, measure) in domains {
            if let Some(joules) = measure.joules {
                let energy_consumption_since_previous_measurement_millijoules = joules*1000.0;
                let line = format!("{timestamp_unix};{domain_id:?};{socket_id};{energy_consumption_since_previous_measurement_millijoules}\n");
                file.write_all(line.as_bytes())?;
            }

        }
    }
    Ok(())
}

fn write_csv_sysinfo(sys: &mut System, timestamp: &SystemTime, file: &mut File) -> io::Result<()> {
    let timestamp_unix = timestamp.duration_since(UNIX_EPOCH).expect("time went backwards ?!").as_millis();
    for cpu in sys.cpus() {
        let cpu_util = cpu.cpu_usage();
        let cpu_id = cpu.name();
        let line = format!("{timestamp_unix};{cpu_id};{cpu_util}\n");
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