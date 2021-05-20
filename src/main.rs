//arg parse
//cope with connection issues (connected to proxy)
//cope with github token not being present. ask at initial startup
//cope with already present repo's. should not be cloned again

use reqwest::{blocking, header::USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::process::Command;
use std::str::FromStr;

type HttpResult<T> = Result<T, reqwest::Error>;

#[derive(Debug, Serialize, Deserialize)]
struct GitRepo {
    ssh_url: String,
}

impl FromStr for GitRepo {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(GitRepo {
            ssh_url: String::from(s),
        })
    }
}

impl GitRepo {
    fn git_clone(self) {
        Command::new("git")
            .arg("clone")
            .arg(self.ssh_url)
            .status()
            .expect("failed to clone");
    }
}

fn get_api_token() -> Result<String, String> {
    let path = String::from("/home/tjeerd/.git_api_token");
    if let Ok(token) = fs::read_to_string(path) {
        Ok(token)
    } else {
        Err(String::from("failed to read token"))
    }
}

fn http_get(url: String) -> HttpResult<blocking::Response> {
    let client = blocking::Client::new();
    let token = get_api_token().expect("failed to get api token"); //handle absense
    client
        .get(url)
        .header(USER_AGENT, "my rust program")
        .header("Authorization", token)
        .send()
}

fn mkdir_p(path: &str) -> Result<&str, std::io::Error> {
    fs::create_dir_all(path)?;

    Ok(path)
}

fn get_repo_list() -> String {
    let token = get_api_token().expect("failed to get api token"); //handle absense
    let url = format!(
        "https://api.github.com/users/tlafebre/repos?access_token={}",
        token
    );

    let res = http_get(url);

    res.unwrap().text().unwrap()
}

fn main() {
    let repo_list = get_repo_list();
    let repo_dir = mkdir_p("/home/tjeerd/repos").unwrap();

    let v: Vec<Value> = serde_json::from_str(&repo_list).unwrap();

    if std::env::set_current_dir(repo_dir).is_ok() {
        for o in v.into_iter() {
            let r = GitRepo::from_str(o["ssh_url"].as_str().unwrap());
            if &o["language"] == "Rust" {
                r.unwrap().git_clone()
            }
        }
    } else {
        eprint!("Something went wrong");
    }
}
