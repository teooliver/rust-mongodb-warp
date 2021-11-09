use crate::db::DB;
use crate::error::Error::InvalidIDError;

use crate::{models::project::ProjectRequest, WebResult};
use fake::{self, Fake};
use mongodb::bson::{doc, oid::ObjectId};
use rand::Rng;
use warp::{http::StatusCode, Reply};

pub const PROJECT_COLORS: [&str; 10] = [
    "#61e294ff",
    "#7bcdbaff",
    "#9799caff",
    "#bd93d8ff",
    "#b47aeaff",
    "#d3d5d4ff",
    "#a2c5acff",
    "#9db5b2ff",
    "#878e99ff",
    "#7f6a93ff",
];

// pub const CLIENT_NAMES = [];
// pub const PROJECT_NAMES = [];
// pub const TASK_NAMES = [];

pub fn generate_clients_data(amount: u8) -> Vec<mongodb::bson::Document> {
    let mut clients: Vec<mongodb::bson::Document> = vec![];

    for _n in 1..amount {
        clients.push(doc! {
            "name": fake::faker::company::en::CompanyName().fake::<String>().to_string(),
            "created_at": chrono::Utc::now().clone(),
            "updated_at": chrono::Utc::now().clone(),
        });
    }

    clients
}

pub fn create_project(clients_ids: Vec<String>) -> ProjectRequest {
    let rng_color_index = rand::thread_rng().gen_range(0..(PROJECT_COLORS.len() - 1));
    let rng_client_index = rand::thread_rng().gen_range(0..(clients_ids.len() - 1));

    let client_id = ObjectId::parse_str(clients_ids[rng_client_index].to_string())
        .map_err(|_| InvalidIDError(clients_ids[rng_client_index].to_owned()))
        .unwrap();

    let new_project = ProjectRequest {
        client: client_id,
        name: fake::faker::company::en::CompanyName().fake(),
        color: PROJECT_COLORS[rng_color_index].to_string(),
        estimate: "".to_string(),
        status: "".to_string(),
    };

    new_project
}

pub fn generate_projects_data(
    amount: u8,
    clients_ids: Vec<String>,
) -> Vec<mongodb::bson::Document> {
    let mut projects: Vec<mongodb::bson::Document> = vec![];
    let project = create_project(clients_ids);

    for _n in 1..amount {
        projects.push(doc! {
            "client": project.client.to_string(),
            "name": project.name.to_string(),
            "color": project.color.to_string(),
            "estimate": project.estimate.to_string(),
            "status": project.status.to_string(),
        });
    }

    projects
}

pub async fn seed_clients(db: &DB) -> WebResult<impl Reply> {
    db.delete_all_clients().await?;
    db.delete_all_projects().await?;
    db.delete_all_tasks().await?;

    db.create_many_clients(generate_clients_data(10)).await?;

    Ok(StatusCode::OK)
}

pub async fn seed_projects(db: &DB) -> WebResult<impl Reply> {
    db.delete_all_projects().await?;
    db.delete_all_tasks().await?;

    let client_ids = db.get_all_clients_ids().await?;

    db.create_many_projects(generate_projects_data(10, client_ids))
        .await?;

    Ok(StatusCode::OK)
}
