
use std::io::{self, BufRead};
use serde::Serialize;


#[derive(Serialize, Debug)]
struct LogLine {
    message: String,
    deployment_id: String,
}

pub fn wrap_with_deployment_id(deployment_id: &str) {
    let stdin = io::stdin();
    let reader = stdin.lock();

    for line_result in reader.lines() {
        match line_result {
            Ok(line) => {
                let log_line = LogLine {
                    message: line,
                    deployment_id: deployment_id.to_string(),
                };
                let json = serde_json::to_string(&log_line);

                match json {
                    Ok(json) => {
                        println!("{}", json);
                    }
                    Err(e) => {
                        eprintln!("Error serializing line: {}", e);
                    }
                }
               
            }
            Err(e) => {
                eprintln!("Error reading line: {}", e);
                break;
            }
        }
    }
}