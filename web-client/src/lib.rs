#[allow(unused_imports)]
use progenitor_client::{encode_path, RequestBuilderExt};
#[allow(unused_imports)]
pub use progenitor_client::{ByteStream, Error, ResponseValue};
#[allow(unused_imports)]
use reqwest::header::{HeaderMap, HeaderValue};
/// Types used as operation parameters and responses.
#[allow(clippy::all)]
pub mod types {
    use serde::{Deserialize, Serialize};
    #[allow(unused_imports)]
    use std::convert::TryFrom;
    /// Error types.
    pub mod error {
        /// Error from a TryFrom or FromStr implementation.
        pub struct ConversionError(std::borrow::Cow<'static, str>);
        impl std::error::Error for ConversionError {}
        impl std::fmt::Display for ConversionError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                std::fmt::Display::fmt(&self.0, f)
            }
        }

        impl std::fmt::Debug for ConversionError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                std::fmt::Debug::fmt(&self.0, f)
            }
        }

        impl From<&'static str> for ConversionError {
            fn from(value: &'static str) -> Self {
                Self(value.into())
            }
        }

        impl From<String> for ConversionError {
            fn from(value: String) -> Self {
                Self(value.into())
            }
        }
    }

    ///CreateListenerInput
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "deployment_id",
    ///    "mappings"
    ///  ],
    ///  "properties": {
    ///    "deployment_id": {
    ///      "type": "string"
    ///    },
    ///    "mappings": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/Mapping"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct CreateListenerInput {
        pub deployment_id: String,
        pub mappings: Vec<Mapping>,
    }

    impl From<&CreateListenerInput> for CreateListenerInput {
        fn from(value: &CreateListenerInput) -> Self {
            value.clone()
        }
    }

    ///CreateListenerOutput
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object"
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct CreateListenerOutput(pub serde_json::Map<String, serde_json::Value>);
    impl std::ops::Deref for CreateListenerOutput {
        type Target = serde_json::Map<String, serde_json::Value>;
        fn deref(&self) -> &serde_json::Map<String, serde_json::Value> {
            &self.0
        }
    }

    impl From<CreateListenerOutput> for serde_json::Map<String, serde_json::Value> {
        fn from(value: CreateListenerOutput) -> Self {
            value.0
        }
    }

    impl From<&CreateListenerOutput> for CreateListenerOutput {
        fn from(value: &CreateListenerOutput) -> Self {
            value.clone()
        }
    }

    impl From<serde_json::Map<String, serde_json::Value>> for CreateListenerOutput {
        fn from(value: serde_json::Map<String, serde_json::Value>) -> Self {
            Self(value)
        }
    }

    ///DeployAwsInput
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "deployment_slug",
    ///    "flake_url",
    ///    "instance_type",
    ///    "subdomain_prefix",
    ///    "template_id"
    ///  ],
    ///  "properties": {
    ///    "deployment_slug": {
    ///      "type": "string"
    ///    },
    ///    "files": {
    ///      "type": [
    ///        "array",
    ///        "null"
    ///      ],
    ///      "items": {
    ///        "$ref": "#/components/schemas/File"
    ///      }
    ///    },
    ///    "flake_url": {
    ///      "type": "string"
    ///    },
    ///    "instance_type": {
    ///      "type": "string"
    ///    },
    ///    "max_size": {
    ///      "type": [
    ///        "integer",
    ///        "null"
    ///      ],
    ///      "format": "int64"
    ///    },
    ///    "min_size": {
    ///      "type": [
    ///        "integer",
    ///        "null"
    ///      ],
    ///      "format": "int64"
    ///    },
    ///    "subdomain_prefix": {
    ///      "type": "string"
    ///    },
    ///    "targets": {
    ///      "type": [
    ///        "array",
    ///        "null"
    ///      ],
    ///      "items": {
    ///        "$ref": "#/components/schemas/Target"
    ///      }
    ///    },
    ///    "template_id": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct DeployAwsInput {
        pub deployment_slug: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub files: Option<Vec<File>>,
        pub flake_url: String,
        pub instance_type: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub max_size: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub min_size: Option<i64>,
        pub subdomain_prefix: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub targets: Option<Vec<Target>>,
        pub template_id: String,
    }

    impl From<&DeployAwsInput> for DeployAwsInput {
        fn from(value: &DeployAwsInput) -> Self {
            value.clone()
        }
    }

    ///DeployAwsOutput
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "id",
    ///    "input"
    ///  ],
    ///  "properties": {
    ///    "id": {
    ///      "type": "string"
    ///    },
    ///    "input": {
    ///      "$ref": "#/components/schemas/DeployAWSInput"
    ///    },
    ///    "lb_arn": {
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct DeployAwsOutput {
        pub id: String,
        pub input: DeployAwsInput,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub lb_arn: Option<String>,
    }

    impl From<&DeployAwsOutput> for DeployAwsOutput {
        fn from(value: &DeployAwsOutput) -> Self {
            value.clone()
        }
    }

    ///File
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "content",
    ///    "path"
    ///  ],
    ///  "properties": {
    ///    "content": {
    ///      "type": "string"
    ///    },
    ///    "path": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct File {
        pub content: String,
        pub path: String,
    }

    impl From<&File> for File {
        fn from(value: &File) -> Self {
            value.clone()
        }
    }

    ///LogInput
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "log"
    ///  ],
    ///  "properties": {
    ///    "log": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct LogInput {
        pub log: String,
    }

    impl From<&LogInput> for LogInput {
        fn from(value: &LogInput) -> Self {
            value.clone()
        }
    }

    ///LogOutput
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object"
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct LogOutput(pub serde_json::Map<String, serde_json::Value>);
    impl std::ops::Deref for LogOutput {
        type Target = serde_json::Map<String, serde_json::Value>;
        fn deref(&self) -> &serde_json::Map<String, serde_json::Value> {
            &self.0
        }
    }

    impl From<LogOutput> for serde_json::Map<String, serde_json::Value> {
        fn from(value: LogOutput) -> Self {
            value.0
        }
    }

    impl From<&LogOutput> for LogOutput {
        fn from(value: &LogOutput) -> Self {
            value.clone()
        }
    }

    impl From<serde_json::Map<String, serde_json::Value>> for LogOutput {
        fn from(value: serde_json::Map<String, serde_json::Value>) -> Self {
            Self(value)
        }
    }

    ///Mapping
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "listener_port",
    ///    "target_port"
    ///  ],
    ///  "properties": {
    ///    "listener_port": {
    ///      "type": "integer",
    ///      "format": "int64"
    ///    },
    ///    "target_port": {
    ///      "type": "integer",
    ///      "format": "int64"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Mapping {
        pub listener_port: i64,
        pub target_port: i64,
    }

    impl From<&Mapping> for Mapping {
        fn from(value: &Mapping) -> Self {
            value.clone()
        }
    }

    ///Target
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "port"
    ///  ],
    ///  "properties": {
    ///    "health_check_enabled": {
    ///      "type": [
    ///        "boolean",
    ///        "null"
    ///      ]
    ///    },
    ///    "health_check_path": {
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    },
    ///    "port": {
    ///      "type": "integer",
    ///      "format": "int64"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Target {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub health_check_enabled: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub health_check_path: Option<String>,
        pub port: i64,
    }

    impl From<&Target> for Target {
        fn from(value: &Target) -> Self {
            value.clone()
        }
    }
}

