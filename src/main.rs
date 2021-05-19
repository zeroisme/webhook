use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use actix_web::middleware::Logger;
use env_logger::Env;
use chrono::{DateTime, Utc};


#[derive(Debug, Deserialize, Serialize)]
struct Alert {
    status: String,
    labels: HashMap<String, String>,
    annotations: HashMap<String, String>,
    #[serde(rename = "startsAt")]
    starts_at: DateTime<Utc>,
    #[serde(rename = "endsAt")]
    ends_at: Option<DateTime<Utc>>,
    #[serde(rename = "generatorURL")]
    generator_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Notification {
    receiver: String,
    status: String,
    alerts: Vec<Alert>,
    version: String,
    #[serde(rename = "groupLabels")]
    group_labels: HashMap<String, String>,
    #[serde(rename = "commonLabels")]
    common_labels: HashMap<String, String>,
    #[serde(rename = "commonAnnotations")]
    common_annotations: HashMap<String, String>,
    #[serde(rename = "externalURL")]
    external_url: String,
    #[serde(rename = "groupKey")]
    group_key: String
}

trait SendAlert {
    fn send(&self, notification: Notification) -> std::io::Result<()>;
}

struct DingTalkWebhook {
    url: String,
    access_token: String
}

impl DingTalkWebhook {
    fn new(url: String, access_token: String) -> Self {
        DingTalkWebhook {
            url: url,
            access_token: access_token,
        }
    }
}

impl SendAlert for DingTalkWebhook {
    fn send(&self, notification: Notification) -> std::io::Result<()> {
        let url = format!("{}?access_token={}", self.url, self.access_token);
        Ok(())
    }
}

#[post("/alert")]
async fn alert(notification: web::Json<Notification>) -> impl Responder {
    // 接收通知
    let noti: Notification = notification.into_inner();
    // 发送报警
    HttpResponse::Ok().body(serde_json::json!(&noti))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(alert)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
