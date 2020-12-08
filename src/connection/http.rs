use crate::connection::SendMessage;
use bytes::Bytes;
use http::Request as HttpRequest;
use http::Response as HttpResponse;
use reqwest::blocking::Client;
use reqwest::blocking::Request;
use reqwest::Error as RequestError;
use std::convert::TryFrom;

impl SendMessage<HttpRequest<Vec<u8>>, Result<HttpResponse<Bytes>, RequestError>> for Client {
    fn send(&self, data: HttpRequest<Vec<u8>>) -> Result<HttpResponse<Bytes>, RequestError> {
        let req: Request = Request::try_from(data)?;
        let response = self.execute(req)?;
        Ok(HttpResponse::builder()
            .body(Bytes::from(response.bytes().unwrap().to_vec()))
            .unwrap())
    }
}
