use std::collections::HashMap;

use axum::http::StatusCode;
use axum_test::TestServer;
use flern::{build_server_with_pool, model::DbConnection};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use sqlx::{Executor, PgPool, postgres::PgPoolOptions};
use tower_cookies::Cookie;
use url::Url;
use uuid::Uuid;

pub async fn setup_test_db() -> FlowDatabase {
    let _ = dotenvy::dotenv();
    let db_name = format!("test_db_{}", Uuid::new_v4());
    let admin_url = std::env::var("TEST_DATABASE_ADMIN_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/postgres".to_string());

    let mut url = Url::parse(&admin_url).unwrap();

    let admin_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(url.as_str())
        .await
        .unwrap();

    admin_pool
        .execute(format!(r#"CREATE DATABASE "{}""#, db_name).as_str())
        .await
        .unwrap();

    url.set_path(&db_name);

    let test_db_url = url.to_string();

    let pool = PgPool::connect(&test_db_url).await.unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();

    FlowDatabase { db_name, pool }
}

/// `FlowDatabase` represents temporary postgres database. This database deletes on `Drop`(when it
/// comes out of scope)
// FIXME: Drop database even if the test panics
pub struct FlowDatabase {
    db_name: String,
    pool: PgPool,
}

impl Drop for FlowDatabase {
    fn drop(&mut self) {
        let db_name = self.db_name.clone();
        let admin_url = std::env::var("TEST_DATABASE_ADMIN_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/postgres".to_string());

        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn_blocking(move || {
                // fresh runtime inside this blocking thread
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    if let Ok(admin_pool) = PgPool::connect(&admin_url).await {
                        admin_pool
                            .execute(format!(r#"DROP DATABASE "{}" WITH (FORCE)"#, db_name).as_str())
                            .await.expect("Unable to drop database");
                    }
                });
            });
        }
    }
}

pub async fn setup_server(pool: &FlowDatabase) -> TestServer {
    let pool = DbConnection::from_pool(pool.pool.clone());
    let server = build_server_with_pool(pool).await.unwrap().1;
    TestServer::new(server).unwrap()
}

#[derive(Debug)]
pub struct FlowContext {
    pub store: HashMap<&'static str, Value>, // a way to pass data between steps
}

impl FlowContext {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    pub fn store(&mut self, key: &'static str, val: Value) {
        self.store.insert(key, val);
    }

    pub fn get(&self, key: &str) -> &Value {
        self.store.get(key).expect("missing store key")
    }

    pub fn get_json<'de, T>(&self, key: &str) -> T
    where
        T: DeserializeOwned,
    {
        let obj = self.get(key);
        let de: T = serde_json::from_value(obj.clone()).expect("Invalid json format");
        de
    }
}

pub struct Action {
    #[allow(unused)]
    pub name: &'static str,
    pub method: &'static str,
    pub path: String,
    pub dyn_path: Option<Box<dyn Fn(&FlowContext) -> String + Send + Sync>>,
    pub body: Option<Value>,
    pub dyn_body: Option<Box<dyn Fn(&FlowContext) -> Value + Send + Sync>>,
    pub expect: StatusCode,
    pub clear_cookies: bool,
    pub save_cookies: bool,
    pub query_params: Vec<(String, String)>,
    pub cookie_asserts: Vec<(&'static str, Box<dyn Fn(&Cookie) + Send + Sync>)>,
    pub body_asserts: Vec<Box<dyn Fn(&str) + Send + Sync>>,
    pub save_as: Option<&'static str>,
}

impl Action {
    pub fn new(name: &'static str, method: &'static str, path: &'static str) -> Self {
        Self {
            name,
            method,
            path: path.to_string(),
            dyn_path: None,
            body: None,
            dyn_body: None,
            expect: StatusCode::OK,
            clear_cookies: false,
            save_cookies: true,
            query_params: vec![],
            cookie_asserts: vec![],
            body_asserts: vec![],
            save_as: None,
        }
    }

    pub fn with_body(mut self, body: Value) -> Self {
        self.body = Some(body);
        self
    }

    pub fn with_expect(mut self, expect: StatusCode) -> Self {
        self.expect = expect;
        self
    }

    pub fn with_save_cookies(mut self, save_cookies: bool) -> Self {
        self.save_cookies = save_cookies;
        self
    }

    pub fn with_clear_cookies(mut self, clear_cookies: bool) -> Self {
        self.clear_cookies = clear_cookies;
        self
    }

    pub fn with_param(mut self, key: &str, val: &str) -> Self {
        self.query_params
            .push((String::from(key), String::from(val)));
        self
    }

    pub fn with_dyn_path<F>(mut self, f: F) -> Self
    where
        F: Fn(&FlowContext) -> String + Send + Sync + 'static,
    {
        self.dyn_path = Some(Box::new(f));
        self
    }

    #[allow(unused)]
    pub fn with_dyn_body<F>(mut self, f: F) -> Self
    where
        F: Fn(&FlowContext) -> Value + Send + Sync + 'static,
    {
        self.dyn_body = Some(Box::new(f));
        self
    }

    pub fn with_save_as(mut self, key: &'static str) -> Self {
        self.save_as = Some(key);
        self
    }

    pub fn assert_cookie<F>(mut self, name: &'static str, check: F) -> Self
    where
        F: Fn(&Cookie) + Send + Sync + 'static,
    {
        self.cookie_asserts.push((name, Box::new(check)));
        self
    }

    pub fn assert_body<F>(mut self, check: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.body_asserts.push(Box::new(check));
        self
    }
}

pub struct Flow {
    actions: Vec<Action>,
}

impl Flow {
    pub fn new() -> Self {
        Self { actions: vec![] }
    }

    pub fn step(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }

    pub async fn run(self, server: &mut TestServer, _db: FlowDatabase) {
        let mut ctx = FlowContext::new(); // create new context for this flow
        for action in self.actions {
            println!("==> Running test action `{}`", action.name);
            if action.clear_cookies {
                server.clear_cookies();
            }

            if action.save_cookies {
                server.save_cookies();
            } else {
                server.do_not_save_cookies();
            }

            let path = if let Some(dyn_path_fn) = action.dyn_path {
                dyn_path_fn(&ctx)
            } else {
                action.path.clone()
            };

            let mut req = match action.method {
                "GET" => server.get(&path),
                "POST" => server.post(&path),
                "PUT" => server.put(&path),
                "DELETE" => server.delete(&path),
                _ => panic!("unsupported method {}", action.method),
            };

            match (action.dyn_body, action.body) {
                (Some(f), _) => {
                    req = req.json(&f(&ctx));
                }
                (_, Some(json)) => req = req.json(&json),
                _ => {}
            }

            if !action.query_params.is_empty() {
                for (k, v) in action.query_params {
                    req = req.add_query_param(&k, v);
                }
            }

            let resp = req.await;
            resp.assert_status(action.expect);
            let cookies = resp.cookies();

            if !action.cookie_asserts.is_empty() {
                for (cookie_name, check) in action.cookie_asserts {
                    let cookie = cookies
                        .get(cookie_name)
                        .unwrap_or_else(|| panic!("Cookie {} is not set", cookie_name));
                    check(cookie);
                }
            }

            if !action.body_asserts.is_empty() {
                let body = resp.json::<Value>();
                let body = serde_json::to_string(&body)
                    .unwrap_or_else(|_| panic!("Unable to serialize body to string"));
                for check in action.body_asserts {
                    check(&body);
                }
            }

            if let Some(save_key) = action.save_as {
                let body = resp.json::<Value>();
                ctx.store(save_key, body);
            }
        }
    }
}

// Common actions builders

pub fn signup_action(name: &str, password: &str) -> Action {
    Action::new("signup", "POST", "/api/v1/account/signup").with_body(json!({
        "username": name,
        "password": password,
    }))
}

pub fn signin_action(name: &str, password: &str) -> Action {
    Action::new("signin", "POST", "/api/v1/account/signin").with_body(json!({
        "username": name,
        "password": password,
    }))
}

pub fn signin_admin_action() -> Action {
    Action::new("signin_admin", "POST", "/api/v1/account/signin").with_body(json!({
        "username": "admin",
        "password": "admin",
    }))
}
