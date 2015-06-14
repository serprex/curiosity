use std::env;
use std::io::{self, Result, ErrorKind};
use std::path::Path;
use docker;
use cosmos;

pub fn get_docker() -> Result<docker::Docker> {
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
        Err(_) => {
            let err = io::Error::new(ErrorKind::NotConnected,
                                 "The connection is not connected with DOCKER_HOST.");
            return Err(err);
        }
    };

    if docker_cert_path != "" {
        let key = Path::new(&docker_cert_path).join("key.pem");
        let cert = Path::new(&docker_cert_path).join("cert.pem");
        let ca = Path::new(&docker_cert_path).join("ca.pem");
        match docker.set_tls(&key, &cert, &ca) {
            Ok(_) => {},
            Err(_) => {
                let err = io::Error::new(ErrorKind::NotConnected,
                                         "The connection is not connected with DOCKER_CERT_PATH.");
                return Err(err);
            }
        }
    }

    return Ok(docker);
}

pub fn get_containers(docker: &docker::Docker) -> Result<Vec<docker::container::Container>> {
    let containers = match docker.get_containers(false) {
        Ok(containers) => containers,
        Err(_) => {
            let err = io::Error::new(ErrorKind::ConnectionAborted,
                                 "Can not get containers.");
            return Err(err);
        }
    };
    return Ok(containers);
}

pub fn get_stats_as_cosmos_container(docker: &docker::Docker, container: &docker::container::Container) -> Result<cosmos::Container> {
    let stats = match docker.get_stats(container) {
        Ok(stats) => stats,
        Err(_) => {
            let err = io::Error::new(ErrorKind::NotConnected,
                                 "Can not get stats of container.");
            return Err(err);
        }
    };
    let delayed_stats = match docker.get_stats(container) {
        Ok(stats) => stats,
        Err(_) => {
            let err = io::Error::new(ErrorKind::ConnectionAborted,
                                 "Can not get stats of container.");
            return Err(err);
        }
    };
    let cosmos_container = container.to_cosmos_container(&stats, &delayed_stats);
    return Ok(cosmos_container);
}

pub fn get_hostname() -> Result<String> {
    let docker = try!(get_docker());
    let hostname = match docker.get_system_info() {
        Ok(system_info) => system_info.Name,
        Err(_) => {
            let err = io::Error::new(ErrorKind::NotConnected,
                                     "Can not get hostname of docker system info.");
            return Err(err);
        }
    };
    return Ok(hostname);
}

trait CosmosContainerDecodable {
    fn to_cosmos_container(&self,
                           stats: &docker::stats::Stats,
                           delayed_stats: &docker::stats::Stats) -> cosmos::Container;
}

impl CosmosContainerDecodable for docker::container::Container {
    fn to_cosmos_container(&self,
                           stats: &docker::stats::Stats,
                           delayed_stats: &docker::stats::Stats) -> cosmos::Container {
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

        /*let mut percpus: Vec<f64> = Vec::new();
        for i in 0..cpus {
            let val = stats.cpu_stats.cpu_usage.percpu_usage[i];
            let delayed_val = delayed_stats.cpu_stats.cpu_usage.percpu_usage[i];
            let percent = get_cpu_percent(val,
                                          delayed_val,
                                          system_usage,
                                          delayed_system_usage,
                                          cpus);
            percpus.push(percent);
        }*/

        let name: Vec<&str> = self.Names[0].split("/").collect();

        let container = cosmos::Container {
            Container: name[name.len() - 1].to_string(),
            Cpu: total_percent as f32,
            Memory: delayed_stats.memory_stats.usage
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
