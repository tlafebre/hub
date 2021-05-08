//arg parse

use reqwest::{blocking, header::USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::process::Command;

type HttpResult<T> = Result<T, reqwest::Error>;

#[derive(Debug, Serialize, Deserialize)]
struct GitRepo {
    clone_url: String,
}

fn get_api_token(location: String) -> Result<String, String> {
    if let Ok(token) = fs::read_to_string(location) {
        Ok(token)
    } else {
        Err(String::from("failed to read token"))
    }
}

fn git_clone(repo: Value) {
    let url = repo["clone_url"].as_str().unwrap();
    Command::new("git")
        .arg("clone")
        .arg(url)
        .status()
        .expect("failed to clone");
}

fn http_get(url: String) -> HttpResult<blocking::Response> {
    let client = blocking::Client::new();

    client.get(url).header(USER_AGENT, "my rust program").send()
}

fn mkdir_p(path: &str) -> Result<&str, std::io::Error> {
    fs::create_dir_all(path)?;

    Ok(path)
}

fn get_repo_list() -> String {
    let path = String::from("/home/tjeerd/.git_api_token");
    let token = get_api_token(path).expect("failed to get api token"); //handle absense
    let url = format!(
        "https://api.github.com/users/tlafebre/repos?access_token={}",
        token
    );

    let res = http_get(url);

    res.unwrap().text().unwrap()
}

fn main() {
    let repo_list = get_repo_list();
    let repo_dir = mkdir_p("repos").unwrap();

    let v: Vec<Value> = serde_json::from_str(&repo_list).unwrap();

    if std::env::set_current_dir(repo_dir).is_ok() {
        for o in v.into_iter() {
            if &o["language"] == "Rust" {
                git_clone(o)
            }
        }
    } else {
        eprint!("Something went wrong");
    }
}
