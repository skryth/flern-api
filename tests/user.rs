mod common;
use reqwest::StatusCode;
use flern::model::entity::UserEntity;
use flern::web::middlewares::AUTH_TOKEN;
use serde_json::json;
use tower_cookies::cookie::SameSite;

use crate::common::{
    Action, Flow, setup_server, setup_test_db, signin_action, signin_admin_action, signup_action,
};

#[tokio::test]
async fn route_signup_test() {
    let pool = setup_test_db().await;
    let mut server = setup_server(&pool).await;

    Flow::new()
        .step(
            signup_action("foobar", "foobaz")
                .assert_cookie(AUTH_TOKEN, |cookie| {
                    assert_eq!(cookie.same_site(), Some(SameSite::Lax));
                    assert_eq!(cookie.path(), Some("/"));
                    assert_eq!(cookie.http_only(), Some(true));
                })
                .assert_body(|body| {
                    let ent: UserEntity = serde_json::from_str(body).expect("Invalid body format");
                    assert_eq!(ent.username(), "foobar");
                })
                .with_expect(StatusCode::OK),
        )
        // try to signup twice
        .step(signup_action("foobar", "foobaz").with_expect(StatusCode::CONFLICT))
        .run(&mut server, pool)
        .await;
}

#[tokio::test]
async fn route_signin_test() {
    let pool = setup_test_db().await;
    let mut server = setup_server(&pool).await;

    Flow::new()
        .step(signup_action("SIGNINTEST", "SIGNINTEST").with_save_cookies(false))
        .step(
            signin_action("SIGNINTEST", "SIGNINTEST")
                .assert_cookie(AUTH_TOKEN, |cookie| {
                    assert_eq!(cookie.same_site(), Some(SameSite::Lax));
                    assert_eq!(cookie.path(), Some("/"));
                    assert_eq!(cookie.http_only(), Some(true));
                })
                .assert_body(|body| {
                    let ent: UserEntity = serde_json::from_str(body).expect("Invalid JSON format");
                    assert_eq!(ent.username(), "SIGNINTEST");
                })
                .with_expect(StatusCode::OK)
                .with_clear_cookies(true),
        )
        // wrong credentials
        .step(
            signin_action("SIGNINTEST", "WRONGPASSWORD")
                .with_save_cookies(false)
                .with_clear_cookies(true)
                .assert_body(|body| {
                    assert!(body.contains("Authentication error"));
                })
                .with_expect(StatusCode::UNAUTHORIZED),
        )
        // non-existing account
        .step(
            signin_action("nonexisting", "nvm")
                .with_expect(StatusCode::UNAUTHORIZED)
                .assert_body(|body| assert!(body.contains("Authentication error"))),
        )
        .run(&mut server, pool)
        .await;
}

#[tokio::test]
async fn route_user_list_test() {
    let pool = setup_test_db().await;
    let mut server = setup_server(&pool).await;

    Flow::new()
        .step(signup_action("FOOBAR", "FOOBAZ").with_save_cookies(true))
        // try to request without admin perms
        .step(
            Action::new("user_list", "GET", "/api/v1/account/page")
                .assert_body(|body| {
                    assert!(body.contains("error"));
                })
                .with_param("limit", "5")
                .with_param("offset", "0")
                .with_expect(StatusCode::FORBIDDEN)
                .with_save_cookies(true),
        )
        // acquire admin account
        .step(signin_admin_action())
        .step(
            Action::new("user_list", "GET", "/api/v1/account/page")
                .with_param("limit", "5")
                .with_param("offset", "0")
                .assert_body(|body| {
                    assert!(body.contains("total"));
                    assert!(body.contains("items"));
                })
                .with_expect(StatusCode::OK),
        )
        .run(&mut server, pool)
        .await;
}

