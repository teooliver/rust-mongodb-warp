use crate::models::task::{TaskAfterGrouped, TaskRequest, TaskResponse, TasksGroupedByDate};
use crate::{error::Error::*, Result};
use chrono::prelude::*;
use futures::StreamExt;
use mongodb::bson::{self, Bson};
use mongodb::bson::{doc, document::Document, oid::ObjectId};
use mongodb::Collection;

use super::{DB, DB_NAME};

impl DB {
    fn get_tasks_collection(&self) -> Collection<Document> {
        self.client.database(DB_NAME).collection("tasks")
    }

    fn doc_to_task(&self, doc: &Document) -> Result<TaskResponse> {
        let id = doc.get_object_id("_id")?;
        let name = doc.get_str("name")?;
        let initial_time = doc.get_datetime("initial_time")?;
        let end_time = doc.get_datetime("end_time")?;
        let project = doc.get_object_id("project")?;
        let created_at = doc.get_datetime("created_at")?;
        let updated_at = doc.get_datetime("updated_at")?;

        // if project.is_none() {
        //     // return error::Err(warp::reject::not_found());
        //     return Err(ObjNotFound);
        // }

        let task = TaskResponse {
            _id: id.to_hex(),
            name: name.to_owned(),
            // initial_time: initial_time.to_string(),
            initial_time: initial_time
                .to_chrono()
                .to_rfc3339_opts(SecondsFormat::Secs, true),
            // end_time: end_time.to_string(),
            end_time: end_time
                .to_chrono()
                .to_rfc3339_opts(SecondsFormat::Secs, true),
            project: Some(project.to_hex()),
            created_at: created_at
                .to_chrono()
                .to_rfc3339_opts(SecondsFormat::Secs, true),
            updated_at: updated_at
                .to_chrono()
                .to_rfc3339_opts(SecondsFormat::Secs, true),
        };

        Ok(task)
    }

    pub async fn get_all_tasks(&self) -> Result<Vec<TaskResponse>> {
        let mut cursor = self
            .get_tasks_collection()
            .find(None, None)
            .await
            .map_err(MongoQueryError)?;

        let mut result: Vec<TaskResponse> = Vec::new();

        while let Some(doc) = cursor.next().await {
            result.push(self.doc_to_task(&doc?)?);
        }

        Ok(result)
    }

    pub async fn get_tasks_grouped_by_date(&self) -> Result<Vec<TasksGroupedByDate>> {
        let lookup_projects = doc! {
            "$lookup": {
                "from": "projects",
                "localField": "project",
                "foreignField": "_id",
                "as": "project",
            }
        };
        let lookup_clients = doc! {
            "$lookup": {
              "from": "clients",
              "localField": "project.client",
              "foreignField": "_id",
              "as": "client",
            }
        };

        let project = doc! {
              "$project": {
                    "_id": "$_id",
                    "name": "$name",
                    "initial_time": "$initial_time",
                    "end_time": "$end_time",
                    "project": { "$arrayElemAt": ["$project.name", 0] },
                    "project_color": { "$arrayElemAt": ["$project.color", 0] },
                    "client": { "$arrayElemAt": ["$client.name", 0] },
                },
        };

        let group = doc! {
            "$group": {
                "_id": { "$dateToString": { "format": "%Y-%m-%d", "date": "$initial_time" } },
                "tasks": { "$push": "$$ROOT" },
                "totalTime": {
                    "$sum":{
                        "$divide": [{ "$subtract": ["$end_time", "$initial_time"] }, 1000],
                    },
                },
            },
        };

        let sort = doc! {
             "$sort": {
                "_id": -1,
            },
        };

        let pipeline = vec![lookup_projects, lookup_clients, project, group, sort];

        let mut cursor = self
            .get_tasks_collection()
            .aggregate(pipeline, None)
            .await?;

        let mut grouped_tasks_vec: Vec<TasksGroupedByDate> = vec![];
        let mut tasks_vec: Vec<TaskAfterGrouped> = vec![];

        while let Some(doc) = cursor.next().await {
            let doc_real = doc.unwrap();
            let id = doc_real.get_str("_id")?;
            let tasks = doc_real.get_array("tasks")?;
            let total_time = doc_real.get_i32("totalTime").unwrap_or(0);

            for item in tasks {
                let task_document = item.as_document().unwrap();

                let _id = task_document.get_object_id("_id")?.to_hex();
                let name = task_document.get_str("name")?.to_string();
                let initial_time = task_document
                    .get_datetime("initial_time")?
                    .to_chrono()
                    .to_rfc3339_opts(SecondsFormat::Secs, true);
                let end_time = task_document
                    .get_datetime("end_time")?
                    .to_chrono()
                    .to_rfc3339_opts(SecondsFormat::Secs, true);
                let project = task_document.get_str("project").ok();

                let project_color = task_document.get_str("project_color").ok();

                let client = task_document.get_str("client").ok();

                fn project_name(proj: Option<&str>) -> Option<String> {
                    match proj {
                        Some(proj) => Some(proj.to_string()),
                        None => None,
                    }
                }

                let task = TaskAfterGrouped {
                    _id,
                    name,
                    initial_time,
                    end_time,
                    project: project_name(project),
                    project_color: project_name(project_color),
                    client: project_name(client),
                };

                tasks_vec.push(task);
            }
            let grouped_tasks = TasksGroupedByDate {
                _id: id.to_string(),
                tasks: tasks_vec.to_owned(),
                total_time: total_time,
            };

            grouped_tasks_vec.push(grouped_tasks);
        }

        Ok(grouped_tasks_vec.to_vec())
    }

