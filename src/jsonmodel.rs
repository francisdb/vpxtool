use serde_json::json;

use crate::{tableinfo::TableInfo, vpx::collection::Collection};

pub fn table_json(table_info: &TableInfo) -> serde_json::Value {
    // TODO convert to a serde
    // TODO add missing data

    let base = json!({
        "name": table_info.table_name,
        "authorName": table_info.author_name,
        "authorEmail": table_info.author_email,
        "authorWebsite": table_info.author_website,
        "description": table_info.table_description,
        "blurb": table_info.table_blurb,
        "rules": table_info.table_rules,
        "version": table_info.table_version,
        "releaseDate": table_info.release_date,
        "saveRev": table_info.table_save_rev,
    });

    let mut extended = base.as_object().unwrap().clone();
    table_info.properties.iter().for_each(|(k, v)| {
        extended.insert(k.clone(), serde_json::Value::String(v.clone()));
    });

    serde_json::Value::Object(extended)
}

pub fn collection_json(collection: &Collection) -> serde_json::Value {
    json!({
        "name": collection.name,
        "items": collection.items,
        "fireEvents": collection.fire_events,
        "stopSingleEvents": collection.stop_single_events,
        "groupElements": collection.group_elements,
    })
}

pub fn collections_json(collections: &[Collection]) -> serde_json::Value {
    let collections: Vec<serde_json::Value> = collections.iter().map(collection_json).collect();
    json!({ "collections": collections })
}
