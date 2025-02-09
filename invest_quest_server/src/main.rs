use warp::{Filter, reply::Response};
use std::{collections::HashMap, fs::{read, read_to_string, File}, sync::{Arc, RwLock, Mutex}, time::{SystemTime, Duration}, io::{Seek, SeekFrom, Write}};
use rand_chacha::ChaCha20Rng;
use rand::SeedableRng;
use warp::http::StatusCode;
use invest_quest_server::{Account, rewards::Reward, AccountMessage, SavingsAccount, SavingsVehicle, CurrentAccount};
use httpdate::{fmt_http_date, parse_http_date};
#[tokio::main]
async fn main() {
    let directory = || std::env::current_dir().unwrap().to_str().unwrap().to_owned();
    let accounts_json = read_to_string(directory()+"/accounts.json").unwrap_or(String::new());
    let accounts: Vec<Account> = serde_json::from_str(&accounts_json)
        .unwrap_or(Vec::new());

    //hash map from username to `index` of account
    //this could be encapsulated but in the interests of speed I'll do it this way for the moment
    let user_data: HashMap<String, usize> =
        accounts
            .iter()
            .map(|a| (String::from(a.get_username())))
            .enumerate()
            .map(|(x,y)| (y,x))
            .collect();
    let user_data: Arc<RwLock<HashMap<String, usize>>> =
        Arc::new(
            RwLock::new(
                user_data
            )
        );

    //hash map from session token to `index` of account
    let sessions: Arc<RwLock<HashMap<String, (usize, SystemTime)>>> = Arc::new(RwLock::new(HashMap::new()));

    //wrap accounts in an Arc<RwLock<_>> so it can be accessed from threads
    let accounts = Arc::new(RwLock::new(accounts));

    let rng: Arc<Mutex<ChaCha20Rng>> = Arc::new(Mutex::new(ChaCha20Rng::from_os_rng()));

    let authorised = {
        let sessions = sessions.clone();
        let accounts = accounts.clone();
        warp::cookie::<String>("auth")
            .and(warp::path::full())
            .and(warp::query::query())
            .map(move |cookie: String, path: warp::filters::path::FullPath, query: HashMap<String, usize>| {
                let sessions = if path.as_str()=="/logout" {
                    //remove session
                    sessions
                        .write()
                        .unwrap()
                        .remove(&cookie);

                    //serve landing page
                    let mut response = Response::new(read(directory()+"/landing.html").unwrap().into());
                    let headers = response.headers_mut();
                    headers.insert("Content-Type", "text/html".parse().unwrap());

                    //issue redirect
                    headers.insert("Location", "/landing.html".parse().unwrap());
                    *response.status_mut() = StatusCode::SEE_OTHER;

                    //early return
                    return(response)
                } else {
                    sessions.read().unwrap()
                };

                let session = sessions.get(&cookie);


                //this if-else is a total mess, but I don't have time to work out how to do it properly
                //this is if (session key is in session list) and (session key hasn't expired)
                if session.map(|(_, exp)| &SystemTime::now()<=exp).unwrap_or(false) {
                    println!("{path:?}");
                    let mut not_found = false;
                    let html = match path.as_str() {
                        "/account" => {
                            let accounts = accounts.read().unwrap();
                            let account = &accounts[sessions.get(&cookie).unwrap().0];
                            let account_message = AccountMessage {
                                name: account.get_username(),
                                balance: account.get_balance()
                            };
                            serde_json::to_vec(&account_message).unwrap()
                        }
                        "/project" => {
                            let accounts = accounts.read().unwrap();
                            let account = &accounts[sessions.get(&cookie).unwrap().0];
                            let mut projection_message = HashMap::<String, Vec<u64>, _>::new();
                            if let Some(&count) = query.get("count") {
                                projection_message
                                    .insert(
                                        "savings".into(),
                                        SavingsAccount::new()
                                            .project(account.get_balance(), count)
                                            .unwrap()
                                    );
                                projection_message
                                    .insert(
                                        "current".into(),
                                        CurrentAccount
                                            .project(account.get_balance(), count)
                                            .unwrap()
                                );
                                serde_json::to_vec(&projection_message).unwrap()
                            } else {
                                Vec::new()
                            }
                        }
                        _ => {
                            read((directory()+&path.as_str()).replace('/',"\\").replace("%20", " ")).unwrap_or_else(|e| {
                                println!("{:?}",e);
                                not_found = true;
                                read(directory()+"/404.html").unwrap()
                            })
                        }
                    };
                    let mut response = Response::new(html.into());
                    match path.as_str().split_once('.') {
                        Some((_, s @ ("jpg"|"png"))) => {
                            if not_found {
                                response.headers_mut()
                                    .insert("Content-Type", str::parse("text/html")
                                        .unwrap()
                                    );
                            } else {
                                response.headers_mut()
                                    .insert("Content-Type", str::parse(&("image/".to_owned() + s))
                                        .unwrap()
                                    );
                            }
                        }
                        Some((_, "")) => {
                            response.headers_mut()
                                .insert("Content-Type", str::parse(&("text/json"))
                                    .unwrap()
                                );
                        }
                        Some((_, s)) => {
                            response.headers_mut()
                                .insert("Content-Type", str::parse(&("text/".to_owned() + s))
                                    .unwrap()
                                );
                        }
                        None => {}
                    }
                    response
                } else {
                    println!("{path:?}");
                    let mut not_found = false;
                    let mut response;
                    match path.as_str() {
                        "/landing.html" => {
                            let body = read(directory()+path.as_str()).unwrap();
                            response = Response::new(body.into());
                            let _ = response.headers_mut().insert("Content-Type", "text/html".parse().unwrap());
                        }
                        "/style.css" => {
                            let body = read(directory()+path.as_str()).unwrap();
                            response = Response::new(body.into());
                            let _ = response.headers_mut().insert("Content-Type", "text/css".parse().unwrap());
                        }
                        "/logo.png" => {
                            let body = read(directory()+path.as_str()).unwrap();
                            response = Response::new(body.into());
                            let _ = response.headers_mut().insert("Content-Type", "image/png".parse().unwrap());
                        }
                        "/login.html" => {
                            let body = read(directory()+path.as_str()).unwrap();
                            response = Response::new(body.into());
                            let _ = response.headers_mut().insert("Content-Type", "text/html".parse().unwrap());
                        }
                        "/register.html" => {
                            let body = read(directory()+path.as_str()).unwrap();
                            response = Response::new(body.into());
                            let _ = response.headers_mut().insert("Content-Type", "text/html".parse().unwrap());
                            let _ = response.headers_mut().insert("Content-Type", "text/html".parse().unwrap());
                        }
                        "/favicon.ico" => {
                            let body = Vec::new();
                            response = Response::new(body.into());
                            let _ = response.headers_mut().insert("Content-Type", "text/html".parse().unwrap());
                            let _ = response.headers_mut().insert("Content-Type", "text/html".parse().unwrap());
                        }
                        _ => {
                            let body = read(directory()+"/landing.html").unwrap();
                            response = Response::new(body.into());
                            let mut headers = response.headers_mut();
                            let _ = headers
                                .insert("Content-Type", "text/html".parse().unwrap());
                            let _ = headers
                                .insert("Location", "/landing.html".parse().unwrap());
                            *response.status_mut() = StatusCode::SEE_OTHER;
                            *response.status_mut() = StatusCode::SEE_OTHER;
                        }
                    }
                    response
                }
            })
    };


    let unauthorised = warp::path::full()
        .map(move |path: warp::filters::path::FullPath| {
            println!("{path:?}");
            let mut not_found = false;
            let mut response;
            match path.as_str() {
                "/landing.html" => {
                    let body = read(directory()+path.as_str()).unwrap();
                    response = Response::new(body.into());
                    let _ = response.headers_mut().insert("Content-Type", "text/html".parse().unwrap());
                }
                "/style.css" => {
                    let body = read(directory()+path.as_str()).unwrap();
                    response = Response::new(body.into());
                    let _ = response.headers_mut().insert("Content-Type", "text/css".parse().unwrap());
                }
                "/logo.png" => {
                    let body = read(directory()+path.as_str()).unwrap();
                    response = Response::new(body.into());
                    let _ = response.headers_mut().insert("Content-Type", "image/png".parse().unwrap());
                }
                "/login.html" => {
                    let body = read(directory()+path.as_str()).unwrap();
                    response = Response::new(body.into());
                    let _ = response.headers_mut().insert("Content-Type", "text/html".parse().unwrap());
                }
                "/register.html" => {
                    let body = read(directory()+path.as_str()).unwrap();
                    response = Response::new(body.into());
                    let _ = response.headers_mut().insert("Content-Type", "text/html".parse().unwrap());
                    let _ = response.headers_mut().insert("Content-Type", "text/html".parse().unwrap());
                }
                _ => {
                    let body = read(directory()+"/landing.html").unwrap();
                    response = Response::new(body.into());
                    let mut headers = response.headers_mut();
                    let _ = headers
                        .insert("Content-Type", "text/html".parse().unwrap());
                    let _ = headers
                        .insert("Location", "/landing.html".parse().unwrap());
                    *response.status_mut() = StatusCode::SEE_OTHER;
                }
            }
            response
        });

    let auth_or_no = warp::get().and(authorised.or(unauthorised));

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
                let register_result = register(form_response.clone(), user_data.clone(), accounts.clone(), directory);
                handle_registration(register_result, directory)
            })
    };
    let login_or_register = warp::post().and(login.or(register));
    warp::serve(auth_or_no.or(login_or_register))
        .tls()
        .cert_path("certificate/investquest.test.crt")
        .key_path("certificate/investquest.test.key")
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
    if account.verify_password(password) {
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
    PasswordsDontMatch,
    SaveError
}

