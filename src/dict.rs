use serde::Serialize;

#[derive(Serialize)]
pub struct Dict(pub Vec<DictEntry>);

#[derive(Serialize, Debug)]
pub struct DictEntry {
    pub name: String,
    pub trans: Vec<String>,
}
