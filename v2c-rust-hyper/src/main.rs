use std::convert::Infallible;

use reqwest::Url;
use serde_yaml::{to_value as v, Value as YamlValue};

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};

#[tokio::main]
pub async fn main() {
    // println!("{}", v2rayn_to_clash("https://yukari.kelu.org/v2/04669d2c04ff47d985673a4e5b0c7ea1").await.unwrap());

    let _ = start_server().await;

    ()
}

async fn start_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([127, 0, 0, 1], 8423).into();

    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handler)) });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    Ok(())
}

async fn handler(
    req: Request<Body>,
) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
    let (parts, _) = req.into_parts();

    let mut resp = Response::new(Body::empty());

    match (parts.method, parts.uri.path()) {
        (hyper::Method::GET, "/v2rayn_to_clash") => {
            if let Some(qs) = parts.uri.query() {
                let mut upstream = String::new();

                for (key, value) in url::form_urlencoded::parse(qs.as_bytes()) {
                    match key.into_owned().as_str() {
                        "from" => {
                            upstream = value.into_owned();
                        }
                        a => {
                            println!("Unknown key-value: {} {}", a, value);
                        }
                    }
                }
                *resp.body_mut() = Body::from(v2rayn_to_clash(upstream.as_str()).await?);
            } else {
                *resp.status_mut() = StatusCode::BAD_REQUEST
            }
        }
        (_, "/v2rayn_to_clash") => {
            *resp.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        }
        _ => {
            *resp.status_mut() = StatusCode::NOT_FOUND;
        }
    }

    Ok(resp)
}

async fn v2rayn_to_clash(url: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let body = reqwest::get(Url::parse(&url)?).await?.text().await?;

    let decoded = base64::decode(body)?;
    let donces = std::str::from_utf8(&decoded)?;

    let mut clash: YamlValue = serde_yaml::from_str("proxies: []")?;
    let clash_proxies = clash
        .get_mut("proxies")
        .ok_or("Wrong")?
        .as_sequence_mut()
        .ok_or("Wrong")?;

    for donce in donces.split_whitespace() {
        if let Some(donce) = donce.strip_prefix("vmess://") {
            let dtwice = base64::decode(donce)?;
            let json = std::str::from_utf8(&dtwice)?;
            let v2rayn: serde_json::Value = serde_json::from_str(json)?;
            let mut cproxy = serde_yaml::Mapping::new();
            cproxy.insert(
                v("name")?,
                v(v2rayn.get("ps").ok_or("Missing remarks(`ps`)")?.to_owned())?,
            );
            cproxy.insert(v("type")?, v("vmess")?);
            cproxy.insert(
                v("server")?,
                v(v2rayn.get("add").ok_or("Missing server(`add`)")?.to_owned())?,
            );
            cproxy.insert(
                v("port")?,
                v(v2rayn.get("port").ok_or("Missing port")?.to_owned())?,
            );
            cproxy.insert(
                v("uuid")?,
                v(v2rayn.get("id").ok_or("Missing uuid")?.to_owned())?,
            );
            cproxy.insert(
                v("alterId")?,
                v(v2rayn.get("aid").ok_or("Missing alter id")?.to_owned())?,
            );
            cproxy.insert(v("cipher")?, v("auto")?);
            cproxy.insert(v("skip-cert-verify")?, v(true)?);
            cproxy.insert(
                v("network")?,
                v(v2rayn.get("net").ok_or("Missing network")?.to_owned())?,
            );
            cproxy.insert(
                v("ws-path")?,
                v(v2rayn.get("path").ok_or("Missing path")?.to_owned())?,
            );
            cproxy.insert(
                v("ws-headers")?,
                serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
            );
            cproxy
                .get_mut(&v("ws-headers")?)
                .ok_or("Wrong")?
                .as_mapping_mut()
                .ok_or("Wrong")?
                .insert(
                    v("host")?,
                    v(v2rayn.get("host").ok_or("Missing fake domain")?.to_owned())?,
                );
            cproxy.insert(
                v("tls")?,
                v(v2rayn.get("tls").ok_or("Missing tls")?.to_owned() == "tls")?,
            );
            clash_proxies.push(YamlValue::Mapping(cproxy));
        }
    }

    Ok(serde_yaml::to_string(&clash)?)
}
