use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct StuMapHeader {
    #[serde(rename = "506FA8D8")]
    pub name: String,
}
