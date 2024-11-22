use std::fmt;
use std::fmt::{Display, Formatter};
use std::time::Duration;

use hyper::client::HttpConnector;
use hyper::header::{AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use hyper::http::StatusCode;
use hyper::{Body, Client, Method, Uri};
use hyper_rustls::HttpsConnector;
use hyper_timeout::TimeoutConnector;
use std::time::SystemTime;
use thiserror::Error;
use tokio_rustls::rustls;
use tokio_rustls::rustls::client::{ServerCertVerified, ServerCertVerifier};
use tokio_rustls::rustls::{Certificate, ClientConfig, PrivateKey, RootCertStore, ServerName};

use crate::UFMCert;

struct NoCertificateVerification;

impl ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
}

const REST_TIME_OUT: Duration = Duration::from_secs(10);

#[derive(Error, Debug)]
pub enum RestError {
    #[error("{0}")]
    Unknown(String),
    #[error("'{0}' not found")]
    NotFound(String),
    #[error("failed to auth '{0}'")]
    AuthFailure(String),
    #[error("invalid configuration '{0}'")]
    InvalidConfig(String),
}

impl From<hyper::Error> for RestError {
    fn from(value: hyper::Error) -> Self {
        if value.is_user() {
            return RestError::AuthFailure(value.message().to_string());
        }

        RestError::Unknown(value.message().to_string())
    }
}

#[derive(Clone, Debug)]
pub enum RestScheme {
    Http,
    Https,
}

impl From<String> for RestScheme {
    fn from(value: String) -> Self {
        match value.to_uppercase().as_str() {
            "HTTP" => RestScheme::Http,
            "HTTPS" => RestScheme::Https,
            _ => RestScheme::Http,
        }
    }
}

impl Display for RestScheme {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RestScheme::Http => write!(f, "http"),
            RestScheme::Https => write!(f, "https"),
        }
    }
}

pub struct RestClientConfig {
    pub address: String,
    pub port: Option<u16>,
    pub scheme: RestScheme,
    pub auth_info: String,
    pub base_path: String,
}

pub struct RestClient {
    base_url: String,
    auth_info: String,
    scheme: RestScheme,
    http_client: hyper::Client<TimeoutConnector<HttpConnector>>,
    https_client: hyper::Client<TimeoutConnector<HttpsConnector<HttpConnector>>>,
}

impl RestClient {
    pub fn new(conf: &RestClientConfig) -> Result<RestClient, RestError> {
        let mut auth_info = conf.auth_info.clone().trim().to_string();
        let mut auto_cert: Option<UFMCert> = None;

        if auth_info.chars().filter(|c| *c == '\n').count() == 2 {
            let mut v = auth_info.split('\n');
            auto_cert = Some(UFMCert {
                ca_crt: v.next().unwrap_or("").to_string(),
                tls_key: v.next().unwrap_or("").to_string(),
                tls_crt: v.next().unwrap_or("").to_string(),
            });
            auth_info = "".to_string();
        } else {
            auth_info = format!("Basic {}", conf.auth_info.clone().trim());
        }

        let base_url = match &conf.port {
            None => format!(
                "{}://{}/{}",
                conf.scheme,
                conf.address,
                conf.base_path.trim_matches('/')
            ),
            Some(p) => format!(
                "{}://{}:{}/{}",
                conf.scheme,
                conf.address,
                p,
                conf.base_path.trim_matches('/')
            ),
        };

        let _ = base_url
            .parse::<Uri>()
            .map_err(|_| RestError::InvalidConfig("invalid rest address".to_string()))?;

        let mut http_connector = TimeoutConnector::new(HttpConnector::new());
        http_connector.set_connect_timeout(Some(REST_TIME_OUT));
        http_connector.set_read_timeout(Some(REST_TIME_OUT));
        http_connector.set_write_timeout(Some(REST_TIME_OUT));

        let config = if auto_cert.is_some() {
            // Get CA root
            let mut roots = RootCertStore::empty();
            let fd = match std::fs::File::open(auto_cert.clone().unwrap().ca_crt) {
                Ok(fd) => fd,
                Err(_) => {
                    return Err(RestError::NotFound(format!(
                        "Root CA file not found at '{}'",
                        auto_cert.clone().unwrap().ca_crt
                    )));
                }
            };
            let mut buf = std::io::BufReader::new(&fd);
            match rustls_pemfile::certs(&mut buf) {
                Ok(certs) => roots.add_parsable_certificates(&certs),
                Err(_) => {
                    return Err(RestError::NotFound(format!(
                        "Root CA file not found at '{}'",
                        auto_cert.clone().unwrap().tls_crt
                    )));
                }
            };

            // Get client certificate
            let certs = {
                let fd = match std::fs::File::open(auto_cert.clone().unwrap().tls_crt) {
                    Ok(fd) => fd,
                    Err(_) => {
                        return Err(RestError::NotFound(format!(
                            "Client Cert file not found at '{}'",
                            auto_cert.clone().unwrap().tls_crt
                        )));
                    }
                };
                let mut buf = std::io::BufReader::new(&fd);
                match rustls_pemfile::certs(&mut buf) {
                    Ok(certs) => certs.into_iter().map(Certificate).collect::<Vec<_>>(),
                    Err(_) => {
                        return Err(RestError::NotFound(format!(
                            "Client Cert file not found at '{}'",
                            auto_cert.clone().unwrap().tls_crt
                        )));
                    }
                }
            };

            // Get client private key
            let key = {
                let fd = match std::fs::File::open(auto_cert.clone().unwrap().tls_key) {
                    Ok(fd) => fd,
                    Err(_) => {
                        return Err(RestError::NotFound(format!(
                            "Client Private Key file not found at '{}'",
                            auto_cert.clone().unwrap().tls_key
                        )));
                    }
                };
                let mut buf = std::io::BufReader::new(&fd);
                use rustls_pemfile::Item;
                match rustls_pemfile::read_one(&mut buf) {
                    Ok(Some(item)) => match item {
                        Item::RSAKey(rsa_key) => Some(PrivateKey(rsa_key)),
                        Item::PKCS8Key(pkcs8_key) => Some(PrivateKey(pkcs8_key)),
                        Item::ECKey(ec_key) => Some(PrivateKey(ec_key)),
                        Item::X509Certificate(_) => {
                            return Err(RestError::NotFound(format!(
                                "Expected Client Private Key but certificate is found '{}'",
                                auto_cert.clone().unwrap().tls_key
                            )));
                        }
                        Item::Crl(_) => {
                            return Err(RestError::NotFound(format!("Expected Client Private Key but certificate revocation list is found '{}'", auto_cert.clone().unwrap().tls_key)));
                        }
                        _ => {
                            return Err(RestError::NotFound(format!(
                                "Client Private Key is corrupted '{}'",
                                auto_cert.clone().unwrap().tls_key
                            )));
                        }
                    },
                    _ => {
                        return Err(RestError::NotFound(format!(
                            "Client Private Key file not found at '{}'",
                            auto_cert.clone().unwrap().tls_key
                        )));
                    }
                }
            };

            let build_no_client_auth_config = || {
                ClientConfig::builder()
                    .with_safe_defaults()
                    .with_root_certificates(roots.clone())
                    .with_no_client_auth()
            };

            if !certs.is_empty() && key.is_some() {
                if let Ok(config) = ClientConfig::builder()
                    .with_safe_defaults()
                    .with_root_certificates(roots.clone())
                    .with_client_auth_cert(certs, key.unwrap())
                {
                    // Use TLS flow with client authentication
                    config
                } else {
                    // Client creation failure
                    build_no_client_auth_config()
                }
            } else {
                // Unable to use client cert/key pair
                build_no_client_auth_config()
            }
        } else {
            // No TLS flow
            ClientConfig::builder()
                .with_safe_defaults()
                .with_custom_certificate_verifier(std::sync::Arc::new(NoCertificateVerification))
                .with_no_client_auth()
        };

        let mut https_connector = TimeoutConnector::new(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_tls_config(config)
                .https_or_http()
                .enable_http1()
                .enable_http2()
                .build(),
        );
        https_connector.set_connect_timeout(Some(REST_TIME_OUT));
        https_connector.set_read_timeout(Some(REST_TIME_OUT));
        https_connector.set_write_timeout(Some(REST_TIME_OUT));

        Ok(Self {
            base_url,
            auth_info,
            scheme: conf.scheme.clone(),
            // TODO(k82cn): Add timout for the clients.
            http_client: Client::builder().build::<_, hyper::Body>(http_connector),
            https_client: Client::builder().build::<_, hyper::Body>(https_connector),
        })
    }

