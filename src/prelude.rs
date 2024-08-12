use std::str::FromStr;
use dotenv::var;
use log::LevelFilter;
use meilisearch_sdk::task_info::TaskInfo;
use meilisearch_sdk::tasks::TaskType;

pub const DEFAULT_TASK_INFO: TaskInfo = TaskInfo { task_uid: 0, index_uid: None, enqueued_at: time::OffsetDateTime::UNIX_EPOCH, status: String::new(), update_type: TaskType::IndexCreation { details: None } };
pub type Res = eyre::Result<()>;
#[macro_export()]
macro_rules! continue_if {
        ($cond:expr) => { if $cond { continue; } };
    }
#[macro_export()]
macro_rules! return_if {
        ($cond:expr) => { if $cond { return Ok(()); } };
    }

pub async fn init_logger() -> Res {
    if std::path::PathBuf::from("logs.txt").exists() {
        tokio::fs::write("logs.txt", "").await?;
    }
    fern::Dispatch::new()
        .format(|out, message, record| {
            let datetime: chrono::DateTime<chrono::Utc> = std::time::SystemTime::now().into();
            let formatted_time = datetime.format_with_items(chrono::format::StrftimeItems::new("%H:%M:%S")).to_string();
            out.finish(format_args!(
                "[{} - {} ({})] {}",
                formatted_time,
                record.level(),
                record.target(),
                message
            ))
        })
        .level(LevelFilter::from_str(&var("LOG")?)?)
        .level_for("tower", LevelFilter::Warn)
        .level_for("reqwest", LevelFilter::Warn)
        .level_for("tokio", LevelFilter::Debug)
        .level_for("hyper_util", LevelFilter::Error)
        .level_for("hyper", LevelFilter::Error)
        .level_for("tracing", LevelFilter::Error)
        .level_for("os_info", LevelFilter::Error)
        .level_for("hyper_rustls", LevelFilter::Error)
        .level_for("rustls", LevelFilter::Error)
        .chain(std::io::stdout())
        .chain(fern::log_file("logs.txt")?).apply()?;

    Ok(())
}