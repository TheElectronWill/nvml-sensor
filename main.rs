fn main() {
    let nvml = nvml_wrapper::Nvml::init().expect("failed to init nvml, please check your driver");
    let mut nvidia = scaphandre::sensors::nvml::NvmlTopology::new(&nvml).expect("something was wrong");
    let test = nvidia.test().expect("Pb with test");
    loop {
        let m = nvidia.fetch_latest_measurement().expect("where are my measurements ?!");
        println!("Got: {m:?}");
        std::thread::sleep_ms(1000);
    }
    println!(":-)");
}