use std::sync::OnceLock;

use reqwest::header::HeaderValue;

use crate::common::request_context;

pub fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .connect_timeout(std::time::Duration::from_secs(10))
            .pool_idle_timeout(std::time::Duration::from_secs(30))
            .pool_max_idle_per_host(20)
            .build()
            .expect("Failed to build reqwest client")
    })
}

pub fn request_id_header_value() -> HeaderValue {
    HeaderValue::from_str(&request_context::outbound_request_id(None))
        .unwrap_or_else(|_| HeaderValue::from_static("unknown"))
}

pub trait OutboundRequestExt {
    fn with_request_id(self) -> Self;
}

impl OutboundRequestExt for reqwest::RequestBuilder {
    fn with_request_id(self) -> Self {
        let hint_uri = self
            .try_clone()
            .and_then(|builder| builder.build().ok())
            .map(|request| request.url().to_string());
        self.header(
            "x-request-id",
            HeaderValue::from_str(&request_context::outbound_request_id(hint_uri.as_deref()))
                .unwrap_or_else(|_| HeaderValue::from_static("unknown")),
        )
    }
}

impl OutboundRequestExt for reqwest::blocking::RequestBuilder {
    fn with_request_id(self) -> Self {
        let hint_uri = self
            .try_clone()
            .and_then(|builder| builder.build().ok())
            .map(|request| request.url().to_string());
        self.header(
            "x-request-id",
            HeaderValue::from_str(&request_context::outbound_request_id(hint_uri.as_deref()))
                .unwrap_or_else(|_| HeaderValue::from_static("unknown")),
        )
    }
}