#[tokio::test]
async fn route_user_update_test() {
    let pool = setup_test_db().await;
    let mut server = setup_server(&pool).await;

    Flow::new()
        // create a pair of users and save their data to `foobar_user` and `foobar2_user`
        .step(
            signup_action("FOOBAR", "FOOBAZ")
                .with_save_cookies(false)
                .with_save_as("foobar_user"),
        )
        .step(
            signup_action("FOOBAR2", "FOOBAZ2")
                .with_save_cookies(true)
                .with_save_as("foobar2_user"),
        )
        // try to update `foobar_user` without permissions
        .step(
            Action::new("user_update", "PUT", "dynamic")
                .with_dyn_path(|ctx| {
                    let foobar_id = ctx.get("foobar_user");
                    let user: UserEntity =
                        serde_json::from_value(foobar_id.clone()).expect("Invalid body format.");
                    format!("/api/v1/account/{}", user.id())
                })
                .with_body(json!({
                    "username": "should fail",
                    "email": "should fail",
                    "password": "should fail",
                }))
                .with_expect(StatusCode::FORBIDDEN)
                .assert_body(|body| {
                    assert!(body.contains("error"));
                }),
        )
        // try to update self, this one should work
        .step(
            Action::new("user_update", "PUT", "dynamic")
                .with_dyn_path(|ctx| {
                    let foobar2 = ctx.get("foobar2_user");
                    let user: UserEntity =
                        serde_json::from_value(foobar2.clone()).expect("Invalid body format.");
                    format!("/api/v1/account/{}", user.id())
                })
                .with_expect(StatusCode::OK)
                .with_body(json!({
                    "username": "FOOBAR3",
                    "password": "doesn't make any sense",
                }))
                .assert_body(|body| {
                    assert!(body.contains("FOOBAR3"));
                }),
        )
        // login as admin to test admin perms
        .step(
            signin_admin_action()
                .with_save_cookies(true)
                .with_clear_cookies(true),
        )
        // try to update foobar with admin perms
        .step(
            Action::new("user_update", "PUT", "dynamic")
                .with_dyn_path(|ctx| {
                    let foobar = ctx.get_json::<UserEntity>("foobar_user");
                    format!("/api/v1/account/{}", foobar.id())
                })
                .with_body(json!({
                    "username": "FOOBAR4",
                    "password": "doesn't make any sense",
                }))
                .with_expect(StatusCode::OK)
                .assert_body(|body| {
                    assert!(body.contains("FOOBAR4"));
                }),
        )
        // try to update foobar to the name of the existing user. This one should fail.
        .step(
            Action::new("user_update", "PUT", "dynamic")
                .with_dyn_path(|ctx| {
                    let foobar = ctx.get_json::<UserEntity>("foobar_user");
                    format!("/api/v1/account/{}", foobar.id())
                })
                .with_body(json!({
                    "username": "FOOBAR3",
                    "password": "doesn't make any sense",
                }))
                .with_expect(StatusCode::CONFLICT)
                .assert_body(|body| {
                    assert!(body.contains("error"));
                }),
        )
        .run(&mut server, pool)
        .await;
}

#[tokio::test]
async fn route_user_delete_test() {
    let pool = setup_test_db().await;
    let mut server = setup_server(&pool).await; 

    Flow::new()
        .step(signup_action("FOOBAR", "FOOBAZ").with_save_cookies(false).with_save_as("foobar"))
        .step(signup_action("FOOBAZ", "FOOBAR").with_save_cookies(true).with_save_as("foobaz"))
        // we can't allow everybody to delete anybody ;D
        .step(
            Action::new("user_delete", "DELETE", "dynamic")
                .with_dyn_path(|ctx| {
                    let foobar = ctx.get_json::<UserEntity>("foobar");
                    format!("/api/v1/account/{}", foobar.id())
                })
                .with_expect(StatusCode::FORBIDDEN)
                .assert_body(|body| {
                    assert!(body.contains("error"));
                })
        )
        // self deletion is allowed
        .step(
            Action::new("user_delete", "DELETE", "dynamic")
                .with_dyn_path(|ctx| {
                    let foobaz = ctx.get_json::<UserEntity>("foobaz");
                    format!("/api/v1/account/{}", foobaz.id())
                })
                .with_expect(StatusCode::OK)
        )
        .step(signin_admin_action())
        // even admin cannot delete the user which doesn't exist :)
        .step(
            Action::new("user_delete", "DELETE", "dynamic")
                .with_dyn_path(|ctx| {
                    let foobaz = ctx.get_json::<UserEntity>("foobaz");
                    format!("/api/v1/account/{}", foobaz.id())
                })
                .with_expect(StatusCode::NOT_FOUND)
        )
        // admin can delete every user he wants
        .step(
            Action::new("user_delete", "DELETE", "dynamic")
                .with_dyn_path(|ctx| {
                    let foobar = ctx.get_json::<UserEntity>("foobar");
                    format!("/api/v1/account/{}", foobar.id())
                })
                .with_expect(StatusCode::OK)
        )
        .run(&mut server, pool)
        .await;
}
