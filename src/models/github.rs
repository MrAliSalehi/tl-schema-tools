use chrono::{Datelike, DateTime};
use serde::{Deserialize, Serialize};
use crate::github::{Month, Year};

#[derive(Serialize, Deserialize)]
pub struct GithubCommitDetail {
    #[serde(rename = "commit")]
    commit: Commit,
}


#[derive(Serialize, Deserialize)]
pub struct Commit {
    #[serde(rename = "committer")]
    committer: CommitAuthor,

}

#[derive(Serialize, Deserialize)]
pub struct CommitAuthor {
    #[serde(rename = "date")]
    date: String,
}


impl GithubCommitDetail {
    pub fn date(&self) -> (Year, Month) {
        let parse = DateTime::parse_from_rfc3339(&self.commit.committer.date).unwrap();
        (parse.year(), parse.month())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubTree {
    #[serde(rename = "tree")]
    pub tree: Vec<InnerTree>,

}

#[derive(Serialize, Deserialize, Debug)]
pub struct InnerTree {
    pub path: String,
    pub url: String,
    pub sha: String,
}

