use warp::{Filter, reply::Response};
use std::{collections::HashMap, fs::read, fs::read_to_string, sync::{Arc, RwLock}};
use warp::http::StatusCode;
use invest_quest_server::{Account, rewards::Reward};
#[tokio::main]
async fn main() {
    let directory = || std::env::current_dir().unwrap().to_str().unwrap().to_owned();
    let accounts = read_to_string(directory()+"/accounts.json").unwrap_or(String::new());
    let user_data: Arc<RwLock<HashMap<String, Account>>> = Arc::new(
        RwLock::new(
            serde_json::from_str(
                &accounts
            ).unwrap_or(HashMap::new())
        )
    );
    let filter = warp::get()
        .and(warp::path::full())
        .map(move |path: warp::filters::path::FullPath| {
            println!("{path:?}");
            let mut not_found = false;
            let html = match path.as_str() {
                _ => {
                    read((directory()+&path.as_str()).replace('/',"\\").replace("%20", " ")).unwrap_or_else(|e| {
                        println!("{:?}",e);
                        not_found = true;
                        read(directory()+"/404.html").unwrap()
                    })
                }
            };
            if path.as_str()=="/rewards_data.json" {
                println!("{}", String::from_utf8(html.clone()).unwrap())
            }
            let mut response = Response::new(html.into());
            match path.as_str().split_once('.') {
                Some((_, s @ ("jpg"|"png"))) => {
                    if not_found {
                        response.headers_mut()
                            .insert("Content-Type", str::parse("text/html")
                            .unwrap());
                    } else {
                        response.headers_mut()
                            .insert("Content-Type", str::parse(&("image/".to_owned() + s))
                            .unwrap());
                    }
                }
                Some((_, s)) => {
                    response.headers_mut()
                        .insert("Content-Type", str::parse(&("text/".to_owned() + s))
                        .unwrap());
                }
                None => {}
            }
            response
        });
    let login = {
        let user_data = user_data.clone();
        warp::path("login.html")
            .and(warp::filters::body::form())
            .map(move |form_response: HashMap<String,String>| {
                println!("{form_response:?}");
                let login_result = login(form_response, user_data.clone());
                handle_login(login_result, directory)
            })
    };
    let register = {
        let user_data = user_data.clone();
        warp::path("register.html")
            .and(warp::filters::body::form())
            .map(move |form_response: HashMap<String,String>| {
                println!("{form_response:?}");
                let register_result = register(form_response, user_data.clone());
                handle_registration(register_result, directory)
            })
    };
    let login_or_register = warp::post().and(login.or(register));
    warp::serve(filter.or(login_or_register))
        .run(([127,0,0,1], 7878))
        .await
}

#[derive(Debug)]
enum LoginError {
    InvalidUsername,
    IncorrectPassword,
    NoPassword
}

fn login(form_response: HashMap<String, String>, user_data: Arc<RwLock<HashMap<String, Account>>>) -> Result<(), LoginError> {
    println!("{form_response:?}");
    let user_data = user_data.read().unwrap();
    let username = form_response.get("username").ok_or(LoginError::InvalidUsername)?;
    let password = form_response.get("password").ok_or(LoginError::NoPassword)?;
    let account = user_data.get(username).ok_or(LoginError::InvalidUsername)?;
    if account.get_hash().eq((password.clone() + account.get_salt()).as_bytes()) {
        Ok(())
    } else {
        Err(LoginError::IncorrectPassword)
    }
}

fn handle_login(login_result: Result<(), LoginError>, directory: impl Fn()->String) -> Response {
    let body = read(directory()+"/login.html").unwrap();
    let mut response = Response::new(body.into());
    match login_result {
        Ok(()) => {
            *response.status_mut() = StatusCode::SEE_OTHER;
            let headers = response.headers_mut();
            let _ = headers.insert("Set-Cookie", "logged in".parse().unwrap());
            let _ = headers.insert("Location", "/index.html".parse().unwrap());
            response
        }
        Err(e) => {
            println!("{e:?}");
            response
        }
    }
}

#[derive(Debug)]
enum RegisterError {
    AlreadyExists,
    MissingValue,
    PasswordsDontMatch
}

fn register(form_response: HashMap<String, String>, user_data: Arc<RwLock<HashMap<String, Account>>>) -> Result<(), RegisterError> {
    let mut user_data = user_data.write().unwrap();
    let username = form_response
        .get("username")
        .ok_or(RegisterError::MissingValue)?
        .as_str();
    let email = form_response
        .get("email")
        .ok_or(RegisterError::MissingValue)?
        .as_str();
    let password = form_response
        .get("password")
        .ok_or(RegisterError::MissingValue)?
        .as_str();
    let second_password = form_response
        .get("passwordconfirm")
        .ok_or(RegisterError::MissingValue)?
        .as_str();
    if password!=second_password {
        return Err(RegisterError::PasswordsDontMatch)
    }
    if user_data.contains_key(username) {
        return Err(RegisterError::AlreadyExists);
    }
    let account = Account::new(username, password, email);
    let _ = user_data.insert(username.to_owned(), account.clone());
    println!("{:?}, pass: {}", *user_data, String::from_utf8(account.get_hash().into()).unwrap());
    Ok(())
}

fn handle_registration(register_result: Result<(), RegisterError>, directory: impl Fn()->String) -> Response {
    let body = read(directory()+"/register.html").unwrap();
    let mut response = Response::new(body.into());
    match register_result {
        Ok(()) => {
            *response.status_mut() = StatusCode::SEE_OTHER;
            let headers = response.headers_mut();
            let _ = headers.insert("Location", "/landing.html".parse().unwrap());
            response
        }
        Err(e) => {
            println!("{e:?}");
            response
        }
    }
}