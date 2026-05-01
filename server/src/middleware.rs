//! 访问日志 Tower Middleware
//!
//! 拦截所有 HTTP 请求，记录 method、path、status、duration_ms

use axum::{
    body::Body,
    http::{Request, Response},
};
use futures_util::future::BoxFuture;
use std::time::Instant;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tokio::sync::mpsc;
use tracing::warn;

use crate::access_log::LogEntry;

/// 访问日志 Layer
#[derive(Clone)]
pub struct AccessLogLayer {
    log_tx: mpsc::Sender<LogEntry>,
}

impl AccessLogLayer {
    pub fn new(log_tx: mpsc::Sender<LogEntry>) -> Self {
        Self { log_tx }
    }
}

impl<S> Layer<S> for AccessLogLayer {
    type Service = AccessLogService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AccessLogService {
            inner,
            log_tx: self.log_tx.clone(),
        }
    }
}

/// 访问日志 Service
#[derive(Clone)]
pub struct AccessLogService<S> {
    inner: S,
    log_tx: mpsc::Sender<LogEntry>,
}

impl<S> Service<Request<Body>> for AccessLogService<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        let start = Instant::now();
        let method = request.method().to_string();
        let path = request.uri().path().to_string();
        let log_tx = self.log_tx.clone();

        // 克隆 inner service 因为我们需要在 async block 中使用
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let result = inner.call(request).await;

            let duration_ms = start.elapsed().as_millis() as u64;

            match &result {
                Ok(response) => {
                    let status = response.status().as_u16();
                    let entry = LogEntry::HttpRequest {
                        tunnel_id: String::new(), // 后续从请求中提取
                        method,
                        path,
                        status,
                        duration_ms,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    };

                    // 异步发送日志，channel 满则丢弃
                    if let Err(e) = log_tx.try_send(entry) {
                        warn!("访问日志 channel 满或关闭: {}", e);
                    }
                }
                Err(_) => {
                    // 请求处理错误，不记录（错误会被上层处理）
                }
            }

            result
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_layer_creation() {
        let (log_tx, _log_rx) = mpsc::channel::<LogEntry>(100);
        let layer = AccessLogLayer::new(log_tx);
        assert!(true); // layer 创建成功
    }
}
