use docker;

#[derive(RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct Container {
    Id: String,
    Image: String,
    Status: String,
    Command: String,
    Created: i64,
    Names: Vec<String>,
    Ports: Vec<docker::container::Port>,
    Stats: Stats
}

#[derive(RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct Stats {
    Network: Network,
    Cpu: Cpu,
    Memory: Memory
}

#[derive(RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct Network {
    RxBytes: i64,
    TxBytes: i64
}

#[derive(RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct Cpu {
    TotalUtilization: f64,
    PerCpuUtilization: Vec<f64>
}

#[derive(RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct Memory {
    Limit: i64,
    Usage: i64
}

pub trait CosmosContainerDecodable {
    fn to_cosmos_container(&self,
                           stats: &docker::stats::Stats,
                           delayed_stats: &docker::stats::Stats,
                           interval: i64) -> Container;
}

impl CosmosContainerDecodable for docker::container::Container {
    fn to_cosmos_container(&self,
                           stats: &docker::stats::Stats,
                           delayed_stats: &docker::stats::Stats,
                           interval: i64) -> Container {
        // network
        let network = Network {
            RxBytes: delayed_stats.network.rx_bytes,
            TxBytes: delayed_stats.network.tx_bytes
        };

        // memory
        let memory = Memory {
            Limit: delayed_stats.memory_stats.limit,
            Usage: delayed_stats.memory_stats.usage
        };

        // cpu
        let cpus = stats.cpu_stats.cpu_usage.percpu_usage.len();
        let total_usage = stats.cpu_stats.cpu_usage.total_usage;
        let delayed_total_usage = delayed_stats.cpu_stats.cpu_usage.total_usage;
        let total_percent = get_percent(total_usage, delayed_total_usage, interval, cpus);

        let mut percpus: Vec<f64> = Vec::new();
        for i in 0..cpus {
            let val = stats.cpu_stats.cpu_usage.percpu_usage[i];
            let delayed_val = delayed_stats.cpu_stats.cpu_usage.percpu_usage[i];
            let percent = get_percent(val, delayed_val, interval, cpus);
            percpus.push(percent);
        }

        let cpu = Cpu {
            TotalUtilization: total_percent,
            PerCpuUtilization: percpus
        };

        let stats = Stats {
            Network: network,
            Cpu: cpu,
            Memory: memory
        };

        let container = Container {
            Id: self.Id.clone(),
            Image: self.Image.clone(),
            Status: self.Status.clone(),
            Command: self.Command.clone(),
            Created: self.Created.clone(),
            Names: self.Names.clone(),
            Ports: self.Ports.clone(),
            Stats: stats
        };

        return container;
    }
}

fn get_percent(val: i64, delayed_val: i64, interval: i64, cpus: usize) -> f64 {
    let delta = delayed_val - val;
    let dpns = (delta - interval) as f64;
    let dps = dpns / (1000000000 as f64);
    let mut percent = dps / (cpus as f64);
    if percent <= 0.0 { percent = 0.0; }
    return percent;
}
