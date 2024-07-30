# NVML sensor

Small tool that retrieves data from NVML (NVIDIA Management Library), including:
- Number of device
- Info like brand, architecture, driver version
- instantaneous power and energy consumption since last measurement
- Other metrics like utilization

The sensor deals with overflows. All results are saved in CSV files.

CPU consumption is also returned, based on [this minimal yet rigorous RAPL-based measurement tool](https://github.com/TheElectronWill/cpu-energy-consumption-comparative-analysis).

You can think of this tool as a preliminary version of [Alumet](https://github.com/alumet-dev/alumet).

# Install

You need to clone both this repository _and_ the minimal tool.
Rename `cpu-energy-consumption-comparative-analysis` to `rapl-ebpf-experiments`.

## Rust setup

If you haven't installed Rust yet, use Rustup to setup all the necessary tools:
```
curl https://sh.rustup.rs -sSf | sh

# if need to uninstall, do rustup self uninstall
```

## Compile in release mode and run

```
cargo build --release
mkdir "$HOME/nvml-sensors-results"
sudo -E target/release/nvml_sensor --result-dir "$HOME/nvml-sensor-results" --period-seconds 1 &

pid=$!
echo "Sensors running with pid $pid"
sleep 10
kill $pid
```

Note that super-user privileges are required to run the tool.
