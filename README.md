# NVML sensor

Goal: Retrieve data from NVML including:
- Number of device
- Info like brand, architecture, driver version
- instantaneous power and energy consumption since last measurement
- Other metrics like utilization

The sensor deals with overflows. 

# Compile in release mode and run

```
cargo build --release
sudo -E target/release/nvml_sensor --result-dir "results" --period-seconds 1 &

pid=$!
echo "Sensors running with pid $pid"
sleep 10
kill $pid
```
