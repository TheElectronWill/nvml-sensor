# NVML sensor

Goal: Retrieve data from NVML including:
- Number of device
- Info like brand, architecture, driver version
- instantaneous power and energy consumption since last measurement

The sensor needs to deal with overflows. 

## Todo
- utilisation
- compute energy by process

## State of work
### Dealing with processes
On chifflet neither `running_compute_processes` or `running_compute_processes_v2` work. I get a panic error:
```
thread 'main' panicked at 'Pb with test: FailedToLoadSymbol("/lib/x86_64-linux-gnu/libnvidia-ml.so: undefined symbol: nvmlDeviceGetComputeRunningProcesses_v3")', src/main.rs:84:30
```
or a compilation error:
```
error[E0599]: no method named `running_compute_processes_count_v2` found for struct `Device` in the current scope
```
I manage to match the result with ok or Err but I can't match the error with FailedToLoadSymbol.
