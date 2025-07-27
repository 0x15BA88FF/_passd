use crate::Config;
use jsonrpsee::{
    Extensions,
    types::{ErrorObject, Params},
};
use log::error;
use passd::models::{metadata::Metadata, secrets::Secrets};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{cmp::Ordering, path::PathBuf, sync::Arc};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "op", content = "value")]
pub enum Operator {
    Eq(Value),
    Gt(Value),
    Lt(Value),
    Contains(Value),
    Regex(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldCondition {
    pub field: String,
    #[serde(flatten)]
    pub operator: Operator,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Filter {
    Not(Box<Filter>),
    And(Vec<Filter>),
    Or(Vec<Filter>),
    Condition(FieldCondition),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SortField {
    pub field: String,
    pub direction: SortDirection,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryRequest {
    pub filter: Option<Filter>,
    pub sort: Option<Vec<SortField>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

fn compare_values(a: &Value, b: &Value) -> Option<Ordering> {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => {
            if let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) {
                a.partial_cmp(&b)
            } else {
                None
            }
        }
        (Value::String(a), Value::String(b)) => Some(a.cmp(b)),
        (Value::Bool(a), Value::Bool(b)) => Some(a.cmp(b)),
        _ => None,
    }
}

impl Filter {
    pub fn matches(&self, meta: &Metadata) -> bool {
        match self {
            Filter::Not(filter) => !filter.matches(meta),
            Filter::And(filters) => filters.iter().all(|f| f.matches(meta)),
            Filter::Or(filters) => filters.iter().any(|f| f.matches(meta)),
            Filter::Condition(cond) => {
                let value = meta.get_field(&cond.field);

                match &cond.operator {
                    Operator::Eq(target) => match value {
                        Ok(Some(val)) => val == *target,
                        Ok(None) => false,
                        Err(_) => false,
                    },
                    Operator::Gt(target) => match value {
                        Ok(Some(val)) => {
                            compare_values(&val, target)
                                == Some(Ordering::Greater)
                        }
                        Ok(None) => false,
                        Err(_) => false,
                    },
                    Operator::Lt(target) => match value {
                        Ok(Some(val)) => {
                            compare_values(&val, target) == Some(Ordering::Less)
                        }
                        Ok(None) => false,
                        Err(_) => false,
                    },
                    Operator::Contains(needle) => match value {
                        Ok(Some(val)) => match (&val, needle) {
                            (Value::Array(haystack), Value::Array(needles)) => {
                                needles.iter().any(|n| haystack.contains(n))
                            }
                            (Value::Array(haystack), _) => {
                                haystack.contains(needle)
                            }
                            (Value::String(hay), Value::String(needle)) => {
                                hay.contains(needle)
                            }
                            _ => val == *needle,
                        },
                        Ok(None) => false,
                        Err(_) => false,
                    },
                    Operator::Regex(pattern) => match value {
                        Ok(Some(val)) => val.as_str().map_or(false, |s| {
                            Regex::new(pattern)
                                .map_or(false, |re| re.is_match(s))
                        }),
                        Ok(None) => false,
                        Err(_) => false,
                    },
                }
            }
        }
    }
}

pub fn handler(
    params: Params,
    ctx: &Arc<Config>,
    _ext: &Extensions,
) -> Result<Vec<PathBuf>, ErrorObject<'static>> {
    let req: QueryRequest = params.parse().map_err(|e| {
        error!("Failed to parse query parameters: {}", e);

        ErrorObject::owned(
            jsonrpsee::types::error::INVALID_PARAMS_CODE,
            "Invalid query parameters",
            Some(e.to_string()),
        )
    })?;

    let result = Secrets {
        config: Arc::clone(ctx),
    }
    .find(
        req.filter
            .map(|filter| move |meta: &Metadata| filter.matches(meta)),
        req.sort.map(|sort_fields| {
            move |a: &Metadata, b: &Metadata| {
                let mut ord = Ordering::Equal;

                for field in &sort_fields {
                    let a_val = a
                        .get_field(&field.field)
                        .ok()
                        .flatten()
                        .unwrap_or(Value::Null);
                    let b_val = b
                        .get_field(&field.field)
                        .ok()
                        .flatten()
                        .unwrap_or(Value::Null);

                    if let Some(cmp) = compare_values(&a_val, &b_val) {
                        ord = match field.direction {
                            SortDirection::Asc => cmp,
                            SortDirection::Desc => cmp.reverse(),
                        };

                        if ord != Ordering::Equal {
                            break;
                        }
                    }
                }
                ord
            }
        }),
        req.offset.map(|x| x as usize),
        req.limit.map(|x| x as usize),
    );

    match result {
        Ok(secrets) => Ok(secrets),
        Err(e) => {
            error!("Failed to find: {}", e);

            Err(ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                "Find failed",
                Some(e.to_string()),
            ))
        }
    }
}
