use passd::types::command_response;
use std::convert::Infallible;
use warp::{Filter, Reply};

pub fn route() -> impl Filter<Extract = impl Reply, Error = Infallible> + Clone {
    warp::any().map(|| {
        warp::reply::with_status(
            warp::reply::json(&command_response::Response {
                status: 404,
                success: false,
                message: "Invalid route".to_string(),
                data: None,
                error: Some(command_response::Error {
                    r#type: Some(command_response::ErrorType::NotFound),
                    message: "The requested endpoint does not exist".to_string(),
                }),
            }),
            warp::http::StatusCode::NOT_FOUND,
        )
    })
}
