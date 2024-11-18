use std::{collections::HashMap, env, time::Duration};

use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};

// The structs follow the json schema defined in the GitHub docs.
// This causes there to be unused fields which then throw a dead
// code warning, these warnings may be suppressed.

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GetContent {
    Struct(GetContentFile),
    List(Vec<GetContentDirectory>),
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct GetContentFile {
    #[serde(rename = "type")]
    pub _type: String,
    pub encoding: String,
    pub size: u64,
    pub name: String,
    pub path: String,
    pub content: String,
    pub sha: String,
    pub url: String,
    pub git_url: Option<String>,
    pub html_url: Option<String>,
    pub download_url: Option<String>,
    pub _links: Links,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct GetContentDirectory {
    #[serde(rename = "type")]
    pub _type: String,
    pub size: u64,
    pub name: String,
    pub path: String,
    pub content: Option<String>,
    pub sha: String,
    pub url: String,
    pub git_url: Option<String>,
    pub html_url: Option<String>,
    pub download_url: Option<String>,
    pub _links: Links,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Links {
    #[serde(rename = "self")]
    pub _self: String,
    pub git: Option<String>,
    pub html: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct GetBaseTreeResponse {
    pub sha: String,
    pub url: String,
    pub truncated: bool,
    pub tree: Vec<ResponseFile>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ResponseFile {
    pub path: String,
    pub mode: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub sha: String,
    pub size: Option<u64>,
    pub url: String,
}

#[derive(Debug, Serialize)]
struct CreateTreeRequestBody {
    pub base_tree: String,
    pub tree: Vec<RequestFile>,
}

#[derive(Debug, Serialize)]
struct RequestFile {
    pub path: String,
    pub mode: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub content: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct CreateTreeResponse {
    pub sha: String,
    pub url: String,
    pub truncated: bool,
    pub tree: Vec<ResponseFile>,
}

pub struct Requests {
    client: Client,
}

pub fn new() -> Result<Requests, u8> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Accept",
        HeaderValue::from_str("application/vnd.github+json").unwrap(),
    );
    headers.insert(
        "X-GitHub-Api-Version",
        HeaderValue::from_str("2022-11-28").unwrap(),
    );
    headers.insert(
        "Authorization",
        HeaderValue::from_str(
            format!("Bearer {}", env::var("REGISTRY_MANAGER_PAT").unwrap()).as_str(),
        )
        .unwrap(),
    );

    match Client::builder()
        .user_agent("paperback-community/registry-manager")
        .default_headers(headers)
        .timeout(Duration::new(10, 0))
        .build()
    {
        Ok(client) => {
            println!("Created a request client");
            Ok(Requests { client })
        }
        Err(err) => {
            println!(
                "Something went wrong while creating the request client: {}",
                &err
            );
            Err(0x1)
        }
    }
}

impl Requests {
    pub fn get_files(
        &self,
        repository: &String,
        path: &String,
        branch: &String,
    ) -> Result<GetContent, u8> {
        let p_response = self
            .client
            .get(format!(
                "https://api.github.com/repos/{}/contents/{}?ref={}",
                &repository, &path, &branch
            ))
            .send();

        match p_response {
            Ok(raw_response) => {
                if raw_response.status() != 200 {
                    eprintln!(
                        "The response was undesired, status code: {}",
                        &raw_response.status(),
                    );
                    return Err(0x1);
                }

                match raw_response.json::<GetContent>() {
                    Ok(response) => {
                        println!(
                            "Requested the file(s) which match the following repository and path: {}/{}",
                            &repository, &path
                        );
                        Ok(response)
                    }
                    Err(err) => {
                        eprintln!(
                            "Something went wrong while deserializing the response to JSON: {}",
                            &err
                        );
                        Err(0x1)
                    }
                }
            }
            Err(err) => {
                eprintln!("Something went wrong while making the request: {}", &err);
                Err(0x1)
            }
        }
    }

    pub fn get_tree(
        &self,
        repository: &String,
        sha_ref: &String,
    ) -> Result<GetBaseTreeResponse, u8> {
        let p_response = self
            .client
            .get(format!(
                "https://api.github.com/repos/{}/git/trees/{}",
                &repository, &sha_ref
            ))
            .send();

        match p_response {
            Ok(raw_response) => {
                if raw_response.status() != 200 {
                    eprintln!(
                        "The response was undesired, status code: {}",
                        &raw_response.status(),
                    );
                    return Err(0x1);
                }

                match raw_response.json::<GetBaseTreeResponse>() {
                    Ok(response) => {
                        println!(
                            "Requested the tree which match the following repository and sha/ref: {}, {}",
                            &repository, &sha_ref
                        );
                        Ok(response)
                    }
                    Err(err) => {
                        eprintln!(
                            "Something went wrong while deserializing the response to JSON: {}",
                            &err
                        );
                        Err(0x1)
                    }
                }
            }
            Err(err) => {
                eprintln!("Something went wrong while making the request: {}", &err);
                Err(0x1)
            }
        }
    }

    pub fn create_tree(
        &self,
        base_tree: String,
        updated_extensions: Vec<(String, HashMap<String, String>)>,
    ) -> Result<CreateTreeResponse, u8> {
        let mut tree = vec![];
        for updated_extension in updated_extensions {
            for updated_extension_file in updated_extension.1.keys() {
                let file = RequestFile {
                    path: updated_extension_file.to_string(),
                    mode: "100644".to_string(),
                    _type: "blob".to_string(),
                    content: updated_extension
                        .1
                        .get(&updated_extension_file.to_string())
                        .unwrap()
                        .to_string(),
                };

                tree.push(file);
            }
        }

        let body = CreateTreeRequestBody { base_tree, tree };

        let p_body_string = serde_json::to_string(&body);

        let p_response;
        match p_body_string {
            Ok(body_string) => p_response = self
                .client
                .post("https://api.github.com/repos/paperback-community/extensions-test/git/trees")
                .body(body_string)
                .send(),
            Err(err) => {
                eprintln!(
                    "Something went wrong while serializing the request body to JSON: {}",
                    &err
                );
                return Err(0x1);
            }
        }

        match p_response {
            Ok(raw_response) => {
                if raw_response.status() != 201 {
                    eprintln!(
                        "The response was undesired, status code: {}",
                        &raw_response.status(),
                    );
                    return Err(0x1);
                }

                let raw_value_response = raw_response.json::<serde_json::Value>().unwrap();

                match serde_json::from_value::<CreateTreeResponse>(raw_value_response) {
                    Ok(response) => {
                        println!("Created a git tree for the updated extensions");
                        Ok(response)
                    }
                    Err(err) => {
                        eprintln!(
                            "Something went wrong while deserializing the response to JSON: {}",
                            &err
                        );
                        Err(0x1)
                    }
                }
            }
            Err(err) => {
                eprintln!("Something went wrong while making the request: {}", &err);
                Err(0x1)
            }
        }
    }
}
