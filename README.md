# NVML sensor

Goal: Retrieve data from NVML including:
- Number of device
- Info like brand, architecture, driver version
- instantaneous power and energy consumption since last measurement

The sensor needs to deal with overflows. 

# Compile in release mode and run

```
cargo build --release
sudo -E target/release/nvml_sensors ARGS... &

pid=$!
echo "Sensors running with pid $pid"
sleep 10
kill $pid
```
