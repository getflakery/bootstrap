use crate::EC2TagData;
use crate::File;
use anyhow::Result;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_route53::types::Change;
use aws_sdk_route53::types::ChangeBatch;
use aws_sdk_route53::types::ResourceRecord;
use aws_sdk_route53::types::ResourceRecordSet;
use aws_sdk_route53::{Client, Error};
use reqwest::get;

async fn get_ip_address() -> Result<String> {
    let response = get("http://169.254.169.254/latest/meta-data/public-ipv4").await?;
    let ip_address = response.text().await?;
    Ok(ip_address)
}

async fn try_get_ip_address() -> Result<String> {
    for _ in 0..100 {
        match get_ip_address().await {
            Ok(ip_address) => return Ok(ip_address),
            Err(e) => {
                eprintln!("{:?} {}:{}", e, file!(), line!());
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
    Err(anyhow::anyhow!("could not get ip address"))
}

async fn create_record(addr: String, name: String) -> Result<()> {
    let region_provider = RegionProviderChain::default_provider();

    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);
    let cb = ChangeBatch::builder().set_changes(Some(vec![Change::builder()
        .set_action(Some(aws_sdk_route53::types::ChangeAction::Upsert))
        .set_resource_record_set(
            Some(ResourceRecordSet::builder()
            .name(name)
            .r#type("A".into())
            .ttl(300)
            .resource_records(ResourceRecord::builder().value(addr).build()?)
            .build()?),
        ).build()?]));

    let request = client
        .change_resource_record_sets()
        .hosted_zone_id("Z03309493AGZOVY2IU47X")
        .change_batch(cb.build()?);

    match request.send().await {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::from(e).into()),
    }
}

pub async fn bootstrap_load_balancer(ec2_tag_data: &EC2TagData) -> Result<()> {
    println!("bootstrap_load_balancer");

    print!("writing /etc/deployment_id");
    let file = File::new(
        "/etc/deployment_id".to_string(),
        ec2_tag_data.deployment_id.clone(),
    );

    file.write()?;
    println!("wrote /etc/deployment_id");

    // try to get the public ip address of the instance
    let ip = try_get_ip_address().await?;
    println!("public ip address: {}", ip);

    // create dns record for {ec2_tag_data.name}.flakery.app
    // in route53
    // if the record already exists, create the record
    // {ec2_tag_data.name}.{ec2_tag_data.deployment_id[0:6]}.flakery.app

    let name_short = format!("{}.{}.flakery.xyz", ec2_tag_data.name, &ec2_tag_data.deployment_id[0..6]);

    println!("creating record for {}", name_short);
    create_record(ip.clone(), name_short.clone()).await?;
    println!("created record for {}", name_short);
    Ok(()) 
}
