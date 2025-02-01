use passd::commands;
use passd::types::{command_request, command_response};
use warp::{
    body::BodyDeserializeError,
    Filter, Rejection, Reply
};

pub fn route() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::post()
        .and(warp::path::end())
        .and(warp::body::json())
        .and_then(handle_request)
        .recover(handle_rejection)
}

async fn handle_request(request: command_request::Request) -> Result<impl Reply, Rejection> {
    if let Some(response) = commands::handler(&request) {
        Ok(warp::reply::with_status(
            warp::reply::json(&response),
            warp::http::StatusCode::from_u16(response.status).unwrap_or(warp::http::StatusCode::OK),
        ))
    } else {
        Ok(warp::reply::with_status(
            warp::reply::json(&command_response::Response {
                data: None,
                status: 400,
                success: false,
                message: "Unknown command".to_string(),
                error: Some(command_response::Error {
                    r#type: Some(command_response::ErrorType::InvalidRequest),
                    message: format!("Command '{}' is not recognized", request.command),
                }),
            }),
            warp::http::StatusCode::BAD_REQUEST
        ))
    }
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&command_response::Response {
                data: None,
                status: warp::http::StatusCode::METHOD_NOT_ALLOWED.into(),
                success: false,
                message: "Method Not Allowed".to_string(),
                error: Some(command_response::Error {
                    r#type: Some(command_response::ErrorType::InvalidRequest),
                    message: "The request method is not allowed".to_string()
                })
            }),
            warp::http::StatusCode::METHOD_NOT_ALLOWED
        ))
    }

    if let Some(_) = err.find::<BodyDeserializeError>() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&command_response::Response {
                data: None,
                status: warp::http::StatusCode::BAD_REQUEST.into(),
                success: false,
                message: "Failed to deserialize body".to_string(),
                error: Some(command_response::Error {
                    r#type: Some(command_response::ErrorType::InvalidRequest),
                    message: "The request body is invalid".to_string()
                })
            }),
            warp::http::StatusCode::BAD_REQUEST
        ))
    }

    Ok(warp::reply::with_status(
        warp::reply::json(&command_response::Response {
            data: None,
            status: warp::http::StatusCode::INTERNAL_SERVER_ERROR.into(),
            success: false,
            message: "Internal Server Error".to_string(),
            error: Some(command_response::Error {
                r#type: Some(command_response::ErrorType::InvalidRequest),
                message: "Something went wrong, try again later".to_string()
            })
        }),
        warp::http::StatusCode::INTERNAL_SERVER_ERROR
    ))
}
