
use anyhow::Result;
use crate::EC2TagData;
use crate::File;

pub fn bootstrap_load_balancer(
    ec2_tag_data: &EC2TagData,
) -> Result<()> {
    println!("bootstrap_load_balancer");


    print!("writing /etc/deployment_id");
    let file = File::new(
        "/etc/deployment_id".to_string(),
        ec2_tag_data.deployment_id.clone(),
    );

    file.write()?;
    println!("wrote /etc/deployment_id");


    Ok(())
}