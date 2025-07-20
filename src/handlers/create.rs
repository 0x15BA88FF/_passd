use jsonrpsee::server::Extensions;
use jsonrpsee::types::Params;
use log::info;

pub fn handler(_params: Params, _ctx: &(), _ext: &Extensions) -> &'static str {
    "Hello, World!"
}
