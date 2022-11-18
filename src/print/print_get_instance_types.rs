use crate::describe_instance;
use anyhow::Result;
use aws_sdk_ec2::model::InstanceType;
use aws_sdk_ec2::model::MemoryGiBPerVCpuRequest;
use aws_sdk_ec2::model::{
    AcceleratorCountRequest, ArchitectureType, BareMetal, BurstablePerformance, InstanceGeneration,
    InstanceRequirementsRequest, LocalStorage, MemoryMiBRequest, VCpuCountRangeRequest, VirtualizationType,
};

// GetInstanceTypesFromInstanceRequirements

/// print attribute-based instances
pub async fn print_instance_type() -> Result<()> {
    let shared_config = aws_config::load_from_env().await;

    let ec2_client = aws_sdk_ec2::Client::new(&shared_config);

    let architectures = vec![ArchitectureType::Arm64];

    let vcpu = VCpuCountRangeRequest::builder().max(16).min(4).build();

    let memory = MemoryGiBPerVCpuRequest::builder().min(1.0).max(8.0).build();
    let memory2 = MemoryMiBRequest::builder().min(2 * 1024).max(32 * 1024).build();

    let accelerators = AcceleratorCountRequest::builder().min(0).max(0).build();

    let requirements = InstanceRequirementsRequest::builder()
        .v_cpu_count(vcpu)
        .memory_mi_b(memory2)
        .memory_gi_b_per_v_cpu(memory)
        .instance_generations(InstanceGeneration::Current)
        .bare_metal(BareMetal::Excluded)
        .burstable_performance(BurstablePerformance::Excluded)
        .accelerator_count(accelerators)
        .local_storage(LocalStorage::Excluded)
        .build();

    let virt_types = vec![VirtualizationType::Hvm, VirtualizationType::Paravirtual];

    let result = ec2_client
        .get_instance_types_from_instance_requirements()
        .set_architecture_types(Some(architectures))
        .instance_requirements(requirements)
        .set_virtualization_types(Some(virt_types))
        .send()
        .await?;

    let instances: Vec<String> = result
        .instance_types()
        .unwrap_or_default()
        .iter()
        .map(|i| i.instance_type().unwrap().to_string())
        .collect::<Vec<_>>();

    for instance in instances {
        let inst = describe_instance(&ec2_client, InstanceType::from(instance.as_ref())).await?;
        println!(
            "{:16} {:4} : vcpus {:8} MB",
            instance,
            inst.v_cpu_info().unwrap().default_cores().unwrap(),
            inst.memory_info().unwrap().size_in_mi_b().unwrap()
        );
    }

    Ok(())
}
