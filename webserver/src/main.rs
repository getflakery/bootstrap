#[macro_use]
extern crate rocket;

use dotenv::dotenv;

use crate::store::Store;

use rocket_okapi::settings::UrlObject;

use rocket_okapi::swagger_ui::make_swagger_ui;
use rocket_okapi::{openapi_get_routes, rapidoc::*, swagger_ui::*};
use rusoto_core::Region;
use rusoto_ec2::Ec2Client;

use std::env;
use tokio::sync::Mutex;

mod error;
mod handlers;
mod store;
mod guards;

// let id = Uuid::new_v4();
use aws_config::BehaviorVersion;

pub struct AppState {
    ec2_client: Ec2Client,
    as_client: rusoto_autoscaling::AutoscalingClient,
    elb_client: rusoto_elbv2::ElbClient,
    ec2_client_ng: aws_sdk_ec2::Client,
    route53_client: aws_sdk_route53::Client,
    store: store::Store,
}

#[rocket::main]
async fn main() {
    dotenv().ok();
    let args: Vec<String> = env::args().collect();
    if args.contains(&"--print-openapi".to_string()) {
        let settings = rocket_okapi::settings::OpenApiSettings::new();
        let spec = rocket_okapi::openapi_spec![
            handlers::deploy::deploy_aws_create,
            handlers::log::log,
            handlers::create_listener::create_listener,
        ](&settings);
        println!("{}", serde_json::to_string_pretty(&spec).unwrap());
        return;
    }

    let ec2_client = Ec2Client::new(Region::default());
    let as_client = rusoto_autoscaling::AutoscalingClient::new(Region::default());
    let elb_client = rusoto_elbv2::ElbClient::new(Region::default());

    let config = aws_config::load_defaults(BehaviorVersion::v2023_11_09()).await;
    let ec2_client_ng = aws_sdk_ec2::Client::new(&config);
    let route53_client = aws_sdk_route53::Client::new(&config);

    let store = Store::new();

    let _ = rocket::build()
        .configure(rocket::Config {
            address: "0.0.0.0".parse().expect("valid IP address"),
            port: 8000,
            ..rocket::Config::default()
        })
        .manage(Mutex::new(AppState {
            ec2_client,
            as_client,
            elb_client,
            ec2_client_ng,
            route53_client,
            store,
        }))
        .mount(
            "/",
            openapi_get_routes![
                handlers::deploy::deploy_aws_create,
                handlers::log::log,
                handlers::create_listener::create_listener,
            ],
        )
        .mount(
            "/",
            routes![
                handlers::create_listener::create_listener_fake,
            ],
        )
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .mount(
            "/rapidoc/",
            make_rapidoc(&RapiDocConfig {
                general: GeneralConfig {
                    spec_urls: vec![UrlObject::new("General", "../openapi.json")],
                    ..Default::default()
                },
                hide_show: HideShowConfig {
                    allow_spec_url_load: false,
                    allow_spec_file_load: false,
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
        .launch()
        .await;
}
