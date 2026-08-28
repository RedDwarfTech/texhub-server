use std::{
    future::{ready, Ready},
    rc::Rc,
};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::LocalBoxFuture;

use crate::common::request_context::{extract_request_id, with_request_scope, RequestLogContext};

pub struct RequestIdMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestIdMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestIdMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestIdMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct RequestIdMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequestIdMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let log_context = RequestLogContext::new(req.method().as_str(), req.uri().to_string());
        let header_id = req
            .headers()
            .get("x-request-id")
            .and_then(|h| h.to_str().ok())
            .filter(|id| !id.is_empty());
        let query_id = query_request_id(req.query_string());
        let request_id = extract_request_id(
            header_id.or(query_id.as_deref()),
            log_context.clone(),
        );

        Box::pin(async move {
            with_request_scope(request_id, log_context, async move {
                service.call(req).await
            })
            .await
        })
    }
}

/// EventSource cannot send custom headers, so clients may pass x-request-id as a query param.
fn query_request_id(query: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let mut parts = pair.splitn(2, '=');
        let key = parts.next()?;
        let value = parts.next().unwrap_or("").trim();
        (key == "x-request-id" && !value.is_empty()).then(|| value.to_string())
    })
}
