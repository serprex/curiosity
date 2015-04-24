use std::env;
use std::io::{Result, Error, ErrorKind};
use std::path::Path;
use docker;

fn get_docker() -> Result<docker::Docker> {
    let docker_host = match env::var("DOCKER_HOST") {
        Ok(host) => host,
        Err(_) => "unix:///var/run/docker.sock".to_string()
    };

    let docker_cert_path = match env::var("DOCKER_CERT_PATH") {
        Ok(host) => host,
        Err(_) => "".to_string()
    };
    
    let mut docker = match docker::Docker::connect(&docker_host) {
        Ok(docker) => docker,
        Err(e) => {
            println!("{}", e);
            let err = Error::new(ErrorKind::NotConnected,
                                 "The connection is not connected.");
            return Err(err);
        }
    };

    if docker_cert_path != "" {
        let key = Path::new(&docker_cert_path).join("key.pem");
        let cert = Path::new(&docker_cert_path).join("cert.pem");
        let ca = Path::new(&docker_cert_path).join("ca.pem");
        docker.set_tls(true);
        docker.set_private_key_file(&key).unwrap();
        docker.set_certificate_file(&cert).unwrap();
        docker.set_ca_file(&ca).unwrap();
    }
    
    return Ok(docker);
}

pub fn get_containers() -> Result<Vec<docker::container::Container>> {
    let docker = try!(get_docker());
    let containers = match docker.get_containers(true) {
        Ok(containers) => containers,
        Err(e) => {
            println!("{}", e);
            let err = Error::new(ErrorKind::ConnectionAborted,
                                 "A connection is aborted");
            return Err(err);
        }
    };
    return Ok(containers);
}

pub fn get_stats_as_cosmos_container(container: &docker::container::Container) -> Result<Container> {
    let docker = try!(get_docker());
    let stats = match docker.get_stats(container) {
        Ok(stats) => stats,
        Err(e) => {
            println!("{}", e);
            let err = Error::new(ErrorKind::NotConnected,
                                 "A connection is aborted.");
            return Err(err);
        }
    };

    let delayed_stats = match docker.get_stats(container) {
        Ok(stats) => stats,
        Err(e) => {
            println!("{}", e);
            let err = Error::new(ErrorKind::ConnectionAborted,
                                 "A connection is aborted.");
            return Err(err);
        }
    };

    let cosmos_container = container.to_cosmos_container(&stats, &delayed_stats);
    return Ok(cosmos_container);
}

pub fn get_hostname() -> Result<String> {
    let docker = try!(get_docker());
    let hostname = match docker.get_info() {
        Ok(info) => info.Name,
        Err(e) => {
            println!("{}", e);
            let err = Error::new(ErrorKind::NotConnected,
                                 "A connection to Docker is aborted.");
            return Err(err);
        }
    };

    return Ok(hostname);
}

trait CosmosContainerDecodable {
    fn to_cosmos_container(&self,
                           stats: &docker::stats::Stats,
                           delayed_stats: &docker::stats::Stats) -> Container;
}

