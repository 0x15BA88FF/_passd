mod command;
mod not_found;

use std::convert::Infallible;
use warp::{Filter, Reply};

pub fn routes() -> impl Filter<Extract = impl Reply, Error = Infallible> + Clone {
    command::route().or(not_found::route())
}
