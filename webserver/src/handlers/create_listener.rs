use rocket_okapi::openapi;
use crate::error::OResult;
use crate::handlers::deploy::DeployAWSOutput;
use rocket::serde::json::Json;
use tokio::sync::Mutex;
use crate::AppState;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
struct CreateListenerInput {
    deployment_id: String,
    listener_port: i64,
    target_port: i64,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Default)]
struct CreateListenerOutput {

}


#[openapi]
#[post("/put")]
pub async fn put(
    state: &State<Mutex<AppState>>,
) -> OResult<()> {
    let mut state = state.lock().await;
    let id = uuid::Uuid::new_v4();
    let dplymnt = state.store.insert_deploy_aws_output(
        DeployAWSOutput {
            id: id.to_string(),
            ..Default::default()
        },
    );
    print!("{:?}", id);
    Ok(Json(()))
}

#[openapi]
#[post("/create-listener", data = "<input>")]
pub async fn create_listener(
    state: &State<Mutex<AppState>>,
    input: Json<CreateListenerInput>,
) -> OResult<CreateListenerOutput> {
    let state = state.lock().await;
    let dplymnt = state.store.get_deploy_aws_output(input.deployment_id.clone());
    print!("{:?}", dplymnt);
    Ok(Json(CreateListenerOutput {}))

    // let  s = &mut state.lock().await;

    // // create target group
    // let elb_client = &s.elb_client;

    // let create_target_group_reqs = input
    //     .targets
    //     .as_ref()
    //     .unwrap_or(&vec![Target {
    //         port: 8000,
    //         ..Default::default()
    //     }])
    //     .iter()
    //     .map(|t| {
    //         rusoto_elbv2::CreateTargetGroupInput {
    //             name: input.deployment_slug.clone(),
    //             protocol: Some("HTTP".to_string()),
    //             port: Some(t.port),
    //             vpc_id: Some(vpc_id.clone()),
    //             health_check_path: t.health_check_path.clone(),
    //             // Add other parameters here as needed
    //             ..Default::default()
    //         }
    //     })
    //     .collect::<Vec<rusoto_elbv2::CreateTargetGroupInput>>();

    // let mut target_group_arns = vec![];

    // for req in create_target_group_reqs {
    //     let resp = elb_client.create_target_group(req).await;

    //     match resp {
    //         Ok(output) => {
    //             println!("Target group created: {:?}", output);
    //             if let Some(target_groups) = output.target_groups {
    //                 target_group_arns.push(target_groups[0].target_group_arn.clone());
    //             }
    //         }
    //         Err(e) => {
    //             return Err(error::Error::new(
    //                 "TargetGroupCreationFailed",
    //                 Some(&e.to_string()),
    //                 500,
    //             ));
    //         }
    //     }
    // }

    // let tarns = input
    //     .targets
    //     .as_ref()
    //     .unwrap_or(&vec![Target {
    //         port: 8000,
    //         ..Default::default()
    //     }])
    //     .iter()
    //     .zip(target_group_arns.iter())
    //     .map(|(t, arn)| Tarn {
    //         target: t.clone(),
    //         arn: arn.clone().unwrap(),
    //     })
    //     .collect::<Vec<Tarn>>();


    // let create_listener_reqs = tarns
    //     .iter()
    //     .map(|tarn| {
    //         rusoto_elbv2::CreateListenerInput {
    //             default_actions: vec![rusoto_elbv2::Action {
    //                 target_group_arn: Some(tarn.arn.clone()),
    //                 type_: "forward".to_string(),
    //                 ..Default::default()
    //             }],
    //             load_balancer_arn: load_balancer_arn.clone().unwrap(),
    //             port: Some(443),
    //             protocol: Some("HTTPS".to_string()),
    //             ssl_policy: Some("ELBSecurityPolicy-2016-08".to_string()),
    //             certificates: Some(vec![rusoto_elbv2::Certificate {
    //                 certificate_arn: Some(
    //                     "arn:aws:acm:us-west-1:150301572911:certificate/3c9f6d82-849a-46e5-b6b6-a3a70b7a97c5"
    //                         .to_string(),
    //                 ),
    //                 ..Default::default()
    //             }]),
    //             // Add other parameters here as needed
    //             ..Default::default()
    //         }
    //     })
    //     .collect::<Vec<rusoto_elbv2::CreateListenerInput>>();

    // for req in create_listener_reqs {
    //     let resp = elb_client.create_listener(req).await;

    //     match resp {
    //         Ok(output) => {
    //             println!("Listener created: {:?}", output);
    //         }
    //         Err(e) => {
    //             return Err(error::Error::new(
    //                 "ListenerCreationFailed",
    //                 Some(&e.to_string()),
    //                 500,
    //             ));
    //         }
    //     }
    // }

    // // attach target group to auto scaling group
    // let attach_tg_req = rusoto_autoscaling::AttachLoadBalancerTargetGroupsType {
    //     auto_scaling_group_name: input.deployment_slug.clone(),
    //     target_group_ar_ns: target_group_arns
    //         .iter()
    //         .map(|t| t.clone().unwrap())
    //         .collect::<Vec<String>>(),
    //     // Add other parameters here as needed
    //     ..Default::default()
    // };

    // let resp = as_client
    //     .attach_load_balancer_target_groups(attach_tg_req)
    //     .await;

    // match resp {
    //     Ok(output) => {
    //         println!("Target group attached: {:?}", output);
    //     }
    //     Err(e) => {
    //         return Err(error::Error::new(
    //             "TargetGroupAttachFailed",
    //             Some(&e.to_string()),
    //             500,
    //         ));
    //     }
    // }

}