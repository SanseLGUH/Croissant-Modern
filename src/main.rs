use reqwest::blocking::Client;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
struct ProfileJson {
    username: String,
    global_name: String,
    locale: String,
    phone: Option<String>,
    email: Option<String>,
}

fn method<R, P>(method: Method, url: &str, token: &str, payload: Option<P>) -> Result<R, String>
    where
        R: DeserializeOwned,
        P: Serialize + Default,
{
    let client = Client::new();
    let mut request_builder = client.request(method.clone(),
        format!("https://discord.com/api/v9{}", &url));
    request_builder = request_builder.header("Authorization", token);
    
    if let Some(payload) = payload {
        request_builder = request_builder.json(&payload);
    }
    
    let response = request_builder.send().map_err(|e| e.to_string())?;
    let response_json: Value = response.json().map_err(|e| e.to_string())?;

    serde_json::from_value::<R>(response_json).map_err(|e| e.to_string())
}

fn profile(token: &str) {
    let response: Result<ProfileJson, String> = method::<ProfileJson, ()>(Method::GET,
        "/users/@me", token, None); 

    match response {
        Ok(resp) => {
          profile_display(&resp);  
        },
        Err(e) => {
            println!("Error with profile: {}", e);
        },
    }
}

use thirtyfour::prelude::*;
use std::process::Command;
use std::path::Path;

async fn login(token: &str) {
    if !Path::new("./chromedriver.exe").exists() {
        println!("You need to install webdriver to use this command");
        return;
    }

    tokio::spawn(
        async move {
            Command::new("./chromedriver")
                .arg("--port=4462")
                .spawn().unwrap();
        }
    );
    
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:4462", caps)
        .await.unwrap();

    driver.goto("https://discord.com/login").await.unwrap();

    let js_code = format!(
        r#"function login(token) {{
            setInterval(() => {{
                document.body.appendChild(document.createElement`iframe`)
                    .contentWindow.localStorage.token = `"{}"`;
            }}, 50);
            setTimeout(() => {{ location.reload(); }}, 2500);
        }}
        login('{}');"#,
        token, token
    );
    driver.execute(js_code, Vec::new()).await.unwrap();
}

fn profile_display(user_data: &ProfileJson) {
    println!("\n==============================");
    println!("|   Профиль пользователя     |");
    println!("==============================");
    println!("|  Имя пользователя:         |");
    println!("|  {: <25} |", user_data.username);
    println!("|----------------------------|");
    println!("|  Глобальное имя:           |");
    println!("|  {: <25} |", user_data.global_name);
    println!("|----------------------------|");
    println!("|  Локаль:                   |");
    println!("|  {: <25} |", user_data.locale);
    println!("|----------------------------|");
    println!("|  Телефон:                  |");
    println!("|  {: <25} |", user_data.phone.as_ref().unwrap_or(&"Нет".to_string()));
    println!("|----------------------------|");
    println!("|  Электронная почта:        |");
    println!("|  {: <25} |", user_data.email.as_ref().unwrap_or(&"Не указано".to_string()));
    println!("==============================");
}

use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!(" croissant-fastesty [ '-profile' or '-login' ] and then token ");
        return;
    }

    match args[1].as_str() {
        "-profile" => profile(&args[2].to_string()),
        "-login" => login(&args[2].to_string()).await,
        _ => println!("Just use '-profile' or '-login'"),
    }
}
