use sqlx::{PgPool, query, query_as};
use crate::models::tl_layer::TlLayer;
use crate::prelude::Res;

pub async fn get_all(db: &PgPool) -> eyre::Result<Vec<TlLayer>> {
    Ok(query_as!(TlLayer,"select * from tl_layer").fetch_all(db).await?)
}
pub async fn get_ids(db: &PgPool) -> eyre::Result<Vec<i32>> {
    Ok(query!("select layer_id from tl_layer").fetch_all(db).await?.into_iter().map(|f| f.layer_id).collect())
}
pub async fn add(db: &PgPool, tl_layer: TlLayer) -> Res {
    query!("insert into tl_layer values($1,$2,$3)",tl_layer.layer_id,tl_layer.layer,tl_layer.release_date).execute(db).await?;
    Ok(())
}