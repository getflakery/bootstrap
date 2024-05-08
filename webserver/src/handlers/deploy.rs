use aws_sdk_ec2::types::{LaunchTemplateInstanceMetadataTagsState, RequestLaunchTemplateData};
use rusoto_elbv2::Elb;

use crate::{error::{self, OResult}, store::Store};
use rocket::serde::json::Json;

use rocket::State;
use rocket_okapi::okapi::schemars::JsonSchema;

use rocket_okapi::openapi;
use rusoto_autoscaling::Autoscaling;
use rusoto_ec2::Ec2;

use tokio::sync::Mutex;

use rocket::serde::{Deserialize, Serialize};

use uuid::Uuid;

use std::collections::HashMap;

use crate::AppState;

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, Default)]
struct File {
    content: String,
    path: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Default, Debug)]
struct Target {
    port: i64,
    health_check_path: Option<String>,
    health_check_enabled: Option<bool>,
}

struct Tarn {
    target: Target,
    arn: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, Default)]
pub struct DeployAWSInput {
    flake_url: String,
    instance_type: String,
    pub deployment_slug: String, // i am the deployment slug @_\/
    subdomain_prefix: String,
    min_size: Option<i64>,
    max_size: Option<i64>,
    targets: Option<Vec<Target>>,
    template_id: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, Default)]
pub struct DeployAWSOutput {
    pub id: String,
    pub input: DeployAWSInput,
    pub lb_arn: Option<String>,
}

impl DeployAWSOutput {
    fn new(input: DeployAWSInput) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            input: input,
            lb_arn: None,
        }
    }
}