#[derive(Clone, Debug)]
///Client for webserver
///
///Version: 0.1.0
pub struct Client {
    pub(crate) baseurl: String,
    pub(crate) client: reqwest::Client,
}

impl Client {
    /// Create a new client.
    ///
    /// `baseurl` is the base URL provided to the internal
    /// `reqwest::Client`, and should include a scheme and hostname,
    /// as well as port and a path stem if applicable.
    pub fn new(baseurl: &str) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let client = {
            let dur = std::time::Duration::from_secs(15);
            reqwest::ClientBuilder::new()
                .connect_timeout(dur)
                .timeout(dur)
        };
        #[cfg(target_arch = "wasm32")]
        let client = reqwest::ClientBuilder::new();
        Self::new_with_client(baseurl, client.build().unwrap())
    }

    /// Construct a new client with an existing `reqwest::Client`,
    /// allowing more control over its configuration.
    ///
    /// `baseurl` is the base URL provided to the internal
    /// `reqwest::Client`, and should include a scheme and hostname,
    /// as well as port and a path stem if applicable.
    pub fn new_with_client(baseurl: &str, client: reqwest::Client) -> Self {
        Self {
            baseurl: baseurl.to_string(),
            client,
        }
    }

    /// Get the base URL to which requests are made.
    pub fn baseurl(&self) -> &String {
        &self.baseurl
    }

    /// Get the internal `reqwest::Client` used to make requests.
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }

    /// Get the version of this API.
    ///
    /// This string is pulled directly from the source OpenAPI
    /// document and may be in any format the API selects.
    pub fn api_version(&self) -> &'static str {
        "0.1.0"
    }
}

