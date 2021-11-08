use std::str::FromStr;

use crate::error;
use crate::error::Error::*;
use crate::models::client::{ClientRequest, ClientResponse};
use bson::serde_helpers::serialize_object_id_as_hex_string;
use bson::Document;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{self, doc, Bson};
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::db::{DB, DB_NAME};

impl DB {
    fn get_clients_collection(&self) -> Collection<Document> {
        self.client.database(DB_NAME).collection("clients")
    }

    pub fn doc_to_client(&self, doc: &Document) -> Result<ClientResponse, error::Error> {
        let id = doc.get_object_id("_id")?;
        let name = doc.get_str("name")?;
        let created_at = doc.get_datetime("created_at")?;
        let updated_at = doc.get_datetime("updated_at")?;

        let client = ClientResponse {
            _id: id.to_hex(),
            name: name.to_owned(),
            created_at: created_at.to_chrono().to_rfc3339(),
            updated_at: updated_at.to_chrono().to_rfc3339(),
        };

        Ok(client)
    }

    pub async fn find_client(&self, id: &str) -> Result<ClientResponse, error::Error> {
        let oid = ObjectId::parse_str(id).map_err(|_| InvalidIDError(id.to_owned()))?;
        let query = doc! {
            "_id": oid,
        };
        let document = self
            .get_clients_collection()
            .find_one(query, None)
            .await
            .map_err(MongoQueryError)?;

        if document.is_none() {
            // return error::Err(warp::reject::not_found());
            return Err(ObjNotFound);
        }

        let result = self.doc_to_client(&document.expect("Document not found"))?;

        Ok(result)
    }

    pub async fn create_client(&self, _entry: &ClientRequest) -> Result<(), error::Error> {
        self.get_clients_collection()
            .insert_one(
                doc! {
                "name": _entry.name.clone(),
                },
                None,
            )
            .await
            .map_err(MongoQueryError)?;

        Ok(())
    }

    pub async fn create_many_clients(
        &self,
        _entry: Vec<mongodb::bson::Document>,
    ) -> Result<(), error::Error> {
        self.get_clients_collection()
            .insert_many(_entry, None)
            .await
            .map_err(MongoQueryError)?;
        Ok(())
    }

    pub async fn delete_client(&self, id: &str) -> Result<(), error::Error> {
        let oid = ObjectId::parse_str(id).map_err(|_| InvalidIDError(id.to_owned()))?;
        let query = doc! {
            "_id": oid,
        };
        self.get_clients_collection()
            .delete_one(query, None)
            .await
            .map_err(MongoQueryError)?;

        Ok(())
    }

    pub async fn delete_all_clients(&self) -> Result<(), error::Error> {
        self.get_clients_collection()
            .delete_many(doc! {}, None)
            .await
            .map_err(MongoQueryError)?;

        Ok(())
    }

    pub async fn get_all_clients_ids(&self) -> Result<Vec<Bson>, error::Error> {
        let clients_ids = self
            .get_clients_collection()
            .distinct("_id", None, None)
            .await
            .map_err(MongoQueryError)?;

        // The `.distinct` method returns a Vec<Bson, Global>
        // I would like to convert clients_ids from Vec<Bson, Global> to Vec<String>.
        // I tried everything I could but can't figure this one out.

        // Should I iterate over each item in clients_ids and convert them individualy to String?
        // In that cause how can I do that?
        // println!("GOT HERE");
        // let string_vec: Vec<String>;
        for item in &clients_ids {
            // let x = bson::from_bson<String>(item);
            // let x = item;
            // let x = item as bson::oid::ObjectId;

            // serialize_object_id_as_hex_string(item);
            // let x: std::result::Result<String, mongodb::bson::de::Error> = bson::from_bson(item);

            // TODO
            // convert item from Bson to String
            // push to string_vec.
            println!("Bson Item {:?}", item); // prints ObjectId("61842402bcfb83e1b27b6d85")
        }

        Ok(clients_ids)
        // Ok(string_vec)
    }
}