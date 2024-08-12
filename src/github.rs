use std::sync::Arc;
use chrono::{NaiveDate, NaiveDateTime};
use clokwerk::{AsyncScheduler, TimeUnits};
use reqwest::Client;
use sqlx::PgPool;
use crate::{continue_if, db};
use crate::models::github::{ GithubCommitDetail, GithubTree};
use crate::models::tl_layer::TlLayer;
use crate::prelude::Res;

pub type Year = i32;
pub type Month = u32;
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36";

pub async fn run_task(db: Arc<PgPool>) -> Res {
    let mut scheduler = AsyncScheduler::new();
    scheduler.every(1.hour())
        .run(move || {
            let db = Arc::clone(&db);
            async move {
                run(db.clone()).await.unwrap();
            }
        });

    Ok(())
}
pub async fn run(db: Arc<PgPool>) -> Res {
    let client = Client::new();
    let previous_layers = db::tl_layer::get_ids(&db).await?;

    let octo = octocrab::instance();
    let c = octo.repos("vrumger", "tl").list_commits().per_page(2).sha("master").send().await?;
    let last_commit = c.items.first().unwrap();


    let resp = client.get(format!("https://api.github.com/repos/vrumger/tl/git/trees/{}", last_commit.commit.tree.sha))
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send().await?
        .text().await?;

    let Ok(tree) = serde_json::from_str::<GithubTree>(&resp) else {
        return Err(eyre::Report::msg("failed to parse the tree"));
    };
    let Some(sch_tree) =  tree.tree.into_iter().find(|f|f.path == "schemes") else {
        return Err(eyre::Report::msg("failed to find schemes tree"));
    };

    let resp = client.get(format!("https://api.github.com/repos/vrumger/tl/git/trees/{}", sch_tree.sha))
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send().await?
        .text().await?;
    
    let Ok(layer_list) = serde_json::from_str::<GithubTree>(&resp) else {
        return Err(eyre::Report::msg("failed to parse the layer_list"));
    };
    for layer in layer_list.tree {
        if !layer.path.ends_with(".tl") || layer.path.contains("unknown") {
            continue;
        }
        let spl = layer.path.split(".").collect::<Vec<_>>()[0].trim();
        let Ok(layer_id) = spl.parse::<u32>() else {
            continue;
        };
        let layer_id = layer_id as i32;

        continue_if!(previous_layers.contains(&layer_id));
        log::trace!("fetching layer {layer_id}");

        let (year, month) = find_commit_date(&client, layer.path).await?;
        let date = NaiveDateTime::from(NaiveDate::from_ymd_opt(year, month, 1).unwrap());
        let u = format!("https://raw.githubusercontent.com/vrumger/tl/{}/schemes/{layer_id}.tl",last_commit.sha);
        println!("url: {u}");
        let layer_content = client.get(u)
            .header(reqwest::header::USER_AGENT, USER_AGENT)
            .send().await?
            .text().await?;

        let layer = TlLayer {
            layer_id,
            release_date: date,
            layer: layer_content,
        };
        db::tl_layer::add(&db, layer).await?;
    }

    Ok(())
}

async fn find_commit_date(client: &Client, path: String) -> eyre::Result<(Year, Month)> {
    let resp = client.get(format!("https://api.github.com/repos/vrumger/tl/commits?path=schemes/{path}"))
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send().await?
        .text().await?;

    let Ok(commit_detail) = serde_json::from_str::<Vec<GithubCommitDetail>>(&resp) else {
        log::error!("failed to parse the commit_detail: {resp}");
        return Err(eyre::Report::msg("failed to parse the commit_detail"));
    };
    Ok(commit_detail.first().unwrap().date())
}