#[allow(clippy::all)]
impl Client {
    ///Get instance ID from queue
    ///
    ///Retrieves the next available EC2 instance ID from the queue.
    ///
    ///Sends a `POST` request to `/deploy/aws/create`
    pub async fn handlers_deploy_deploy_aws_create<'a>(
        &'a self,
        body: &'a types::DeployAwsInput,
    ) -> Result<ResponseValue<types::DeployAwsOutput>, Error<()>> {
        let url = format!("{}/deploy/aws/create", self.baseurl,);
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                reqwest::header::ACCEPT,
                reqwest::header::HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .build()?;
        let result = self.client.execute(request).await;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            400u16 => Err(Error::ErrorResponse(ResponseValue::empty(response))),
            404u16 => Err(Error::ErrorResponse(ResponseValue::empty(response))),
            422u16 => Err(Error::ErrorResponse(ResponseValue::empty(response))),
            500u16 => Err(Error::ErrorResponse(ResponseValue::empty(response))),
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }

    ///Get instance ID from queue
    ///
    ///Retrieves the next available EC2 instance ID from the queue.
    ///
    ///Sends a `POST` request to `/log`
    pub async fn handlers_log_log<'a>(
        &'a self,
        body: &'a types::LogInput,
    ) -> Result<ResponseValue<types::LogOutput>, Error<()>> {
        let url = format!("{}/log", self.baseurl,);
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                reqwest::header::ACCEPT,
                reqwest::header::HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .build()?;
        let result = self.client.execute(request).await;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            400u16 => Err(Error::ErrorResponse(ResponseValue::empty(response))),
            404u16 => Err(Error::ErrorResponse(ResponseValue::empty(response))),
            422u16 => Err(Error::ErrorResponse(ResponseValue::empty(response))),
            500u16 => Err(Error::ErrorResponse(ResponseValue::empty(response))),
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }

    ///Sends a `POST` request to `/create-listener`
    pub async fn handlers_create_listener_create_listener<'a>(
        &'a self,
        body: &'a types::CreateListenerInput,
    ) -> Result<ResponseValue<types::CreateListenerOutput>, Error<()>> {
        let url = format!("{}/create-listener", self.baseurl,);
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                reqwest::header::ACCEPT,
                reqwest::header::HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .build()?;
        let result = self.client.execute(request).await;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            400u16 => Err(Error::ErrorResponse(ResponseValue::empty(response))),
            404u16 => Err(Error::ErrorResponse(ResponseValue::empty(response))),
            422u16 => Err(Error::ErrorResponse(ResponseValue::empty(response))),
            500u16 => Err(Error::ErrorResponse(ResponseValue::empty(response))),
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
}

/// Items consumers will typically use such as the Client.
pub mod prelude {
    #[allow(unused_imports)]
    pub use super::Client;
}
