use anyhow::{Context, Error};
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use paste::paste;
use serde::{Deserialize, Serialize};
use shuttle_axum::ShuttleAxum;
use sqlx::{database::HasArguments, migrate::Migrator, query::Query, FromRow, PgPool, Postgres};
use time::Date;
use tracing::{error, info};
use uuid::Uuid;

static MIGRATOR: Migrator = sqlx::migrate!();

macro_rules! get {
    ($name:ident) => {
        paste! {
            async fn [<get_ $name:lower s>](
                State(app_state): State<AppState>,
            ) -> Result<impl IntoResponse, StatusCode> {
                let sql = concat!("SELECT * FROM ", stringify!([<$name:lower>]));
                sqlx::query_as::<_, $name>(&sql)
                    .fetch_all(&app_state.pool)
                    .await
                    .context(sql)
                    .map_err(internal_server_error)
                    .map(Json)
            }
        }
    };
}

macro_rules! post {
    ($name:ident, $bind:expr, $args:literal) => {
        paste! {
            async fn [<post_ $name:lower>](
                State(app_state): State<AppState>,
                Json(entity): Json<$name>,
            ) -> Result<impl IntoResponse, StatusCode> {
                let sql = concat!(
                    "INSERT INTO ",
                    stringify!([<$name:lower>]),
                    " VALUES (",
                    $args,
                    ")"
                );
                let query = $bind(entity, sqlx::query(&sql));
                query
                    .execute(&app_state.pool)
                    .await
                    .context(sql)
                    .map_err(internal_server_error)
                    .map(|_| StatusCode::CREATED)
            }
        }
    };
}

#[shuttle_runtime::main]
async fn ticklist_main(#[shuttle_shared_db::Postgres] pool: PgPool) -> ShuttleAxum {
    MIGRATOR.run(&pool).await.context("run migrations")?;

    let router = Router::new()
        .route("/", get(ready))
        .route("/crags", get(get_crags).post(post_crag))
        .route("/routes", get(get_routes).post(post_route))
        .route("/ascents", get(get_ascents).post(post_ascent))
        .with_state(AppState { pool });

    Ok(router.into())
}

#[derive(Clone)]
struct AppState {
    pool: PgPool,
}

async fn ready() -> impl IntoResponse {
    StatusCode::OK
}

// Crag ============================================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
struct Crag {
    #[serde(default)]
    id: Uuid,
    name: String,
    location: String,
}

get!(Crag);

post!(Crag, bind_post_crag, "$1, $2, $3");

fn bind_post_crag<'q>(
    crag: Crag,
    query: Query<'q, Postgres, <Postgres as HasArguments<'q>>::Arguments>,
) -> Query<'q, Postgres, <Postgres as HasArguments<'q>>::Arguments> {
    query
        .bind(Uuid::now_v7())
        .bind(crag.name)
        .bind(crag.location)
}

// Route ===========================================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
struct Route {
    #[serde(default)]
    id: Uuid,
    crag_id: Uuid,
    name: String,
}

get!(Route);

post!(Route, bind_post_route, "$1, $2, $3");

fn bind_post_route<'q>(
    route: Route,
    query: Query<'q, Postgres, <Postgres as HasArguments<'q>>::Arguments>,
) -> Query<'q, Postgres, <Postgres as HasArguments<'q>>::Arguments> {
    query
        .bind(Uuid::now_v7())
        .bind(route.crag_id)
        .bind(route.name)
}

// Ascent ==========================================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
struct Ascent {
    #[serde(default)]
    id: Uuid,
    route_id: Uuid,
    date: Date,
}

get!(Ascent);

post!(Ascent, bind_post_ascent, "$1, $2, $3");

fn bind_post_ascent<'q>(
    ascent: Ascent,
    query: Query<'q, Postgres, <Postgres as HasArguments<'q>>::Arguments>,
) -> Query<'q, Postgres, <Postgres as HasArguments<'q>>::Arguments> {
    info!("{ascent:?}");
    query
        .bind(Uuid::now_v7())
        .bind(ascent.route_id)
        .bind(ascent.date)
}

// Misc ============================================================================================

fn internal_server_error(error: Error) -> StatusCode {
    error!(
        error = display(format!("{error:#}")),
        backtrace = %error.backtrace(),
        "internal server error"
    );
    StatusCode::INTERNAL_SERVER_ERROR
}
