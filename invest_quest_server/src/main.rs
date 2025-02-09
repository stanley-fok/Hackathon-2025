use warp::{Filter, reply::Response};
use std::{collections::HashMap, fs::read, fs::read_to_string, sync::{Arc, RwLock, Mutex}, time::{SystemTime, Duration}};
use rand_chacha::ChaCha20Rng;
use rand::SeedableRng;
use warp::http::StatusCode;
use invest_quest_server::{Account, rewards::Reward};
use httpdate::fmt_http_date;
#[tokio::main]
async fn main() {
    let directory = || std::env::current_dir().unwrap().to_str().unwrap().to_owned();
    let accounts_json = read_to_string(directory()+"/accounts.json").unwrap_or(String::new());
    let accounts: Vec<Account> = serde_json::from_str(&accounts_json).unwrap_or(Vec::new());

    //hash map from username to `index` of account
    //this could be encapsulated but in the interests of speed I'll do it this way for the moment
    let user_data: HashMap<String, usize> = accounts.iter().map(|a| (String::from(a.get_username()))).enumerate().map(|(x,y)| (y,x)).collect();
    let user_data: Arc<RwLock<HashMap<String, usize>>> = Arc::new(
        RwLock::new(
            user_data
        )
    );

    //hash map from session token to `index` of account
    let sessions: Arc<RwLock<HashMap<String, (usize, SystemTime)>>> = Arc::new(RwLock::new(HashMap::new()));

    //wrap accounts in an Arc<RwLock<_>> so it can be accessed from threads
    let accounts = Arc::new(RwLock::new(accounts));

    let rng: Arc<Mutex<ChaCha20Rng>> = Arc::new(Mutex::new(ChaCha20Rng::from_os_rng()));

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
        let accounts = accounts.clone();
        let sessions = sessions.clone();
        let rng = rng.clone();
        warp::path("login.html")
            .and(warp::filters::body::form())
            .map(move |form_response: HashMap<String,String>| {
                println!("{form_response:?}");

                let login_result = login(form_response.clone(), user_data.clone(), accounts.clone(), sessions.clone(), rng.clone());
                handle_login(login_result, directory)
            })
    };
    let register = {
        let user_data = user_data.clone();
        let accounts = accounts.clone();
        warp::path("register.html")
            .and(warp::filters::body::form())
            .map(move |form_response: HashMap<String,String>| {
                println!("{form_response:?}");
                let register_result = register(form_response.clone(), user_data.clone(), accounts.clone());
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

//returns session key and expiry time in result
fn login(form_response: HashMap<String, String>, user_data: Arc<RwLock<HashMap<String, usize>>>, accounts: Arc<RwLock<Vec<Account>>>, sessions: Arc<RwLock<HashMap<String, (usize, SystemTime)>>>, rng: Arc<Mutex<ChaCha20Rng>>) -> Result<(String, SystemTime), LoginError> {
    println!("{form_response:?}");
    let user_data = user_data.read().unwrap();
    let accounts = accounts.read().unwrap();
    let username = form_response.get("username").ok_or(LoginError::InvalidUsername)?;
    let password = form_response.get("password").ok_or(LoginError::NoPassword)?;
    let account_index = *user_data.get(username).ok_or(LoginError::InvalidUsername)?;
    let account = &accounts[account_index];
    if account.get_hash().eq((password.clone() + account.get_salt()).as_bytes()) {
        let mut session_key = String::with_capacity(64);
        let mut rng = rng.lock().unwrap();
        let mut sessions = sessions.write().unwrap();
        while session_key.is_empty() || sessions.contains_key(&session_key) {
            session_key.clear();
            //4 64 bit ints to make a 256 bit key
            let mut session_key_ints = [0_u64;4];

            //randomise key
            <[_] as rand::Fill>::fill(&mut session_key_ints, &mut rng);

            for n in session_key_ints {
                session_key.push_str(&format!("{n:0>16x}"));
            }
        }

        let expires = SystemTime::now()+Duration::from_secs(60*60*24);
        sessions.insert(session_key.clone(), (account_index, expires));


        Ok((session_key, expires))
    } else {
        Err(LoginError::IncorrectPassword)
    }
}

fn handle_login(login_result: Result<(String, SystemTime), LoginError>, directory: impl Fn()->String) -> Response {
    let body = read(directory()+"/login.html").unwrap();
    let mut response = Response::new(body.into());
    match login_result {
        Ok((session_key, expires)) => {
            *response.status_mut() = StatusCode::SEE_OTHER;
            let headers = response.headers_mut();
            let cookie = String::from("auth=")+&session_key+"; Expires="+&fmt_http_date(expires);
            let _ = headers.insert("Set-Cookie", cookie.parse().unwrap());
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

fn register(form_response: HashMap<String, String>, user_data: Arc<RwLock<HashMap<String, usize>>>, accounts: Arc<RwLock<Vec<Account>>>) -> Result<(), RegisterError> {
    let mut user_data = user_data.write().unwrap();
    let mut accounts = accounts.write().unwrap();
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
    let _ = user_data.insert(username.to_owned(), accounts.len());
    accounts.push(account);
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