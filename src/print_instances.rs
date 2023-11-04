use crate::describe_instance;
use crate::get_integer_with_len;
use crate::get_region_config;
use crate::get_string_network_and_len;
use crate::get_string_with_dot_and_len;
use crate::get_string_with_len;
use crate::get_string_with_len_and_padding;
use anyhow::Result;
use aws_sdk_ec2::types::ArchitectureType;
use aws_sdk_ec2::types::InstanceType;
use aws_sdk_ec2::types::InstanceTypeInfo;
use aws_sdk_ec2::types::NetworkInfo;
use std::cmp::Ordering;

fn get_nr_of_efas(info: &NetworkInfo) -> i32 {
    if let Some(efa_info) = info.efa_info() {
        efa_info.maximum_efa_interfaces().unwrap()
    } else {
        0
    }
}

fn get_architecture(archs: &[ArchitectureType]) -> String {
    for arch in archs {
        match arch {
            ArchitectureType::Arm64 => {
                return "Arm64".to_string();
            }
            ArchitectureType::I386 | ArchitectureType::X8664 | ArchitectureType::X8664Mac => {
                return "X86".to_string();
            }
            _ => {
                //return "unknown".to_string();
            }
        }
    }
    "unknown".to_string()
}

#[derive(Clone)]
struct Gpu {
    manufacturer: String,
    name: String,
    count: i32,
    memory: i32,
}

impl Gpu {
    fn new(manufacturer: &str, name: &str, count: i32, memory: i32) -> Self {
        Self {
            manufacturer: manufacturer.to_string(),
            name: name.to_string(),
            count,
            memory,
        }
    }

    fn manufacturer(&self) -> &str {
        &self.manufacturer
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn count(&self) -> i32 {
        self.count
    }

    fn memory(&self) -> i32 {
        self.memory
    }
}

#[derive(Clone)]
struct InstanceStorage {
    gb: Option<i64>,
}

impl InstanceStorage {
    fn new(gb: Option<i64>) -> Self {
        Self { gb }
    }
}

struct Network {
    network_performance: String,
    efa: i32,
}

impl Network {
    fn new(network_performance: &str, efa: i32) -> Self {
        Self {
            network_performance: network_performance.to_string(),
            efa,
        }
    }
}

struct Cpu {
    name: InstanceType,
    arch: String,
    cores: i32,
    memory: i64,
}

impl Cpu {
    fn new(name: InstanceType, arch: &str, cores: i32, memory: i64) -> Self {
        Self {
            name,
            arch: arch.to_string(),
            cores,
            memory,
        }
    }
}

struct Instance {
    cpu: Cpu,
    network: Network,
    ebs: i32,
    gpus: Vec<Gpu>,
    instance_storage: InstanceStorage,
}

impl Instance {
    fn new(
        cpu: Cpu,
        network: Network,
        ebs: i32,
        gpus: &[Gpu],
        instance_storage: InstanceStorage,
    ) -> Self {
        Self {
            cpu,
            network,
            ebs,
            gpus: gpus.to_vec(),
            instance_storage,
        }
    }

    fn name(&self) -> &str {
        self.cpu.name.as_str()
    }

    fn arch(&self) -> &str {
        &self.cpu.arch
    }

    fn cores(&self) -> i32 {
        self.cpu.cores
    }

    fn memory(&self) -> i64 {
        self.cpu.memory
    }

    fn network_performance(&self) -> String {
        self.network.network_performance.clone()
    }

    fn get_efas(&self) -> i32 {
        self.network.efa
    }

    fn ebs(&self) -> i32 {
        self.ebs
    }

    fn gpu_len(&self) -> usize {
        self.gpus.len()
    }

    fn gpus(&self) -> &[Gpu] {
        &self.gpus
    }

    fn instance_storage(&self) -> InstanceStorage {
        self.instance_storage.clone()
    }
}

impl Ord for Instance {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.cores() < other.cores() {
            return Ordering::Less;
        }
        if self.cores() > other.cores() {
            return Ordering::Greater;
        }
        assert!(self.cores() == other.cores());
        if self.memory() < other.memory() {
            return Ordering::Less;
        }
        if self.memory() > other.memory() {
            return Ordering::Greater;
        }
        Ordering::Equal
    }
}

impl PartialOrd for Instance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Instance {
    fn eq(&self, other: &Self) -> bool {
        self.cores() == other.cores() && self.memory() == other.memory()
    }
}

impl Eq for Instance {}

