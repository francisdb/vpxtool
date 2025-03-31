use bevy::diagnostic::DiagnosticPath;
use bevy::prelude::*;

/// Adds a System Information Diagnostic, specifically `cpu_usage` (in %) and `mem_usage` (in %)
///
/// Note that gathering system information is a time intensive task and therefore can't be done on every frame.
/// Any system diagnostics gathered by this plugin may not be current when you access them.
///
/// Supported targets:
/// * linux,
/// * windows,
/// * android,
/// * macOS
///
/// NOT supported when using the `bevy/dynamic` feature even when using previously mentioned targets
///
/// # See also
///
/// [`LogDiagnosticsPlugin`](crate::LogDiagnosticsPlugin) to output diagnostics to the console.
#[derive(Default)]
pub struct SystemInformationDiagnosticsPlugin;
impl Plugin for SystemInformationDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        internal::setup_plugin(app);
    }
}

impl SystemInformationDiagnosticsPlugin {
    /// Total system cpu usage in %
    pub const CPU_USAGE: DiagnosticPath = DiagnosticPath::const_new("system/cpu_usage");
    /// Total system memory usage in %
    pub const MEM_USAGE: DiagnosticPath = DiagnosticPath::const_new("system/mem_usage");
    /// Current process cpu usage in %
    pub const PROCESS_CPU_USAGE: DiagnosticPath = DiagnosticPath::const_new("process/cpu_usage");
    /// Current process memory usage in MB
    pub const PROCESS_MEM_USAGE: DiagnosticPath = DiagnosticPath::const_new("process/mem_usage");
}

/// A resource that stores diagnostic information about the system.
/// This information can be useful for debugging and profiling purposes.
///
/// # See also
///
/// [`SystemInformationDiagnosticsPlugin`] for more information.
#[derive(Debug, Resource)]
#[allow(unused)]
pub struct SystemInfo {
    pub os: String,
    pub kernel: String,
    pub cpu: String,
    pub core_count: String,
    pub memory: String,
}

pub mod internal {
    use bevy::app::{App, First, Startup, Update};
    use bevy::diagnostic::{Diagnostic, Diagnostics, DiagnosticsStore};
    use bevy::ecs::system::Resource;
    use bevy::ecs::{prelude::ResMut, system::Local};
    use bevy::tasks::{AsyncComputeTaskPool, Task, available_parallelism, block_on, poll_once};
    use bevy::utils::tracing::info;
    use std::sync::Arc;
    use std::{sync::Mutex, time::Instant};
    use sysinfo::{CpuRefreshKind, MemoryRefreshKind, ProcessesToUpdate, RefreshKind, System};

    use super::{SystemInfo, SystemInformationDiagnosticsPlugin};

    const BYTES_TO_GIB: f64 = 1.0 / 1024.0 / 1024.0 / 1024.0;

    pub(super) fn setup_plugin(app: &mut App) {
        app.add_systems(Startup, setup_system)
            .add_systems(First, launch_diagnostic_tasks)
            .add_systems(Update, read_diagnostic_tasks)
            .init_resource::<SysinfoTasks>();
    }

    fn setup_system(mut diagnostics: ResMut<DiagnosticsStore>) {
        diagnostics
            .add(Diagnostic::new(SystemInformationDiagnosticsPlugin::CPU_USAGE).with_suffix("%"));
        diagnostics
            .add(Diagnostic::new(SystemInformationDiagnosticsPlugin::MEM_USAGE).with_suffix("%"));
        diagnostics.add(
            Diagnostic::new(SystemInformationDiagnosticsPlugin::PROCESS_CPU_USAGE).with_suffix("%"),
        );
        diagnostics.add(
            Diagnostic::new(SystemInformationDiagnosticsPlugin::PROCESS_MEM_USAGE)
                .with_suffix("MB"),
        );
    }

    struct SysinfoRefreshData {
        current_cpu_usage: f64,
        current_used_mem: f64,
        process_cpu_usage: f64,
        process_mem_usage: f64, // in MB
    }

    #[derive(Resource, Default)]
    struct SysinfoTasks {
        tasks: Vec<Task<SysinfoRefreshData>>,
    }

