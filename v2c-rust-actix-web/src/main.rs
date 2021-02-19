use serde_yaml::{to_value as v, Value as YamlValue};

use actix_web::{client::Client, get, web, App, HttpServer, Responder};

use serde::Deserialize;

#[derive(Deserialize)]
struct UpstreamInfo {
    from: String,
}

#[get("/v2rayn_to_clash")]
async fn index(info: web::Query<UpstreamInfo>) -> impl Responder {
    println!("GET /v2rayn_to_clash?from=**secret**");
    v2rayn_to_clash(&info.from).await.unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(index))
        .bind("127.0.0.1:8423")?
        .run()
        .await
}

async fn v2rayn_to_clash(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    // let body = reqwest::get(Url::parse(&url)?).await?.text().await?;

    let client = Client::default();

    let body = client.get(url).send().await?.body().await?.to_vec();

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