fn get_instance(info: &InstanceTypeInfo, instance: &InstanceType) -> Instance {
    let arch = get_architecture(info.processor_info().unwrap().supported_architectures());
    let mut gpu_vec = Vec::new();
    if let Some(gpu_info) = info.gpu_info() {
        for gpu in gpu_info.gpus() {
            gpu_vec.push(Gpu::new(
                gpu.manufacturer().unwrap(),
                gpu.name().unwrap(),
                gpu.count().unwrap(),
                gpu.memory_info().unwrap().size_in_mib().unwrap() / 1024,
            ));
        }
    }

    let is;
    if let Some(storage) = info.instance_storage_info() {
        is = InstanceStorage::new(storage.total_size_in_gb());
    } else {
        is = InstanceStorage::new(None);
    }

    let cpu = Cpu::new(
        instance.clone(),
        &arch,
        info.v_cpu_info().unwrap().default_cores().unwrap(),
        info.memory_info().unwrap().size_in_mib().unwrap() / 1024,
    );

    let network = Network::new(
        info.network_info().unwrap().network_performance().unwrap(),
        get_nr_of_efas(info.network_info().unwrap()),
    );

    Instance::new(
        cpu,
        network,
        info.ebs_info()
            .unwrap()
            .ebs_optimized_info()
            .unwrap()
            .maximum_bandwidth_in_mbps()
            .unwrap()
            / 1024,
        &gpu_vec,
        is,
    )
}

// "us-east-1"
async fn collect(instances: &[InstanceType]) -> Vec<Instance> {
    let shared_config = get_region_config("us-east-1").await;
    let shared_config2 = get_region_config("us-east-2").await;
    let shared_config3 = get_region_config("us-west-2").await;

    let ec2 = aws_sdk_ec2::Client::new(&shared_config);
    let ec2_2 = aws_sdk_ec2::Client::new(&shared_config2);
    let ec2_3 = aws_sdk_ec2::Client::new(&shared_config3);

    let mut vec = Vec::new();

    for instance in instances {
        if let Ok(info) = describe_instance(&ec2, instance.clone()).await {
            let instance = get_instance(&info, instance);
            vec.push(instance);
        } else if let Ok(info) = describe_instance(&ec2_2, instance.clone()).await {
            let instance = get_instance(&info, instance);
            vec.push(instance);
        } else if let Ok(info) = describe_instance(&ec2_3, instance.clone()).await {
            let instance = get_instance(&info, instance);
            vec.push(instance);
        } else {
            println!("failed for {}", instance.as_str());
        }
    }

    vec.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

    // largest in front
    vec.reverse();

    vec
}

/// print information about EC2 instances
pub async fn print_instances(instances: &[InstanceType]) -> Result<()> {
    let instance_data = collect(instances).await;
    print(&instance_data);

    Ok(())
}

const TOP_LINE: &[(&str, usize)] = &[
    ("instance", 17),
    ("arch", 8),
    ("cores", 8),
    ("memory", 8),
    ("network", 20),
    ("EFA", 8),
    ("EBS", 8),
    ("vendor", 10),
    ("model", 10),
    ("number", 10),
    ("memory", 10),
    ("storage", 10),
];

fn print(instances: &[Instance]) {
    for element in TOP_LINE {
        print!("{} | ", get_string_with_len(element.0, element.1));
    }

    println!();

    println!("{}", get_string_with_len_and_padding("", 162, '-'));

    for instance in instances {
        print!("{} | ", get_string_with_dot_and_len(instance.name(), 8, 17));
        print!("{} | ", get_string_with_len(instance.arch(), 8));
        print!("{} | ", get_integer_with_len(instance.cores(), 8));
        print!("{} | ", get_integer_with_len(instance.memory(), 8));
        print!(
            "{} | ",
            get_string_network_and_len(&instance.network_performance(), 12, 20)
        );
        print!("{} | ", get_integer_with_len(instance.get_efas(), 8));
        print!("{} | ", get_integer_with_len(instance.ebs(), 8));

        if instance.gpu_len() > 0 {
            for gpu in instance.gpus() {
                print!("{} | ", get_string_with_len(gpu.manufacturer(), 10));
                print!("{} | ", get_string_with_len(gpu.name(), 10));
                print!("{} | ", get_integer_with_len(gpu.count(), 10));
                print!("{} | ", get_integer_with_len(gpu.memory(), 10));
            }
        } else {
            print!("{} | ", get_string_with_len(" ", 10));
            print!("{} | ", get_string_with_len(" ", 10));
            print!("{} | ", get_string_with_len(" ", 10));
            print!("{} | ", get_string_with_len(" ", 10));
        }

        if let Some(storage) = instance.instance_storage().gb {
            print!("{} | ", get_integer_with_len(storage, 10));
        } else {
            print!("{} | ", get_string_with_len(" ", 10));
        }

        println!()
    }
}
