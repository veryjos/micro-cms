use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::entity::{Entity, FieldData};
use crate::cache::Cache;

#[derive(Default, Debug, Deserialize)]
pub struct QuerySortOptions {
    by: String
}

#[derive(Default, Debug, Deserialize)]
pub struct QueryEntity {
    pub sort: Option<QuerySortOptions>,
    pub filter: Option<HashMap<String, String>>,
    pub fields: Vec<String>
}

#[derive(Default, Debug, Deserialize)]
pub struct Query {
    #[serde(flatten)]
    pub entities: HashMap<String, QueryEntity>
}

#[derive(Default, Debug, Serialize)]
pub struct QueryResult<'a> {
    #[serde(flatten)]
    pub groups: HashMap<&'a str, Vec<QueryResultEntity<'a>>>
}

#[derive(Default, Debug, Serialize)]
pub struct QueryResultEntity<'a> {
    pub id: &'a str,

    #[serde(flatten)]
    pub fields: HashMap<&'a str, QueryResultFieldData<'a>>
}

impl<'a> QueryResultEntity<'a> {
    pub fn filter_fields(&mut self, fields: &Vec<&str>) {
        self.fields.retain(|name, _| fields.contains(name));
    }
}

impl<'a> From<(&'a str, &'a Entity)> for QueryResultEntity<'a> {
    fn from(data: (&'a str, &'a Entity)) -> Self {
        QueryResultEntity {
            id: data.0,
            fields: data.1.fields.iter()
                .map(|(name, data)| (name.as_str(), data.into()))
                .collect()
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum QueryResultFieldData<'a> {
    Str(&'a String),
    Bin(&'a Vec<u8>),
    Num(&'a f64)
}

impl<'a> From<&'a FieldData> for QueryResultFieldData<'a> {
    fn from(field_data: &'a FieldData) -> QueryResultFieldData<'a> {
        match field_data {
            FieldData::Str(ref d) => QueryResultFieldData::Str(d),
            FieldData::Bin(ref d) => QueryResultFieldData::Bin(d),
            FieldData::Num(ref d) => QueryResultFieldData::Num(d)
        }
    }
}

impl Query {
    pub fn evaluate<'a>(&'a self, cache: &'a Cache) -> QueryResult<'a> {
        let mut result = QueryResult::default();

        for (ty, query_ent) in &self.entities {
            let group = cache.get_group(ty);
            let mut group_result = vec!();

            for (id, entity) in group.entities.iter()
                .filter(|(id, _)| query_ent.filter.as_ref()
                        .map_or_else(
                            || true,
                            |f| f.get("id").map_or(false, |ref s| s == id))
                        )
            {
                let mut query_result_entity: QueryResultEntity = (id.as_str(), entity).into();
                query_result_entity.filter_fields(&query_ent.fields.iter()
                    .map(|s| s.as_ref())
                    .collect());

                group_result.push(query_result_entity);
            }

            // Sort
            if let Some(sort_options) = &query_ent.sort {
                group_result.sort_unstable_by(|a: &QueryResultEntity, b: &QueryResultEntity| {
                    let (lhs, rhs) = match (b.fields.get(sort_options.by.as_str()).unwrap(), a.fields.get(sort_options.by.as_str()).unwrap()) {
                        (QueryResultFieldData::Str(lhs), QueryResultFieldData::Str(rhs)) => (lhs, rhs),
                        _ => panic!("Can only sort string types")
                    };

                    rhs.cmp(lhs)
                });
            }

            result.groups.insert(&ty, group_result);
        }

        result
    }
}