fn get_tag_data(
    template_id: String,
    flake_url: String,
    deployment_id: String,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut tags = HashMap::new();

    let turso_token = std::env::var("TURSO_TOKEN")
        .map_err(|e| error::Error::new("TursoTokenCreationFailed", Some(&e.to_string()), 500))?;
    let file_encryption_key = std::env::var("FILE_ENCRYPTION_KEY").map_err(|e| {
        error::Error::new("FileEncryptionKeyCreationFailed", Some(&e.to_string()), 500)
    })?;
    tags.insert("turso_token".to_string(), turso_token);
    tags.insert("file_encryption_key".to_string(), file_encryption_key);
    tags.insert("template_id".to_string(), template_id);
    tags.insert("flake_url".to_string(), flake_url);
    tags.insert("deployment_id".to_string(), deployment_id);
    Ok(tags)
}
/// Get instance ID from queue
///
/// Retrieves the next available EC2 instance ID from the queue.
#[openapi]
#[post("/deploy/aws/create", data = "<input>")]
pub async fn deploy_aws_create(
     mut state: &State<AppState>,
     mut store: &State<Mutex<Store>>,
    input: Json<DeployAWSInput>,
) -> OResult<DeployAWSOutput> {
    println!("Input: {:?}", input.0.clone().deployment_slug);
    let mut output = DeployAWSOutput::new(input.0.clone());

    println!("getting tag data");

    let tags = get_tag_data(
        input.template_id.clone(),
        input.flake_url.clone(),
        output.id.clone(),
    )
    .map_err(|e| error::Error::new("TagDataCreationFailed", Some(&e.to_string()), 500))?;

    println!("got tag data");

    println!("Tags: {:?}", tags);

    println!("Creating launch template");
    
    // instead create launch template with ec2_client_ng b/c that has access to
    // the latest version of the api
    println!("hmm");

    let ec2_client_ng =  state.ec2_client_ng.clone();
    ec2_client_ng
        .create_launch_template()
        .set_launch_template_name(Some(input.deployment_slug.clone()))
        .set_launch_template_data(Some(
            RequestLaunchTemplateData::builder()
                .instance_type(aws_sdk_ec2::types::InstanceType::T3Small)
                .image_id("ami-07dba754bbb515299")
                .set_metadata_options(Some(
                    aws_sdk_ec2::types::LaunchTemplateInstanceMetadataOptionsRequest::builder()
                        .set_instance_metadata_tags(Some(
                            LaunchTemplateInstanceMetadataTagsState::Enabled,
                        ))
                        .build(),
                ))
                .set_tag_specifications(Some(vec![
                    aws_sdk_ec2::types::LaunchTemplateTagSpecificationRequest::builder()
                        .set_resource_type(Some(aws_sdk_ec2::types::ResourceType::Instance))
                        .set_tags(Some(
                            tags.iter()
                                .map(|(k, v)| {
                                    aws_sdk_ec2::types::Tag::builder()
                                        .set_key(Some(k.clone()))
                                        .set_value(Some(v.clone()))
                                        .build()
                                })
                                .collect(),
                        ))
                        .build(),
                ]))
                .set_block_device_mappings(Some(vec![
                    aws_sdk_ec2::types::LaunchTemplateBlockDeviceMappingRequest::builder()
                        .device_name("/dev/xvda")
                        .ebs(
                            aws_sdk_ec2::types::LaunchTemplateEbsBlockDeviceRequest::builder()
                                .volume_size(80)
                                .volume_type(aws_sdk_ec2::types::VolumeType::Gp2)
                                .delete_on_termination(true)
                                .build(),
                        )
                        .build(),
                ]))
                .build(),
        ))
        .send()
        .await
        .map_err(|e| {
            error::Error::new("LaunchTemplateCreationFailed", Some(&e.to_string()), 500)
        })?;
    
    println!("Launch template created");

    let as_client =  state.as_client.clone();

    println!("Creating auto scaling group");
    // create auto scaling group
    let create_asg_req = rusoto_autoscaling::CreateAutoScalingGroupType {
        auto_scaling_group_name: input.deployment_slug.clone(),
        launch_template: Some(rusoto_autoscaling::LaunchTemplateSpecification {
            launch_template_name: Some(input.deployment_slug.clone()),
            ..Default::default()
        }),
        min_size: input.min_size.unwrap_or(1),
        max_size: input.max_size.unwrap_or(1),
        vpc_zone_identifier: Some("subnet-040ebc679c54ecf38".to_string()),
        // availability_zones: Some(vec!["us-west-1a".to_string(), "us-west-1c".to_string()]),
        // desired_capacity: 1,
        // Add other parameters here as needed
        ..Default::default()
    };

    let resp = as_client.create_auto_scaling_group(create_asg_req).await;

    match resp {
        Ok(output) => {
            println!("Auto scaling group created: {:?}", output);
        }
        Err(e) => {
            return Err(error::Error::new(
                "AutoScalingGroupCreationFailed",
                Some(&e.to_string()),
                500,
            ));
        }
    }

    let vpc_id = "vpc-031c620b47a9ea885".to_string();
    let public_subnets = vec![
        "subnet-040ebc679c54ecf38".to_string(),
        "subnet-0e22657a6f50a3235".to_string(),
    ];

    // create security groups
    let create_sg_req = rusoto_ec2::CreateSecurityGroupRequest {
        description: "Security group for the deployment".to_string(),
        group_name: input.deployment_slug.clone(),
        vpc_id: Some(vpc_id.clone()),
        // Add other parameters here as needed
        ..Default::default()
    };

    let ec2_client:  rusoto_ec2::Ec2Client = state.ec2_client.clone();


    let resp = ec2_client.create_security_group(create_sg_req).await;
    let sg_id = match resp {
        Ok(output) => {
            println!("Security group created: {:?}", output);
            output.group_id.clone()
        }
        Err(e) => {
            return Err(error::Error::new(
                "SecurityGroupCreationFailed",
                Some(&e.to_string()),
                500,
            ));
        }
    };
    // rusoto_ec2::AuthorizeSecurityGroupIngressRequest {
    //     group_id:sg_id.clone(),
    //     // Add other parameters here as needed
    //     ..Default::default()
    // };

    let authorize_sg_reqs = input
        .targets
        .as_ref()
        .unwrap_or(&vec![Target {
            port: 8000,
            ..Default::default()
        }])
        .iter()
        .chain(&vec![Target {
            port: 443,
            ..Default::default()
        }])
        .map(|t| rusoto_ec2::AuthorizeSecurityGroupIngressRequest {
            group_id: sg_id.clone(),
            from_port: Some(t.port),
            to_port: Some(t.port),
            ip_protocol: Some("TCP".to_string()),
            cidr_ip: Some("0.0.0.0/0".to_string()),
            ..Default::default()
        })
        .collect::<Vec<rusoto_ec2::AuthorizeSecurityGroupIngressRequest>>();

    for req in authorize_sg_reqs {
        let resp = ec2_client.authorize_security_group_ingress(req).await;

        match resp {
            Ok(output) => {
                println!("Security group ingress rules added: {:?}", output);
            }
            Err(e) => {
                return Err(error::Error::new(
                    "SecurityGroupIngressRulesAdditionFailed",
                    Some(&e.to_string()),
                    500,
                ));
            }
        }
    }

    // create load balancer
    let create_lb_req = rusoto_elbv2::CreateLoadBalancerInput {
        name: input.deployment_slug.clone(),
        subnets: Some(public_subnets),
        security_groups: Some(vec![sg_id.expect("sg should be set")]), // todo add security group
        // Add other parameters here as needed
        ..Default::default()
    };

    let elb_client = state.elb_client.clone();

    let resp = elb_client.create_load_balancer(create_lb_req).await;

    let (lb_dns, load_balancer_arn) = match resp {
        Ok(output) => {
            println!("Load balancer created: {:?}", output);
            let lb_dns = output.load_balancers.as_ref().unwrap()[0].dns_name.clone();
            let load_balancer_arn = output.load_balancers.as_ref().unwrap()[0]
                .load_balancer_arn
                .clone();
            (lb_dns, load_balancer_arn)
        }
        Err(e) => {
            return Err(error::Error::new(
                "LoadBalancerCreationFailed",
                Some(&e.to_string()),
                500,
            ));
        }
    };

    // set load balancer arn in output
    output.lb_arn = Some(load_balancer_arn.clone().unwrap());
    let s = &mut state;

    // let store = &mut s.store;
    // store.insert_deploy_aws_output(output.clone());

    let s = &mut state;

    let route53_client = &s.route53_client;

    let resp = route53_client
        .change_resource_record_sets()
        .set_change_batch(Some(
            aws_sdk_route53::types::ChangeBatch::builder()
                .set_changes(Some(vec![aws_sdk_route53::types::Change::builder()
                    .set_action(Some(aws_sdk_route53::types::ChangeAction::Create))
                    .set_resource_record_set(Some(
                        aws_sdk_route53::types::ResourceRecordSet::builder()
                            .set_name(Some(input.subdomain_prefix.clone() + ".app.flakery.xyz"))
                            .set_type(Some(aws_sdk_route53::types::RrType::Cname))
                            .set_ttl(Some(300))
                            .set_resource_records(Some(vec![
                                aws_sdk_route53::types::ResourceRecord::builder()
                                    .set_value(Some(lb_dns.clone().unwrap()))
                                    .build()
                                    .unwrap(),
                            ]))
                            .build()
                            .unwrap(),
                    ))
                    .build()
                    .unwrap()]))
                .build()
                .unwrap(),
        ))
        .set_hosted_zone_id(Some("Z03309493AGZOVY2IU47X".to_string()))
        .send()
        .await;

    match resp {
        Ok(output) => {
            println!("Record set created: {:?}", output);
        }
        Err(e) => {
            eprintln!("Failed to create record set: {:?}", e); // Log the full error
            return Err(error::Error::new(
                "RecordSetCreationFailed",
                Some(&e.to_string()),
                500,
            ));
        }
    }


    store.lock().await.insert_deploy_aws_output(output.clone());

    Ok(Json(output))
}
