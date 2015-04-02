use docker;

#[derive(RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct Container {
    Id: String,
    Image: String,
    Status: String,
    Command: String,
    Created: f64,
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
    RxBytes: f64,
    TxBytes: f64
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
    Limit: f64,
    Usage: f64
}

pub trait CosmosContainerDecodable {
    fn to_cosmos_container(&self, stats: &docker::stats::Stats) -> Container;
}

impl CosmosContainerDecodable for docker::container::Container {
    fn to_cosmos_container(&self, stats: &docker::stats::Stats) -> Container {
        let network = Network {
            RxBytes: stats.network.rx_bytes,
            TxBytes: stats.network.tx_bytes
        };

        let cpu = Cpu {
            TotalUtilization: stats.cpu_stats.cpu_usage.total_usage, // fix value
            PerCpuUtilization: stats.cpu_stats.cpu_usage.percpu_usage.clone()
        };

        let memory = Memory {
            Limit: stats.memory_stats.limit,
            Usage: stats.memory_stats.usage
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
