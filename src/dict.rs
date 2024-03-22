use serde::Serialize;

#[derive(Serialize)]
pub struct Dict(pub Vec<Entry>);

#[derive(Debug, Serialize)]
pub enum Kind {
    Word,
    Phrase
}

#[derive(Debug, Serialize)]
pub struct Entry {
    pub name: String,
    pub trans: Vec<String>,
    pub kind: Kind
}
