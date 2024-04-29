use futures_util::StreamExt;
use mongodb::bson::doc;
use mongodb::Collection;
use reywen::client::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;

pub type MResult<T> = Result<T, mongodb::error::Error>;

#[derive(Clone)]
pub struct Db {
    pub warnings: Collection<Warning>,
    pub c_alias: Collection<CAlias>,
}

#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct CAlias {
    // server ID
    pub _id: String,
    pub channels: HashSet<String>,
}

#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct Warning {
    pub message: Option<String>,
    pub _id: String,
    pub user_id: String,
    pub server_id: String,
}
impl Db {
    pub async fn init() -> Self {
        let uri = env::var("MONGO_URI").unwrap().replace(['"', ], "");

        println!("INIT: DB_URI = {uri}");
        let db = mongodb::Client::with_uri_str(uri)
            .await
            .unwrap()
            .database("autoguard");
        Self {
            warnings: db.collection("warnings"),
            c_alias: db.collection("c_alias"),
        }
    }
}

// alias
impl Db {
    pub async fn c_alias_poll(
        &self,
        channel_id: impl Into<String>,
        client: &Client,
    ) -> MResult<Option<String>> {
        let channel_id = channel_id.into();
        match self
            .c_alias
            .find_one(
                doc! {
                    "channels": channel_id.clone(),
                },
                None,
            )
            .await?
        {
            None => {
                let id = client
                    .channel_fetch(&channel_id)
                    .await
                    .ok()
                    .map(|a| a.server_id());

                let Some(Some(id)) = id else { return Ok(None) };

                self.c_alias_insert(CAlias {
                    _id: id.clone(),
                    channels: [channel_id.clone()].into(),
                })
                .await?;
                Ok(Some(id))
            }
            Some(data) => Ok(Some(data._id)),
        }
    }
    pub async fn c_alias_get_server(
        &self,
        server_id: impl Into<String>,
    ) -> MResult<Option<CAlias>> {
        self.c_alias
            .find_one(doc! {"_id": server_id.into()}, None)
            .await
    }
    pub async fn c_alias_insert(&self, doc: CAlias) -> MResult<()> {
        match self.c_alias_get_server(&doc._id).await? {
            None => {
                self.c_alias.insert_one(doc, None).await?;
            }
            Some(mut data) => {
                data.channels.extend(doc.channels);
                let channels: Vec<String> = data.channels.into_iter().collect();
                let update = doc! { "$set": { "channels": channels } };
                self.c_alias
                    .update_one(doc! {"_id": &doc._id}, update, None)
                    .await?;
            }
        }
        Ok(())
    }
}

// warnings
impl Db {
    pub async fn warn_user(&self, warn: Warning) -> MResult<()> {
        self.warnings.insert_one(warn, None).await.map(|_| {})
    }
    pub async fn warnings(
        &self,
        user_id: impl Into<String>,
        server_id: impl Into<String>,
    ) -> MResult<Vec<Warning>> {
        Ok(self
            .warnings
            .find(
                doc! {"user_id": user_id.into(), "server_id": server_id.into()},
                None,
            )
            .await?
            .filter_map(|a| async move { a.ok() })
            .collect()
            .await)
    }
}