impl CosmosContainerDecodable for docker::container::Container {
    fn to_cosmos_container(&self,
                           stats: &docker::stats::Stats,
                           delayed_stats: &docker::stats::Stats) -> Container {
        // network
        let network = Network {
            RxBytes: delayed_stats.network.rx_bytes,
            TxBytes: delayed_stats.network.tx_bytes,
            RxBytesDelta: delayed_stats.network.rx_bytes - stats.network.rx_bytes,
            TxBytesDelta: delayed_stats.network.tx_bytes - stats.network.tx_bytes
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
        let system_usage = stats.cpu_stats.system_cpu_usage;
        let delayed_system_usage = delayed_stats.cpu_stats.system_cpu_usage;
        let total_percent = get_cpu_percent(total_usage,
                                            delayed_total_usage,
                                            system_usage,
                                            delayed_system_usage,
                                            cpus);

        let mut percpus: Vec<f64> = Vec::new();
        for i in 0..cpus {
            let val = stats.cpu_stats.cpu_usage.percpu_usage[i];
            let delayed_val = delayed_stats.cpu_stats.cpu_usage.percpu_usage[i];
            let percent = get_cpu_percent(val,
                                          delayed_val,
                                          system_usage,
                                          delayed_system_usage,
                                          cpus);
            percpus.push(percent);
        }

        let cpu = Cpu {
            TotalUtilization: total_percent,
            PerCpuUtilization: percpus
        };

        // stats
        let stats = Stats {
            Network: network,
            Cpu: cpu,
            Memory: memory
        };

        // names
        let mut names: Vec<String> = Vec::new();
        for name in self.Names.iter() {
            let is_contained = name.as_bytes()[0] == "/".as_bytes()[0];
            match is_contained {
                true => {
                    let mut index = 0;
                    let mut new_name: Vec<u8> = Vec::new();
                    for b in name.as_bytes() {
                        index += 1;
                        if index == 1 { continue; }
                        new_name.push(*b);
                    }
                    names.push(String::from_utf8(new_name).unwrap());
                }
                false => { names.push(name.clone()); }
            };
        }

        // container
        let container = Container {
            Id: self.Id.clone(),
            Image: self.Image.clone(),
            Status: self.Status.clone(),
            Command: self.Command.clone(),
            Created: self.Created.clone(),
            Names: names,
            Ports: self.Ports.clone(),
            Stats: stats,
            SizeRw: self.SizeRw,
            SizeRootFs: self.SizeRootFs
        };

        return container;
    }
}

fn get_cpu_percent(cpu_val: u64,
                   delayed_cpu_val: u64,
                   system_val: u64,
                   delayed_system_val: u64,
                   cpus: usize) -> f64 {
    let cpu_val_delta: f64 = (delayed_cpu_val - cpu_val) as f64;
    let system_val_delta: f64 = (delayed_system_val - system_val) as f64;
    let mut percent = (cpu_val_delta / system_val_delta) * cpus as f64 * 100.0 as f64;
    if percent <= 0.0 { percent = 0.0; }
    return percent;
}

#[derive(RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct Container {
    pub Id: String,
    pub Image: String,
    pub Status: String,
    pub Command: String,
    pub Created: u64,
    pub Names: Vec<String>,
    pub Ports: Vec<docker::container::Port>,
    pub Stats: Stats,
    pub SizeRw: u64,
    pub SizeRootFs: u64
}

#[derive(RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct Stats {
    pub Network: Network,
    pub Cpu: Cpu,
    pub Memory: Memory
}

#[derive(RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct Network {
    pub RxBytes: u64,
    pub TxBytes: u64,
    pub RxBytesDelta: u64,
    pub TxBytesDelta: u64
}

#[derive(RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct Cpu {
    pub TotalUtilization: f64,
    pub PerCpuUtilization: Vec<f64>
}

#[derive(RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct Memory {
    pub Limit: u64,
    pub Usage: u64
}

impl Clone for Container {
    fn clone(&self) -> Self {
        let container = Container {
            Id: self.Id.clone(),
            Image: self.Image.clone(),
            Status: self.Status.clone(),
            Command: self.Command.clone(),
            Created: self.Created,
            Names: self.Names.clone(),
            Ports: self.Ports.clone(),
            Stats: self.Stats.clone(),
            SizeRw: self.SizeRw,
            SizeRootFs: self.SizeRootFs
        };
        return container;
    }
}

impl Clone for Stats {
    fn clone(&self) -> Self {
        let stats = Stats {
            Network: self.Network.clone(),
            Cpu: self.Cpu.clone(),
            Memory: self.Memory.clone()
        };
        return stats;
    }
}

impl Clone for Network {
    fn clone(&self) -> Self {
        let network = Network {
            RxBytes: self.RxBytes,
            TxBytes: self.TxBytes,
            RxBytesDelta: self.RxBytesDelta,
            TxBytesDelta: self.TxBytesDelta
        };
        return network;
    }
}

impl Clone for Cpu {
    fn clone(&self) -> Self {
        let cpu = Cpu {
            TotalUtilization: self.TotalUtilization,
            PerCpuUtilization: self.PerCpuUtilization.clone()
        };
        return cpu;
    }
}

impl Clone for Memory {
    fn clone(&self) -> Self {
        let memory = Memory {
            Limit: self.Limit,
            Usage: self.Usage,
        };
        return memory;
    }
}
