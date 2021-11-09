use crate::db::DB;
use crate::models::client::ClientRequest;
use crate::WebResult;
use warp::{http::StatusCode, reject, reply::json, Reply};

// pub async fn fetch_all_clients_handler(db: DB) -> WebResult<impl Reply> {
//     let tasks = db.get_all_clients_ids().await.map_err(|e| reject::custom(e))?;
//     Ok(json(&tasks))
// }

pub async fn fetch_client_handler(id: String, db: DB) -> WebResult<impl Reply> {
    let tasks = db.find_client(&id).await.map_err(|e| reject::custom(e))?;
    Ok(json(&tasks))
}

pub async fn create_client_handler(body: ClientRequest, db: DB) -> WebResult<impl Reply> {
    db.create_client(&body)
        .await
        .map_err(|e| reject::custom(e))?;
    // TODO: Return the created object
    Ok(StatusCode::CREATED)
}