    fn launch_diagnostic_tasks(
        mut tasks: ResMut<SysinfoTasks>,
        // TODO: Consider a fair mutex
        mut sysinfo: Local<Option<Arc<Mutex<System>>>>,
        // TODO: FromWorld for Instant?
        mut last_refresh: Local<Option<Instant>>,
    ) {
        let sysinfo = sysinfo.get_or_insert_with(|| {
            Arc::new(Mutex::new(System::new_with_specifics(
                RefreshKind::nothing()
                    .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
                    .with_memory(MemoryRefreshKind::everything()),
            )))
        });

        let last_refresh = last_refresh.get_or_insert_with(Instant::now);

        let thread_pool = AsyncComputeTaskPool::get();

        // Only queue a new system refresh task when necessary
        // Queuing earlier than that will not give new data
        if last_refresh.elapsed() > sysinfo::MINIMUM_CPU_UPDATE_INTERVAL
            // These tasks don't yield and will take up all of the task pool's
            // threads if we don't limit their amount.
            && tasks.tasks.len() * 2 < available_parallelism()
        {
            let sys = Arc::clone(sysinfo);
            let task = thread_pool.spawn(async move {
                let mut sys = sys.lock().unwrap();

                sys.refresh_cpu_specifics(CpuRefreshKind::nothing().with_cpu_usage());
                sys.refresh_memory();
                let current_cpu_usage = sys.global_cpu_usage().into();
                // `memory()` fns return a value in bytes
                let total_mem = sys.total_memory() as f64 / BYTES_TO_GIB;
                let used_mem = sys.used_memory() as f64 / BYTES_TO_GIB;
                let current_used_mem = used_mem / total_mem * 100.0;

                let pid = sysinfo::get_current_pid().expect("Failed to get current process ID");
                sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);

                let process_mem_usage = sys
                    .process(pid)
                    .map(|p| p.memory() as f64 / 1024.0 / 1024.0) // Convert to MB
                    .unwrap_or(0.0);

                let process_cpu_usage = sys
                    .process(pid)
                    .map(|p| p.cpu_usage() as f64)
                    .unwrap_or(0.0);

                SysinfoRefreshData {
                    current_cpu_usage,
                    current_used_mem,
                    process_cpu_usage,
                    process_mem_usage,
                }
            });
            tasks.tasks.push(task);
            *last_refresh = Instant::now();
        }
    }

    fn read_diagnostic_tasks(mut diagnostics: Diagnostics, mut tasks: ResMut<SysinfoTasks>) {
        tasks.tasks.retain_mut(|task| {
            let Some(data) = block_on(poll_once(task)) else {
                return true;
            };

            diagnostics.add_measurement(&SystemInformationDiagnosticsPlugin::CPU_USAGE, || {
                data.current_cpu_usage
            });
            diagnostics.add_measurement(&SystemInformationDiagnosticsPlugin::MEM_USAGE, || {
                data.current_used_mem
            });
            diagnostics.add_measurement(
                &SystemInformationDiagnosticsPlugin::PROCESS_CPU_USAGE,
                || data.process_cpu_usage,
            );
            diagnostics.add_measurement(
                &SystemInformationDiagnosticsPlugin::PROCESS_MEM_USAGE,
                || data.process_mem_usage,
            );
            false
        });
    }

    impl Default for SystemInfo {
        fn default() -> Self {
            let sys = System::new_with_specifics(
                RefreshKind::nothing()
                    .with_cpu(CpuRefreshKind::everything())
                    .with_memory(MemoryRefreshKind::nothing().with_ram()),
            );

            let system_info = SystemInfo {
                os: System::long_os_version().unwrap_or_else(|| String::from("not available")),
                kernel: System::kernel_version().unwrap_or_else(|| String::from("not available")),
                cpu: sys
                    .cpus()
                    .first()
                    .map(|cpu| cpu.brand().trim().to_string())
                    .unwrap_or_else(|| String::from("not available")),
                core_count: System::physical_core_count()
                    .map(|x| x.to_string())
                    .unwrap_or_else(|| String::from("not available")),
                // Convert from Bytes to GibiBytes since it's probably what people expect most of the time
                memory: format!("{:.1} GiB", sys.total_memory() as f64 * BYTES_TO_GIB),
            };

            info!("{:?}", system_info);
            system_info
        }
    }
}
