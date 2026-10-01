use std::time::Duration;
use std::env;
use std::{fs, io};
use thirtyfour::common::capabilities::firefox::LogLevel::Warn;
use thirtyfour::prelude::*;
use tokio::process::Command;
use tokio::time::sleep;
use std::process::Stdio; 

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let current_dir = std::env::current_dir()?;

    match env::current_dir() {
        Ok(path) => println!("Текущая директория выполнения: {}", path.display()),
        Err(e) => eprintln!("Не удалось получить текущую директорию: {}", e),
    }

    let mut driver_process = Command::new("./geckodriver")
        .env("TMPDIR", &current_dir) 
        .spawn()
        .expect("Не удалось запустить geckodriver");
    
    let caps = DesiredCapabilities::firefox();
    let mut driver = None;
    
    for i in 1..=5 {
        sleep(Duration::from_secs(1)).await;
        println!("Попытка подключения №{}...", i);
        
        match WebDriver::new("http://localhost:4444", caps.clone()).await {
            Ok(web_driver) => {
                driver = Some(web_driver);
                break;
            }
            Err(e) => {
                if i == 5 {
                    println!("Не удалось подключиться к geckodriver после 5 попыток.");
                    println!("Финальная ошибка подключения: {:?}", e);
                    driver_process.kill().await?;
                    return Err(e.into());
                }
            }
        }
    }
    

    let driver = driver.unwrap();

    println!("{}",get_json());
    let start_url = get_json();
    driver.goto(start_url).await?;

    let mut input = String::new();

    let _ = io::stdin().read_line(&mut input);

    let current_url = driver.current_url().await?;
    let clean_url = current_url.to_string();
    save_json(&clean_url);

    driver.quit().await?;
    driver_process.kill().await?;
    
    Ok(())
}

fn get_json() -> String{
    let json_file: Result<String, std::io::Error> = fs::read_to_string("data.json");
    let check_json = match json_file{
        Ok(check_json) => check_json,
        Err(_) => panic!("Can't read file")
    };
    let json_data: serde_json::Value = serde_json::from_str(&check_json).expect("Can't parse json");
    let url = json_data[0]["adress"].as_str().expect("Adress is not string");
    url.to_string()

}
fn save_json(saved_url:&str){

    let res: Result<String, std::io::Error> = fs::read_to_string("data.json");
    let s = match res {
        Ok(s) => s,
        Err(_) => panic!("Can't read file")
    };

    let mut json_data: serde_json::Value = serde_json::from_str(&s).expect("Can't parse json");

    json_data[0]["adress"] = serde_json::json!(saved_url);

    std::fs::write("data.json", serde_json::to_string_pretty(&json_data).unwrap())
        .expect("Can't write to file");
}