fn register(form_response: HashMap<String, String>, user_data: Arc<RwLock<HashMap<String, usize>>>, accounts: Arc<RwLock<Vec<Account>>>, directory: impl Fn()->String) -> Result<(), RegisterError> {
    println!("{:?}", form_response);
    let mut user_data = user_data.write().unwrap();
    let mut accounts = accounts.write().unwrap();
    let username = form_response
        .get("username")
        .ok_or(RegisterError::MissingValue)?
        .as_str();
    let email = form_response
        .get("e-mail")
        .ok_or(RegisterError::MissingValue)?
        .as_str();
    let password = form_response
        .get("password")
        .ok_or(RegisterError::MissingValue)?
        .as_bytes();
    let second_password = form_response
        .get("passwordconfirm")
        .ok_or(RegisterError::MissingValue)?
        .as_bytes();
    if password!=second_password {
        return Err(RegisterError::PasswordsDontMatch)
    }
    if user_data.contains_key(username) {
        return Err(RegisterError::AlreadyExists);
    }
    let account = Account::new(username, password, email);
    let _ = user_data.insert(username.to_owned(), accounts.len());
    let mut accounts_file = File::options().write(true).create(true).open("accounts.json").map_err(|_| RegisterError::SaveError)?;

    //seek to one off the end and then insert a ',' in place of the ']' if possible,
    //otherwise seek to start and insert a '['
    if let Err(_) = accounts_file.seek(SeekFrom::End(-1)) {
        accounts_file.seek(SeekFrom::Start(0)).map_err(|_| RegisterError::SaveError)?;
        accounts_file.write(&[b'[']).map_err(|_| RegisterError::SaveError)?;
    } else {
        println!("here");
        accounts_file.write(&[b',']).map_err(|_| RegisterError::SaveError)?;
    }

    //append the account json to the file
    serde_json::to_writer(&accounts_file, &account).map_err(|_| RegisterError::SaveError)?;

    //close the json vector
    accounts_file.write(&[b']']).map_err(|_| RegisterError::SaveError)?;

    //add the account to the runtime list
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