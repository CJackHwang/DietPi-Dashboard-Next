use std::{
    collections::HashMap,
    net::IpAddr,
    ops::{Deref, DerefMut},
};

use config::frontend::FrontendConfig;
use http_body_util::BodyExt;
use hyper::{
    StatusCode,
    body::{Bytes, Incoming},
    header,
    http::request::Parts as RequestParts,
};
use proto::{
    backend::ResponseBackendMessage,
    frontend::{ActionFrontendMessage, RequestFrontendMessage},
};

use crate::backend::BackendHandle;

use super::{
    FrontendContext,
    auth::SharedLoginMap,
    response::{RedirectType, ServerResponse},
};

pub type HyperRequest = hyper::Request<Incoming>;

fn get_cookies(parts: &RequestParts) -> HashMap<String, String> {
    let cookie_header = parts
        .headers
        .get(header::COOKIE)
        .and_then(|x| x.to_str().ok())
        .unwrap_or_default();

    cookie_header
        .split("; ")
        .filter_map(|x| x.split_once('='))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

pub struct BackendData {
    pub backend_list: Vec<(IpAddr, String)>,
    pub current_backend: CurrentBackendData,
}

pub struct CurrentBackendData {
    pub addr: IpAddr,
    pub handle: BackendHandle,
    pub update: Option<String>,
}

pub struct ServerRequest {
    parts: RequestParts,
    body: Option<Incoming>,
    cookies: HashMap<String, String>,
    context: FrontendContext,
}

impl ServerRequest {
    pub fn new(req: HyperRequest, context: FrontendContext) -> Self {
        let (parts, body) = req.into_parts();

        let cookies = get_cookies(&parts);

        Self {
            parts,
            body: Some(body),
            cookies,
            context,
        }
    }

    pub fn path_segments(&self) -> impl Iterator<Item = &str> {
        self.uri.path().split('/').filter(|x| !x.is_empty())
    }

    pub fn config(&self) -> &FrontendConfig {
        &self.context.config
    }

    pub fn extract_backends(&self) -> Result<BackendData, ServerResponse> {
        let backends = self.context.backends.lock().unwrap();
        let backend_list: Vec<_> = backends
            .iter()
            .map(|(addr, info)| (*addr, info.nickname.clone()))
            .collect();

        if backend_list.is_empty() {
            return Err(ServerResponse::new()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .body("no connected backends"));
        }

        let current_backend = {
            let cookie_ip = self
                .cookies
                .get("backend")
                .and_then(|x| x.parse::<IpAddr>().ok());

            let (&addr, backend_info) = cookie_ip
                .and_then(|x| backends.get_key_value(&x))
                .or_else(|| backends.get_key_value(&backend_list[0].0))
                .unwrap();

            CurrentBackendData {
                addr,
                handle: backend_info.handle.clone(),
                update: backend_info.update.clone(),
            }
        };

        Ok(BackendData {
            backend_list,
            current_backend,
        })
    }

    pub async fn send_backend_req(
        &self,
        req: RequestFrontendMessage,
    ) -> Result<ResponseBackendMessage, ServerResponse> {
        let backend_handle = self.extract_backends()?.current_backend.handle;

        backend_handle.send_req(req).await.map_err(|err| {
            ServerResponse::new()
                .status(StatusCode::BAD_GATEWAY)
                .body(format!("backend request failed: {err}"))
        })
    }

    pub async fn send_backend_action(
        &self,
        msg: ActionFrontendMessage,
    ) -> Result<(), ServerResponse> {
        let backend_handle = self.extract_backends()?.current_backend.handle;

        backend_handle.send_action(msg).await.map_err(|err| {
            ServerResponse::new()
                .status(StatusCode::BAD_GATEWAY)
                .body(format!("backend action failed: {err}"))
        })
    }

    pub fn extract_query<Qu: serde::de::DeserializeOwned>(&self) -> Result<Qu, ServerResponse> {
        let query = self.uri.query().unwrap_or_default();

        serde_urlencoded::from_str(query).map_err(|err| {
            ServerResponse::new()
                .status(StatusCode::BAD_REQUEST)
                .body(format!("invalid query params: {err}"))
        })
    }

    pub async fn extract_form<T: serde::de::DeserializeOwned>(
        &mut self,
    ) -> Result<T, ServerResponse> {
        let body = self.extract_body().await?;

        let body = body.collect().await.map_err(|_| {
            ServerResponse::new()
                .status(StatusCode::BAD_REQUEST)
                .body("needs body")
        })?;

        serde_urlencoded::from_bytes(&body.to_bytes()).map_err(|_| {
            ServerResponse::new()
                .status(StatusCode::BAD_REQUEST)
                .body("invalid form body")
        })
    }

    pub async fn extract_body(&mut self) -> Result<Incoming, ServerResponse> {
        let Some(body) = self.body.take() else {
            return Err(ServerResponse::new()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("form already extracted"));
        };

        Ok(body)
    }

    pub fn is_fixi(&self) -> bool {
        self.headers.contains_key("nm-request")
    }

    pub fn check_login(&self) -> Result<(), ServerResponse> {
        if self.config().enable_login {
            let err_resp = if self.is_fixi() {
                Err(ServerResponse::new()
                    .body(r#"<meta http-equiv="refresh" content="0; url=/login" />"#))
            } else {
                Err(ServerResponse::new().redirect(RedirectType::SeeOther, "/login"))
            };

            let Some(token) = self.cookies.get("token") else {
                return err_resp;
            };

            if !self.context.logins.get().contains_token(token) {
                return err_resp;
            }
        }

        Ok(())
    }

    pub fn extract_logins(&self) -> SharedLoginMap {
        self.context.logins.clone()
    }
}

impl Deref for ServerRequest {
    type Target = RequestParts;

    fn deref(&self) -> &Self::Target {
        &self.parts
    }
}

impl DerefMut for ServerRequest {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parts
    }
}
