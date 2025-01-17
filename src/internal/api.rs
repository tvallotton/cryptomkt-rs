use std::collections::HashMap;
use std::fmt::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::{header::{HeaderMap, HeaderValue}, Url};
use ring::hmac::{sign, Key, HMAC_SHA384};

use serde::de::DeserializeOwned;
use serde_json;
// use log::Level;

/// X-MKT-APIKEY: La API key como un string
const X_MKT_APIKEY: &'static str = "X-MKT-APIKEY";
/// X-MKT-SIGNATURE: El mensaje firmado generado por el usuario (ver abajo)
const X_MKT_SIGNATURE: &'static str = "X-MKT-SIGNATURE";
/// X-MKT-TIMESTAMP: Un timestamp para tu llamada
const X_MKT_TIMESTAMP: &'static str = "X-MKT-TIMESTAMP";

use crate::internal::errors::{CryptoMktErrorType, CryptoMktResult};
use crate::internal::request::HttpRequest;

///
/// API Interna
///
#[derive(Debug, Clone)]
pub struct Api<R>
where
    R: HttpRequest<Result=CryptoMktResult<String>>
{
    api_key: String,
    secret_key: String,
    domain: String,
    api_version: String,
    req: Box<R>,
}

impl<R> Api<R>
where
    R: HttpRequest<Result=CryptoMktResult<String>>
{
    ///
    /// Crea una instancia de tipo API
    ///
    /// Argumentos
    ///     api_key: Cryptomarket API_KEY
    ///     secret_key: Cryptomarket SECRET_KEY
    ///     http_transport: Interfaz por donde se harían las peticiones Get y Post al servicio
    ///
    pub fn new<'a>(api_key: &'a str, secret_key: &'a str, http_transport: Box<R>) -> Self {
        Api {
            api_key: api_key.to_string(),
            secret_key: secret_key.to_string(),
            domain: "https://api.cryptomkt.com/".to_string(),
            api_version: "v1".to_string(),
            req: http_transport,
        }
    }
    /// Devuelve el dominio
    pub fn domain(&self) -> String {
        self.domain.clone()
    }

    /// Devuelve la version del API
    pub fn api_version(&self) -> String {
        self.api_version.clone()
    }

    ///
    /// Construye la URL
    ///
    /// Argumentos
    ///     endpoint: Endpoint desde donde se va a extraer los datos
    ///     params: Parámetros de la url
    ///
    pub fn build_url<'a>(&self, endpoint: &'a str, params: &HashMap<String, String>) -> Url {
        let mut api_url = Url::parse(&self.domain).unwrap();
        // Adiciona la version de la API
        api_url = api_url
            .join(format!("{}/", &self.api_version).as_str())
            .unwrap();
        // Adiciona el endpoint
        api_url = api_url.join(endpoint).unwrap();

        for (key, value) in params {
            api_url
                .query_pairs_mut()
                .append_pair(key.as_str(), value.as_str());
        }
        api_url
    }

    ///
    ///
    /// Argumentos
    ///     endpoint: Endpoint desde donde se va a extraer los datos
    ///     params: Parámetros de la url
    ///     is_public: indica si el endpoint es public
    ///
    pub async fn get_edge<'a, T>(
        &self,
        endpoint: &'a str,
        params: HashMap<String, String>,
        is_public: bool,
    ) -> CryptoMktResult<T>
    where
        T: DeserializeOwned,
    {
        let api_url = self.build_url(endpoint, &params);
        let headers = self.build_headers(endpoint, &params, is_public, true);
        let result = self.req.get(api_url, headers).await?;
        match serde_json::from_str(&result) {
            Ok(sr) => Ok(sr),
            Err(e) => {
                println!("{:?}", e);
                Err(CryptoMktErrorType::MalformedResource)
            }
        }
    }
    ///
    ///
    /// Argumentos
    ///     endpoint: Endpoint desde donde se va a extraer los datos
    ///     params: Parámetros de la url
    ///     is_public: indica si el endpoint es public
    ///
    pub async fn post_edge<'a, T>(
        &self,
        endpoint: &'a str,
        payload: HashMap<String, String>,
    ) -> CryptoMktResult<T>
    where
        T: DeserializeOwned,
    {
        let api_url = self.build_url(endpoint, &HashMap::new());
        let headers = self.build_headers(endpoint, &payload, false, false);
        let result = self.req.post(api_url, headers, payload).await?;
        match serde_json::from_str(&result) {
            Ok(sr) => Ok(sr),
            Err(e) => {
                println!("{:?}", e);
                Err(CryptoMktErrorType::MalformedResource)
            }
        }
    }

    ///
    /// Crea el formato para el header => X-MKT-SIGNATURE
    ///
    /// Argumentos
    ///     endpoint: Dirección relativa desde donde se van a extraer los datos o donde se enviarán
    ///     payload: Parámetros de la URL
    ///     is_get: Define si el método de encuesta es GET
    ///
    pub fn build_signature_format<'a>(
        &self,
        endpoint: &'a str,
        payload: &HashMap<String, String>,
        is_get: bool,
    ) -> String {
        // body = str(timestamp)+'/v1/orders/create' + '0.3' + 'ethclp' + '10000' + 'buy'
        let mut signature: String = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => n.as_secs().to_string(),
            Err(_) => "".to_string(),
        };
        // Adiciona /api_version/endpoint
        signature += format!("/{}/{}", &self.api_version, &endpoint).as_str();
        // si es POST se adicionan los valores de las llaves
        if !is_get {
            let mut keys = payload.keys().collect::<Vec<_>>();
            // Ordena las llaves alfabéticamente
            keys.sort();
            for k in keys {
                signature += payload.get(k).unwrap();
            }
        }
        signature
    }

    ///
    /// Devuelve firmado el mensaje pasado como parámetro
    ///
    /// Argumentos
    ///     msg: cadena de texto que se requiere firmar
    ///
    pub fn sign_msg<'a>(&self, msg: &'a str) -> String {
        let s_key = Key::new(HMAC_SHA384, self.secret_key.as_bytes());
        let sign = sign(&s_key, msg.as_bytes());

        let mut output = String::new();
        for byte in sign.as_ref() {
            write!(output, "{:02x}", byte).unwrap();
        }

        output
    }
    ///
    /// Conforma los headers para realizar la petición al servidor, en caso de no ser publica
    /// adiciona los headers para la autenticación
    ///
    ///  Argumentos
    ///     endpoint: Endpoint desde donde se va a extraer los datos
    ///     payload: Parámetros de la url
    ///     is_public: indica si el endpoint es public
    ///     is_get: Define si el método de encuesta es GET
    ///
    fn build_headers<'a>(
        &self,
        endpoint: &'a str,
        payload: &HashMap<String, String>,
        is_public: bool,
        is_get: bool,
    ) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if !is_public {
            let msg_to_sign = self.build_signature_format(endpoint, &payload, is_get);
            let timestamp = msg_to_sign.split("/").collect::<Vec<&str>>();
            headers.insert(
                X_MKT_APIKEY,
                HeaderValue::from_str(self.api_key.as_str()).unwrap(),
            );
            headers.insert(
                X_MKT_SIGNATURE,
                HeaderValue::from_str(self.sign_msg(msg_to_sign.as_str()).as_str()).unwrap(),
            );
            headers.insert(
                X_MKT_TIMESTAMP,
                HeaderValue::from_str(timestamp.first().unwrap()).unwrap(),
            );
        }
        headers
    }
}
