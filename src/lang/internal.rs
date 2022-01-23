

pub const INTERNAL_NAMESPACE: &str = "_internal_";

#[derive(Debug, Clone, PartialEq)]
pub enum LeafType {
  String
, Int
, Float
, Bool
, Id
}

const STRING: &str = "String";
const INT: &str = "Int";
const FLOAT: &str = "Float";
const BOOL: &str = "Bool";
const ID: &str = "Id";

impl LeafType {
  pub fn all() -> Vec<LeafType> {
    vec!(LeafType::String, LeafType::Int, LeafType::Float, LeafType::Bool)
  }

  fn as_str(&self) -> &str {
    match &self {
      LeafType::String => STRING
    , LeafType::Int => INT
    , LeafType::Float => FLOAT
    , LeafType::Bool => BOOL
    , LeafType::Id => ID
    }
  }

  pub fn name(&self) -> String {
    self.as_str().to_string()
  }

  pub fn is_leaf_type(type_name: &str) -> bool {
    LeafType::all().iter().map(|l| l.as_str()).collect::<Vec<&str>>().contains(&type_name)
  }
}