    pub async fn find_task(&self, id: &str) -> Result<TaskResponse> {
        let oid = ObjectId::parse_str(id).map_err(|_| InvalidIDError(id.to_owned()))?;
        let query = doc! {
            "_id": oid,
        };
        let document = self
            .get_tasks_collection()
            .find_one(query, None)
            .await
            .map_err(MongoQueryError)?;

        if document.is_none() {
            // return error::Err(warp::reject::not_found());
            return Err(ObjNotFound);
        }

        let result = self.doc_to_task(&document.unwrap())?;

        Ok(result)
    }

    pub async fn create_task(&self, _entry: &TaskRequest) -> Result<Bson> {
        let initial_time: chrono::DateTime<Utc> = _entry.initial_time.parse().unwrap();
        // let initial_time: bson::DateTime = chrono_dt.into();

        let end_time: chrono::DateTime<Utc> = _entry.end_time.parse().unwrap();
        // let end_time: bson::DateTime = chrono_endtime.into();

        let project: Option<ObjectId> = _entry.project.clone();

        let new_task = self
            .get_tasks_collection()
            .insert_one(
                doc! {
                "name": _entry.name.clone(),
                "initial_time": initial_time,
                "end_time": end_time,
                "project": Some(project),
                "created_at": chrono::Utc::now().clone(),
                "updated_at": chrono::Utc::now().clone(),
                },
                None,
            )
            .await
            .map_err(MongoQueryError)?;

        Ok(new_task.inserted_id)
    }

    pub async fn edit_task(&self, id: &str, _entry: &TaskRequest) -> Result<()> {
        let oid = ObjectId::parse_str(id).map_err(|_| InvalidIDError(id.to_owned()))?;

        let chrono_dt: chrono::DateTime<Utc> = _entry.initial_time.parse().unwrap();
        let initial_time: bson::DateTime = chrono_dt.into();
        let chrono_endtime: chrono::DateTime<Utc> = _entry.end_time.parse().unwrap();
        let end_time: bson::DateTime = chrono_endtime.into();
        let project: Option<ObjectId> = _entry.project.clone();

        let query = doc! {
            "_id": oid,
        };

        let doc = doc! {
            "$set": {
                "name": _entry.name.clone(),
                "initial_time": initial_time.clone(),
                "end_time": end_time.clone(),
                "project": project,
                "updated_at": chrono::Utc::now().clone(),
                }
        };

        self.get_tasks_collection()
            .find_one_and_update(query, doc, None)
            .await
            .map_err(MongoQueryError)?;

        Ok(())
    }
    pub async fn delete_all_tasks(&self) -> Result<()> {
        self.get_tasks_collection()
            .delete_many(doc! {}, None)
            .await
            .map_err(MongoQueryError)?;

        Ok(())
    }

    pub async fn delete_task(&self, id: &str) -> Result<()> {
        let oid = ObjectId::parse_str(id).map_err(|_| InvalidIDError(id.to_owned()))?;
        let query = doc! {
            "_id": oid,
        };
        self.get_tasks_collection()
            .delete_one(query, None)
            .await
            .map_err(MongoQueryError)?;

        Ok(())
    }

    pub async fn create_many_tasks(&self, _entry: Vec<mongodb::bson::Document>) -> Result<()> {
        self.get_tasks_collection()
            .insert_many(_entry, None)
            .await
            .map_err(MongoQueryError)?;
        Ok(())
    }
}
