use sysinfo::System;

#[derive(Debug)]
pub struct ProcessInfo {
    /// This was a stupid move, change it later
    pub pid: String,
    pub name: String,
    pub memory_mb: f64,
}

pub fn get_system_processes() -> Vec<ProcessInfo> {
    let mut system = System::new_all();
    system.refresh_all();

    system
        .processes()
        .iter()
        .map(|(pid, process)| {
            let memory_mb = process.memory() as f64 / 1024.0 / 1024.0;

            ProcessInfo {
                pid: pid.to_string(),
                name: process.name().to_string_lossy().to_string(),
                memory_mb
            }
        })
        .collect()
}
