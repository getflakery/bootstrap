use std::collections::HashMap;

use crate::handlers::deploy::DeployAWSOutput;



pub struct Store {
    deploy_aws_outputs: HashMap<String, DeployAWSOutput>,
}

impl Store {
    pub fn new() ->  Self {
         Self {
            deploy_aws_outputs: HashMap::new(),
        }
    }

    pub fn insert_deploy_aws_output(&mut self, output: DeployAWSOutput) {
        self.deploy_aws_outputs
            .insert(output.id.clone(), output.clone());
    }

    pub fn get_deploy_aws_output(&self, id: String) -> Option<&DeployAWSOutput> {
        self.deploy_aws_outputs.get(&id)
    }
}