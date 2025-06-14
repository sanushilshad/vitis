use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
pub enum ProcessType {
    Immediate,
    Deferred,
}
