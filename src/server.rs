use std::sync::{Arc, RwLock};

use rocket::http::Status;

use crate::entity::FieldData;
use crate::providers::{Provider, FsProvider};
use crate::query::{Query, QueryResultEntity};

const MAX_QUERY_LEN: u64 = 2048;

pub struct ServerConfig {
    pub bind_address: String,
    pub port: u16,
}

pub struct Server {
    _config: ServerConfig,
}

type ProviderState<P> = Arc<RwLock<P>>;

#[rocket::get("/")]
fn get_index() -> String {
    let version = env!("CARGO_PKG_VERSION");

    format!("micro-cms version {version}")
}

#[rocket::get("/ent/<ty>/<ent_id>?<fields>")]
fn get_entity(
    ty: String,
    ent_id: String,
    fields: Option<String>,
    provider: rocket::State<ProviderState<FsProvider>>
) -> Result<Vec<u8>, Status> {
    let provider = match provider.read() {
        Ok(p) => p,
        _ => return Err(Status::BadRequest)
    };

    let cache = match provider.read_cache() {
        Ok(p) => p,
        _ => return Err(Status::BadRequest)
    };

    let group = cache.get_group(&ty);

    let query_tup = (ent_id.as_str(), group.get_entity(&ent_id));
    let mut response_ent: QueryResultEntity = query_tup.into();

    // Filter the fields that we got back
    if let Some(fields_str) = fields {
        let fields: Vec<_> = fields_str.split(',').collect();
        response_ent.filter_fields(&fields);
    }

    let response_str = serde_json::to_string(&response_ent).unwrap();
    Ok(response_str.into())
}

#[rocket::get("/ent/<ty>/<ent_id>/<field_name>")]
fn get_field(
    ty: String,
    ent_id: String,
    field_name: String,
    provider: rocket::State<ProviderState<FsProvider>>
) -> Result<Vec<u8>, Status> {
    let provider = match provider.read() {
        Ok(p) => p,
        _ => return Err(Status::BadRequest)
    };

    let cache = match provider.read_cache() {
        Ok(p) => p,
        _ => return Err(Status::BadRequest)
    };

    // Get everything before dot path seperator
    let tokens: Vec<&str> = field_name.split('.').collect();
    let field_name = tokens[0];

    let group = cache.get_group(&ty);
    let ent = group.get_entity(&ent_id);
    let field_data = ent.fields.get(field_name);

    match field_data {
        Some(FieldData::Str(d)) => Ok(d.clone().into()),
        Some(FieldData::Bin(d)) => Ok(d.clone()),
        _ => Err(Status::BadRequest)
    }
}

#[rocket::post("/query", data = "<input_data>")]
fn query(
    input_data: rocket::Data,
    provider: rocket::State<ProviderState<FsProvider>>
) -> Result<Vec<u8>, Status>  {
    let input = {
        use std::io::Read;

        let mut output = String::new();
        input_data.open().take(MAX_QUERY_LEN).read_to_string(&mut output).expect("Failed to read data stream into string");
        output
    };

    let provider = match provider.read() {
        Ok(p) => p,
        _ => return Err(Status::BadRequest)
    };

    let cache = match provider.read_cache() {
        Ok(p) => p,
        _ => return Err(Status::BadRequest)
    };

    let query: Query = serde_json::from_str(&input).unwrap();

    // Evaluate the query
    let result = query.evaluate(&cache);

    Ok(serde_json::to_string(&result).unwrap().into())
}

impl Server {
    pub fn new(config: ServerConfig) -> Server {
        Server {
            _config: config
        }
    }

    pub fn listen(&self, provider: FsProvider) {
        let provider_arc = Arc::new(RwLock::new(provider));

        rocket::ignite()
            .manage(Arc::clone(&provider_arc))
            .mount("/", rocket::routes![get_index])
            .mount("/", rocket::routes![query])
            .mount("/", rocket::routes![get_field])
            .mount("/", rocket::routes![get_entity])
            .launch();

        // Join the provider before the server dies
        // TODO: this will go boom if there's multiple strong arcs
        if let Ok(provider) = Arc::try_unwrap(provider_arc) {
            provider.into_inner().unwrap().join();
        }
    }
}
