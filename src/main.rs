use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use actix_web::middleware::Logger;
use log;
use env_logger::Env;
use chrono::{DateTime, Utc};
use tera::Tera;
use tera::Context;
use async_trait::async_trait;
use url::Url;

use std::collections::HashMap;
use std::error::Error;

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

#[async_trait]
trait SendAlert {
    async fn send(&self, content: String) -> Result<reqwest::Response,reqwest::Error>;
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

#[derive(Debug, Serialize)]
struct Data {
    msgtype: String,
    markdown: HashMap<String, String>
}

#[async_trait]
impl SendAlert for DingTalkWebhook {
    async fn send(&self, content: String) -> Result<reqwest::Response, reqwest::Error> {
        let url =Url::parse(&format!("{}?access_token={}", self.url, self.access_token)).expect("Parse dingtalk url error: ");

        let client = reqwest::Client::new();
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());

        let mut markdown = HashMap::new();
        markdown.insert("title".to_string(), "alert".to_string());
        markdown.insert("text".to_string(), content);

        let data = Data {
            msgtype: "markdown".to_string(),
            markdown: markdown,
        };

        Ok(client.post(url).headers(headers).json(&data).send().await?)
    }
}

fn render_template(notification: &Notification) -> Result<String, Box<dyn Error>> {
    let mut tera = Tera::new("templates/*")?;
    tera.autoescape_on(vec![".md"]);
    let mut context = Context::new();
    context.insert("notification", notification);
    let content = tera.render("alert.md", &context)?;
    Ok(content)
}

#[derive(Debug,Serialize, Deserialize)]
struct ResponseBody {
    errcode: i32,
    errmsg: String,
}

#[post("/alert")]
async fn alert(notification: web::Json<Notification>) -> impl Responder {
    let url = "https://oapi.dingtalk.com/robot/send".to_string();
    let access_token = match std::env::var("access_token") {
        Ok(token) => token,
        Err(e) => {
            log::error!("Get dingtalk access_token error: {}", e);
            ::std::process::exit(1);
        }
    };
    let dingtalk_webhook = DingTalkWebhook::new(url, access_token);
    // 接收通知
    let mut notification = notification.into_inner();
    if let Ok(alertmanager_url) = std::env::var("alertmanager_url") {
        notification.external_url = alertmanager_url;
    }
        
    match render_template(&notification){
        Ok(content) => { 
            match dingtalk_webhook.send(content).await {
                Ok(resp) => {
                    let resp = resp.json::<ResponseBody>().await;
                    match resp {
                        Err(e) => {
                            log::error!("send to dingtalk error: {}", e);
                            return HttpResponse::BadRequest().body(format!("send to dingtalk error"));
                        },
                        Ok(resp) => {
                            if resp.errcode == 0 {
                                return HttpResponse::Ok().body("ok");
                            }
                            log::error!("send to dingtalk error, errcode: {}, errmsg: {}", resp.errcode, resp.errmsg);
                            return HttpResponse::BadRequest().body(format!("errcode: {}, errmsg: {}", resp.errcode, resp.errmsg));
                        },
                    }
                }
                Err(e) => {
                    log::error!("send to dingtalk error: {}", e);
                    HttpResponse::BadRequest().body(format!("send to dingtalk error"))
                }
            }
        }
        Err(e) =>  { 
            log::error!("render template error: {}", e);
            HttpResponse::InternalServerError().body(format!("render template error"))
        }
    }
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
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
