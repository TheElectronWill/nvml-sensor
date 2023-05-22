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
### Creating a rust appli
Je me suis un peu perdue dans l'idée d'utiliser Cargo. Voilà là où j'en suis :
```
error[E0432]: unresolved import `nvml_sensor::NvmlTopology`
 --> src/main.rs:3:19
  |
3 | use nvml_sensor::{NvmlTopology};
  |                   ^^^^^^^^^^^^ no `NvmlTopology` in the root
```

### Dealing with processes
On chifflet neither `running_compute_processes` or `running_compute_processes_v2` work. I get a panic error:
```
thread 'main' panicked at 'Pb with test: FailedToLoadSymbol("/lib/x86_64-linux-gnu/libnvidia-ml.so: undefined symbol: nvmlDeviceGetComputeRunningProcesses_v3")', src/main.rs:84:30
```
or a compilation error:
```
error[E0599]: no method named `running_compute_processes_count_v2` found for struct `Device` in the current scope
```
I manage to match the result with FailedToLoadSymbol. I haven't started working on the compilation error.
