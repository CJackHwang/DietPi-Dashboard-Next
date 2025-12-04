use std::convert::Infallible;

use futures_util::{Stream, StreamExt};
use http_body_util::{Full, StreamBody, combinators::BoxBody};
use hyper::{
    StatusCode,
    body::{Bytes, Frame},
    header::{self, HeaderName, HeaderValue},
    http::response::Builder as ResponseBuilder,
};

pub struct ServerResponse {
    builder: ResponseBuilder,
    body: BoxBody<Bytes, Infallible>,
}
pub type BuiltResponse = hyper::Response<BoxBody<Bytes, Infallible>>;

pub enum RedirectType {
    Permanent,
    SeeOther,
}

impl ServerResponse {
    pub fn new() -> Self {
        Self {
            builder: ResponseBuilder::new(),
            body: BoxBody::default(),
        }
    }

    pub fn status(mut self, status: StatusCode) -> Self {
        self.builder = self.builder.status(status);
        self
    }

    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: TryInto<HeaderName>,
        K::Error: Into<hyper::http::Error>,
        V: TryInto<HeaderValue>,
        V::Error: Into<hyper::http::Error>,
    {
        self.builder = self.builder.header(key, value);
        self
    }

    pub fn redirect(self, typ: RedirectType, path: &str) -> Self {
        let status = match typ {
            RedirectType::Permanent => StatusCode::PERMANENT_REDIRECT,
            RedirectType::SeeOther => StatusCode::SEE_OTHER,
        };

        self.status(status).header(header::LOCATION, path)
    }

    pub fn body<T: Into<Bytes>>(mut self, body: T) -> Self {
        self.body = BoxBody::new(Full::new(body.into()));
        self
    }

    pub fn stream_body<T: Stream<Item = I> + Send + Sync + 'static, I: Into<Bytes>>(
        mut self,
        stream: T,
    ) -> Self {
        let stream = stream.map(|x| Ok(Frame::data(x.into())));
        self.body = BoxBody::new(StreamBody::new(stream));
        self
    }

    pub fn build(self) -> BuiltResponse {
        self.builder.body(self.body).unwrap()
    }
}
