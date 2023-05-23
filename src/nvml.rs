use nvml_wrapper::{error::NvmlError, Nvml, Device, struct_wrappers::device::{Utilization, ProcessInfo}, structs::device::UtilizationInfo};

// Like Topology but for nvidia GPU
// TODO: add that to the usual `Topology`, in `Option<NvmlTopology>` or something like that.
// TODO: in the `MetricGenerator`, use that to push new metrics: instant energy and consumption in milli Joules since last time
// TODO: push the name of the GPU, number of devices, power limit, gpu usage, etc.
// (TODO): try to get the processes on the GPU and assign them a part of the GPU's consumption
pub struct NvmlTopology<'a> {
    devices: Vec<Device<'a>>,
    previous_measurement: Vec<u64>,
}

#[derive(Debug)]
pub struct NvmlMeasurement {
    pub device_index: u32,
    pub consumption_millij: u64,
    pub instantaneous_power: u32,
    // utilization between 0 and 100, and is valid for the last time period
    pub utilization: Utilization,
    pub util_decoder: UtilizationInfo,
    pub util_encoder: UtilizationInfo,
    pub compute_processes: Vec<ProcessInfo>,
    pub graphic_processes: Vec<ProcessInfo>,
}

impl<'a> NvmlTopology<'a> {
    pub fn new(nvml: &'a Nvml) -> Result<NvmlTopology<'a>, NvmlError> {
        let gpu_count = nvml.device_count()?;
        // find all the GPUs
        let mut devices = Vec::new();
        for i in 0..gpu_count {
            println!("Found device {i}");
            let d = nvml.device_by_index(i)?;
            devices.push(d);
        }
        // create the sensor with all the last measurements at zero
        let sensor = NvmlTopology { devices, previous_measurement: vec![0; gpu_count as usize] };
        Ok(sensor)
    }

    pub fn refresh(&mut self) {
        todo!()
    }

    pub fn fetch_latest_measurement(&mut self) -> Result<Vec<NvmlMeasurement>, NvmlError> {
        let mut measurements = Vec::new();
        let mut new_previous_measurements = Vec::new();
        for (index, device) in self.devices.iter().enumerate() {
            let (energy_diff, energy_total) = self.compute_energy_diff(index, device)?;
            let point = NvmlMeasurement {
                device_index: device.index()?,
                consumption_millij: energy_diff,
                instantaneous_power: device.power_usage()?,
                utilization: device.utilization_rates()?,
                util_decoder: device.decoder_utilization()?,
                util_encoder: device.encoder_utilization()?,
                compute_processes: get_compute_processes(device)?,
                graphic_processes: get_graphic_processes(device)?,
            };
            measurements.push(point);
            new_previous_measurements.push(energy_total);
        }
        self.previous_measurement = new_previous_measurements;
        Ok(measurements)
    }

    fn compute_energy_diff(&self, index: usize, device: &Device) -> Result<(u64, u64), NvmlError> {
        let energy_consumption = device.total_energy_consumption()?;
        let previous_consumption = self.previous_measurement[index];
        let res = if previous_consumption > energy_consumption {
            u64::MAX - previous_consumption + energy_consumption
        } else {
            energy_consumption - previous_consumption
        };
        Ok((res, energy_consumption))
    }

}

fn get_compute_processes(device: &Device) -> Result<Vec<ProcessInfo>, NvmlError> {
    match device.running_compute_processes() {
        Ok(res) => Ok(res),
        Err(NvmlError::FailedToLoadSymbol(_)) => {
            device.running_compute_processes_v2()
        },
        Err(e) => Err(e)
    }
}

fn get_graphic_processes(device: &Device) -> Result<Vec<ProcessInfo>, NvmlError> {
    match device.running_graphics_processes() {
        Ok(res) => Ok(res),
        Err(NvmlError::FailedToLoadSymbol(_)) => {
            device.running_graphics_processes_v2()
        },
        Err(e) => Err(e)
    }
}

pub fn test() -> Result<(), NvmlError> {
    let nvml = Nvml::init()?;
    let gpu_count = nvml.device_count()?;
    for i in 0..dbg!(gpu_count) {
        println!("Found device {i}");
        let device = nvml.device_by_index(i)?;
        let brand = device.brand()?; // GeForce on my system
        // let info = device.pci_info()?;

        let arch = device.architecture()?;
        let driver_version = nvml.sys_driver_version()?;

        let power_usage = device.power_usage()?;
        let total_energy_consumption = device.total_energy_consumption()?;
        let fan_speed = device.fan_speed(0)?; // Currently 17% on my system
        let power_limit = device.enforced_power_limit()?; // 275k milliwatts on my system
        let memory_info = device.memory_info()?; // Currently 1.63/6.37 GB used on my system

        println!("== GPU {brand:?} {arch}, driver {driver_version} ==");
        println!("fan speed = {fan_speed}");
        println!("memory = {memory_info:?}");
        println!("power: {power_usage} (usage) / {power_limit} (limit)");
        println!("Energy consumed since last driver reload: {total_energy_consumption} (mJ)");


    }
    Ok(())
}