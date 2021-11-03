use crate::db::db::DB;
use crate::models::task::TaskRequest;
use crate::WebResult;
use warp::{http::StatusCode, reject, reply::json, Reply};

pub async fn fetch_all_tasks_handler(db: DB) -> WebResult<impl Reply> {
    let tasks = db.get_all_tasks().await.map_err(|e| reject::custom(e))?;
    Ok(json(&tasks))
}

pub async fn fetch_task_handler(id: String, db: DB) -> WebResult<impl Reply> {
    let tasks = db.find_task(&id).await.map_err(|e| reject::custom(e))?;
    Ok(json(&tasks))
}

pub async fn create_task_handler(body: TaskRequest, db: DB) -> WebResult<impl Reply> {
    db.create_task(&body).await.map_err(|e| reject::custom(e))?;
    // TODO: Return the created object
    Ok(StatusCode::CREATED)
}

pub async fn delete_all_tasks_handler(db: DB) -> WebResult<impl Reply> {
    db.delete_all_tasks().await.map_err(|e| reject::custom(e))?;
    // TODO: Return the delete objects
    Ok(StatusCode::OK)
}

pub async fn edit_task_handler(id: String, body: TaskRequest, db: DB) -> WebResult<impl Reply> {
    db.edit_task(&id, &body)
        .await
        .map_err(|e| reject::custom(e))?;
    // TODO: Return the edited object
    Ok(StatusCode::OK)
}
pub async fn delete_task_handler(id: String, db: DB) -> WebResult<impl Reply> {
    db.delete_task(&id).await.map_err(|e| reject::custom(e))?;
    // Return the deleted object
    Ok(StatusCode::OK)
}
