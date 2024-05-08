use rocket_okapi::openapi;
use rusoto_autoscaling::Autoscaling;
use rusoto_elbv2::Elb;
use crate::error::OResult;

use rocket::serde::json::Json;
use tokio::sync::Mutex;
use crate::AppState;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;

use crate::guards::debug_header;

#[derive(Serialize, Deserialize, JsonSchema, Clone)]

pub struct Mapping {
    listener_port: i64,
    target_port: i64,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct CreateListenerInput {
    deployment_id: String,
    mappings: Vec<Mapping>,

}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Default)]
pub struct CreateListenerOutput {

}

#[post("/create-listener", data = "<_input>")]
pub async fn create_listener_fake(_debug: debug_header::DebugHeader, _input: Json<CreateListenerInput>) -> Json<CreateListenerOutput> {
    Json(CreateListenerOutput {})
}

#[openapi]
#[post("/create-listener", data = "<input>", rank=2)]
pub async fn create_listener(
    mut state: &State<AppState>,
    store: &State<Mutex<crate::store::Store>>,
    input: Json<CreateListenerInput>,
) -> OResult<CreateListenerOutput> {
    // todo make this a config
    let vpc_id = "vpc-031c620b47a9ea885".to_string();
    let public_subnets = vec![
        "subnet-040ebc679c54ecf38".to_string(),
        "subnet-0e22657a6f50a3235".to_string(),
    ];
    let s =  store.lock().await;
    let dplymnt = s.get_deploy_aws_output(input.deployment_id.clone());
    // let dplymnt = None;
    print!("{:?}", dplymnt);
    let d = match dplymnt {
        Some(d) => {
           d
        },
        None => {
            return Err(crate::error::Error::new(
                "DeploymentNotFound",
                Some("Deployment not found"),
                404,
            ));
        }
    };

    let  s = &mut state;

    // create target group
    let elb_client = &s.elb_client;

    let create_target_group_reqs = input
        .mappings
        .iter()
        .map(|t| {
            rusoto_elbv2::CreateTargetGroupInput {
                name: format!("{}-{}", d.input.deployment_slug, t.target_port),
                protocol: Some("HTTP".to_string()),
                port: Some(t.target_port),
                vpc_id: Some(vpc_id.clone()),
                health_check_path: Some("/".to_string()),
                // Add other parameters here as needed
                ..Default::default()
            }
        })
        .collect::<Vec<rusoto_elbv2::CreateTargetGroupInput>>();

    let mut target_group_arns = vec![];

    for req in create_target_group_reqs {
        let resp = elb_client.create_target_group(req).await;

        match resp {
            Ok(output) => {
                println!("Target group created: {:?}", output);
                if let Some(target_groups) = output.target_groups {
                    target_group_arns.push(target_groups[0].target_group_arn.clone());
                }
            }
            Err(e) => {
                return Err(crate::error::Error::new(
                    "TargetGroupCreationFailed",
                    Some(&e.to_string()),
                    500,
                ));
            }
        }
    }

    let create_listener_reqs = target_group_arns
        .iter()
        .map(|t| {
            rusoto_elbv2::CreateListenerInput {
                default_actions: vec![rusoto_elbv2::Action {
                    target_group_arn: t.clone(),
                    type_: "forward".to_string(),
                    ..Default::default()
                }],
                load_balancer_arn: d.lb_arn.clone().unwrap(),
                port: Some(443),
                protocol: Some("HTTPS".to_string()),
                ssl_policy: Some("ELBSecurityPolicy-2016-08".to_string()),
                certificates: Some(vec![rusoto_elbv2::Certificate {
                    certificate_arn: Some(
                        "arn:aws:acm:us-west-1:150301572911:certificate/3c9f6d82-849a-46e5-b6b6-a3a70b7a97c5"
                            .to_string(),
                    ),
                    ..Default::default()
                }]),
                // Add other parameters here as needed
                ..Default::default()
            }
        })
        .collect::<Vec<rusoto_elbv2::CreateListenerInput>>();

    for req in create_listener_reqs {
        let resp = elb_client.create_listener(req).await;

        match resp {
            Ok(output) => {
                println!("Listener created: {:?}", output);
            }
            Err(e) => {
                return Err(crate::error::Error::new(
                    "ListenerCreationFailed",
                    Some(&e.to_string()),
                    500,
                ));
            }
        }
    }

    // attach target group to auto scaling group
    let attach_tg_req = rusoto_autoscaling::AttachLoadBalancerTargetGroupsType {
        auto_scaling_group_name: d.input.deployment_slug.clone(),
        target_group_ar_ns: target_group_arns
            .iter()
            .map(|t| t.clone().unwrap())
            .collect::<Vec<String>>(),
        // Add other parameters here as needed
        ..Default::default()
    };

    let as_client = &s.as_client;

    let resp = as_client
        .attach_load_balancer_target_groups(attach_tg_req)
        .await;

    match resp {
        Ok(output) => {
            println!("Target group attached: {:?}", output);
        }
        Err(e) => {
            return Err(crate::error::Error::new(
                "TargetGroupAttachFailed",
                Some(&e.to_string()),
                500,
            ));
        }
    }
    Ok(Json(CreateListenerOutput {}))

}