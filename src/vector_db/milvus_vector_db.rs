use std::borrow::Cow;
use fastembed::Embedding;
use log::debug;
use milvus::client::Client;
use milvus::collection::{Collection, SearchOption};
use milvus::data::FieldColumn;
use milvus::error::Error;
use milvus::index::MetricType;
use milvus::proto::common::MsgType::SearchResult;
use milvus::schema::{CollectionSchema, CollectionSchemaBuilder, FieldSchema};
use milvus::value::Value;
use crate::vector_db::VectorDbFunc;

pub struct Milvus {
    url: String,
    client: Client,
    schema: CollectionSchema,
    collection: Collection
}

impl Milvus {
    pub async fn new(url: String) -> Result<Self, Box<dyn std::error::Error>> {
        let schema = CollectionSchemaBuilder::new(Self::SCHEMA_NAME, "")
            .add_field(FieldSchema::new_primary_varchar(
                Self::ID_FIELD,
                "",
                false,
                1000
            ))
            .add_field(FieldSchema::new_float_vector(
                Self::VECTOR_FIELD,
                "",
                512
            ))
            .add_field(FieldSchema::new_varchar(
                Self::CONTENT_FIELD,
                "",
                65535
            ))
            .add_field(FieldSchema::new_varchar(
                Self::METADATA_FIELD,
                "",
                65535
            ))
            .build()?;

        let client = Client::new(url.clone()).await?;
        let exists = client.has_collection(Self::SCHEMA_NAME).await?;

        let collection: Collection;
        if !exists {
            debug!("向量集合不存在，开始新建");
            collection = client.create_collection(schema.clone(), None).await?;
        } else {
            debug!("使用已有向量集合");
            collection = client.get_collection(Self::SCHEMA_NAME).await?;
        }
        collection.load_partition_list().await?;

        Ok(Milvus {
            url,
            client,
            schema,
            collection
        })
    }
}

impl VectorDbFunc for Milvus {
    async fn insert(&self, id: String, content: String, vector: Vec<f32>, metadata: String) {
        let data = vec![
            FieldColumn::new(self.schema.get_field(Self::ID_FIELD).unwrap(), vec![id]),
            FieldColumn::new(self.schema.get_field(Self::VECTOR_FIELD).unwrap(), vector),
            FieldColumn::new(self.schema.get_field(Self::CONTENT_FIELD).unwrap(), vec![content]),
            FieldColumn::new(self.schema.get_field(Self::METADATA_FIELD).unwrap(), vec![metadata]),
        ];
        let r = self.collection.insert(data, None).await.unwrap();
        println!("{:?}", r)
    }

    async fn search(&self, vector: Vec<f32>, top_k: i32) -> String {
        let option = SearchOption::new();
        let value = vec![Value::FloatArray(Cow::from(vector))];
        let result = self.collection.search(value, Self::VECTOR_FIELD, top_k, MetricType::L2,
                                          [Self::CONTENT_FIELD, Self::METADATA_FIELD], &option)
            .await.expect("TODO: panic message");
        for i in result.into_iter() {
            println!("{:?}", i.field);
            println!("{:?}", i.score);
            println!("{:?}", i.size);
        }
        String::new()
    }
}