    pub async fn get<'a, T: serde::de::DeserializeOwned>(
        &'a self,
        path: &'a str,
    ) -> Result<T, RestError> {
        let resp = self.execute_request(Method::GET, path, None).await?;
        if resp.eq("{}") {
            return Err(RestError::NotFound("not found".to_string()));
        }

        let data = serde_json::from_str(&resp)
            .map_err(|_| RestError::InvalidConfig("invalid response".to_string()))?;

        Ok(data)
    }

    pub async fn list<'a, T: serde::de::DeserializeOwned>(
        &'a self,
        path: &'a str,
    ) -> Result<T, RestError> {
        let resp = self.execute_request(Method::GET, path, None).await?;
        let data = serde_json::from_str(&resp)
            .map_err(|_| RestError::InvalidConfig("invalid response".to_string()))?;

        Ok(data)
    }

    pub async fn post(&self, path: &str, data: String) -> Result<(), RestError> {
        self.execute_request(Method::POST, path, Some(data)).await?;

        Ok(())
    }

    pub async fn put(&self, path: &str, data: String) -> Result<(), RestError> {
        self.execute_request(Method::PUT, path, Some(data)).await?;

        Ok(())
    }

    pub async fn delete(&self, path: &str) -> Result<(), RestError> {
        self.execute_request(Method::DELETE, path, None).await?;

        Ok(())
    }

    async fn execute_request(
        &self,
        method: Method,
        path: &str,
        data: Option<String>,
    ) -> Result<String, RestError> {
        let url = format!("{}/{}", self.base_url, path.trim_matches('/'));
        let uri = url
            .parse::<Uri>()
            .map_err(|_| RestError::InvalidConfig("invalid path".to_string()))?;

        let body = data.unwrap_or_default();
        log::debug!("Method: {method}, URL: {url}, Body: {body}");

        let req = hyper::Request::builder()
            .method(method)
            .uri(uri)
            .header(USER_AGENT, env!("CARGO_PKG_NAME"))
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, self.auth_info.to_string())
            .body(Body::from(body))
            .map_err(|_| RestError::InvalidConfig("invalid rest request".to_string()))?;

        let body = match &self.scheme {
            RestScheme::Http => self
                .http_client
                .request(req)
                .await
                .map_err(|e| RestError::Unknown(format!("rest request failure: {:?}", e)))?,
            RestScheme::Https => self
                .https_client
                .request(req)
                .await
                .map_err(|e| RestError::Unknown(format!("rest request failure: {:?}", e)))?,
        };

        let status = body.status();
        let chunk = hyper::body::to_bytes(body.into_body()).await?;
        let data = String::from_utf8(chunk.to_vec()).unwrap();

        match status {
            StatusCode::OK => Ok(data),
            _ => Err(RestError::Unknown(data)),
        }
    }
}
