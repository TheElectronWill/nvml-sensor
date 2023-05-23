pub mod nvml;
use nvml_wrapper::{error::NvmlError, Nvml, Device};
use nvml::{NvmlTopology};

fn main() {
    let nvml = nvml_wrapper::Nvml::init().expect("failed to init nvml, please check your driver");
    let mut nvidia = nvml::NvmlTopology::new(&nvml).expect("something was wrong");
    loop {
        let m = nvidia.fetch_latest_measurement().expect("where are my measurements ?!");
        println!("Got: {m:?}");
        std::thread::sleep_ms(1000);
    }
    println!(":-)");
}