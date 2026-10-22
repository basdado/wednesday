use std::{env, time::Duration};

use axum::{
    Router,
    extract::State,
    http::{HeaderValue, StatusCode, header},
    response::{Html, IntoResponse},
    routing::get,
};
use jiff::{Timestamp, civil::Weekday, tz::TimeZone};
use tower_http::{set_header::SetResponseHeaderLayer, timeout::TimeoutLayer};

macro_rules! asset {
    ($path:literal, $content_type:literal) => {
        (
            concat!("/", $path),
            $content_type,
            include_bytes!(concat!("../public-html/", $path)).as_slice(),
        )
    };
}

const ASSETS: &[(&str, &str, &[u8])] = &[
    asset!("demfrog.jpg", "image/jpeg"),
    asset!("demtadpole.jpg", "image/jpeg"),
    asset!("favicon.ico", "image/x-icon"),
    asset!("favicon-16x16.png", "image/png"),
    asset!("favicon-32x32.png", "image/png"),
    asset!("apple-touch-icon.png", "image/png"),
    asset!("android-chrome-144x144.png", "image/png"),
    asset!("mstile-150x150.png", "image/png"),
    asset!("safari-pinned-tab.svg", "image/svg+xml"),
    asset!("site.webmanifest", "application/manifest+json"),
    asset!("browserconfig.xml", "application/xml"),
];

const TEMPLATE: &str = include_str!("../public-html/index.html");

struct Page {
    title: &'static str,
    og_title: &'static str,
    og_description: &'static str,
    image: &'static str,
}

const WEDNESDAY: Page = Page {
    title: "It is wednesday, my dudes!",
    og_title: "It is wednesday, my dudes!",
    og_description: "Aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    image: "demfrog.jpg",
};

const NOT_WEDNESDAY: Page = Page {
    title: "It is not quite wednesday, my pals",
    og_title: "It is not quite wednesday",
    og_description: "My pals",
    image: "demtadpole.jpg",
};

fn render(page: &Page) -> String {
    TEMPLATE
        .replace("{{title}}", page.title)
        .replace("{{og_title}}", page.og_title)
        .replace("{{og_description}}", page.og_description)
        .replace("{{image}}", page.image)
}

#[derive(Clone)]
struct AppState {
    tz: TimeZone,
    wednesday_html: &'static str,
    not_wednesday_html: &'static str,
}

async fn index(State(state): State<AppState>) -> impl IntoResponse {
    let today = Timestamp::now().to_zoned(state.tz.clone()).weekday();
    let html = if today == Weekday::Wednesday {
        state.wednesday_html
    } else {
        state.not_wednesday_html
    };
    // The page changes at midnight, so crawlers and browsers must not cache it.
    ([(header::CACHE_CONTROL, "no-cache")], Html(html))
}

fn time_zone() -> TimeZone {
    match env::var("TZ") {
        Ok(name) if !name.is_empty() => TimeZone::get(&name)
            .unwrap_or_else(|err| panic!("invalid TZ {name:?}: {err}")),
        _ => TimeZone::UTC,
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.ok();
    };
    let terminate = async {
        if let Ok(mut sig) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            sig.recv().await;
        }
    };
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let state = AppState {
        tz: time_zone(),
        wednesday_html: render(&WEDNESDAY).leak(),
        not_wednesday_html: render(&NOT_WEDNESDAY).leak(),
    };

    let mut router = Router::new().route("/", get(index));
    for &(path, content_type, body) in ASSETS {
        router = router.route(
            path,
            get(move || async move {
                (
                    [
                        (header::CONTENT_TYPE, content_type),
                        (header::CACHE_CONTROL, "public, max-age=86400"),
                    ],
                    body,
                )
            }),
        );
    }

    let app = router
        .with_state(state)
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(10),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ));

    let port = env::var("PORT").unwrap_or_else(|_| "8080".into());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("failed to bind");
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}
