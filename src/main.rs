// arg parse list | clone
// flags: directory
// envvar: LANG
// cope with connection issues (connected to proxy)
// cope with github token not being present. ask at initial startup
// cope with already present repo's. should not be cloned again

use std::error::Error;
use std::fmt;
use std::fs;
use std::process::Command;

use reqwest::{blocking, header::USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Value;

type HttpResult<T> = Result<T, reqwest::Error>;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GitRepo {
    name: String,
    ssh_url: String,
    lang: String,
}

impl From<Value> for GitRepo {
    fn from(v: Value) -> Self {
        let name = v["full_name"].to_string().replace('"', "");
        let ssh_url = v["ssh_url"].to_string();
        let lang = v["language"].to_string().replace('"', "");

        Self {
            name,
            ssh_url,
            lang,
        }
    }
}

impl fmt::Display for GitRepo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let repo = format!(
            "- {:<30} {}",
            self.name.split('/').nth(1).unwrap_or(""),
            self.lang
        );
        write!(f, "{}", repo)
    }
}

#[derive(Debug)]
struct GitRepos {
    repos: Vec<GitRepo>,
}

impl fmt::Display for GitRepos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let longest = self.repos.iter().map(|r| r.name.len()).max().unwrap_or(0);
        let mut text = format!("Repo Name{}Language:\n", " ".repeat(longest - 12));
        for r in self.repos.iter() {
            text.push_str(&format!("{:>width$}\n", r, width = longest));
        }

        write!(f, "{}", text)
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
        Ok(token.trim().to_string())
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
        .bearer_auth(token)
        .send()
}

fn mkdir_p(path: &str) -> Result<&str, std::io::Error> {
    fs::create_dir_all(path)?;

    Ok(path)
}

fn get_repo_list() -> Result<GitRepos, Box<dyn Error>> {
    let token = get_api_token().expect("failed to get api token"); //handle absense
    let url = format!(
        "https://api.github.com/users/tlafebre/repos?access_token={}",
        token
    );

    let res = http_get(url)?.text()?;
    let v: Vec<Value> = serde_json::from_str(&res)?;
    let repos: Vec<GitRepo> = v.into_iter().map(|o| GitRepo::from(o)).collect();

    Ok(GitRepos { repos })
}

fn main() {
    let repo_list = get_repo_list();

    println!("{}", repo_list.unwrap());
}
