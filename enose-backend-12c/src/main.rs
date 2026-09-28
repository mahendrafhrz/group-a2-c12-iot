use anyhow::Result;
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderValue, StatusCode, header},
    response::Html,
    response::{IntoResponse, Response},
    routing::get,
};
use chrono::{DateTime, Utc};
use printpdf::path::{PaintMode, WindingOrder};
use printpdf::{BuiltinFont, Color, Mm, PdfDocument, PdfLayerReference, Point, Polygon, Rgb};
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use serde::{Deserialize, Serialize};
use sqlx::{AnyPool, FromRow, any::AnyPoolOptions};
use std::{
    borrow::Borrow, env, fs::OpenOptions, io::BufWriter, path::Path, sync::Arc, time::Duration,
};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    pool: AnyPool,
}

#[derive(Debug, Deserialize)]
struct CreateMeasurement {
    sample_id: i32,
    score: f64,
    accuracy: f64,
    source: Option<String>,
    device_id: Option<String>,
    features: Option<Vec<f64>>,
    captured_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, FromRow)]
struct Measurement {
    id: String,
    sample_id: i32,
    score: f64,
    accuracy: f64,
    source: String,
    device_id: Option<String>,
    features_json: Option<String>,
    captured_at: String,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct AnalyticsSummary {
    total_measurements: i64,
    sample_count: i64,
    average_score: f64,
    average_accuracy: f64,
    latest_captured_at: Option<String>,
    by_sample: Vec<SampleSummary>,
}

#[derive(Debug, Serialize, FromRow)]
struct SampleSummary {
    sample_id: i32,
    measurement_count: i64,
    average_score: f64,
    average_accuracy: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(env("RUST_LOG", "info"))
        .init();

    let database_url = env("DATABASE_URL", "sqlite://./enose.db");
    if let Some(path) = database_url.strip_prefix("sqlite://./") {
        if !Path::new(path).exists() {
            OpenOptions::new().create(true).write(true).open(path)?;
        }
    }
    sqlx::any::install_default_drivers();
    let pool = AnyPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS measurements (
            id TEXT PRIMARY KEY,
            sample_id INTEGER NOT NULL CHECK(sample_id BETWEEN 1 AND 18),
            score REAL NOT NULL CHECK(score BETWEEN 0 AND 100),
            accuracy REAL NOT NULL CHECK(accuracy BETWEEN 0 AND 100),
            source TEXT NOT NULL DEFAULT 'api',
            device_id TEXT,
            features_json TEXT,
            captured_at TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    let _ = sqlx::query("ALTER TABLE measurements ADD COLUMN source TEXT NOT NULL DEFAULT 'api'")
        .execute(&pool)
        .await;

    let state = AppState { pool };
    if env("MQTT_ENABLED", "false").eq_ignore_ascii_case("true") {
        tokio::spawn(mqtt_worker(state.clone()));
    }

    let app = Router::new()
        .route("/health", get(health))
        .route("/dashboard", get(dashboard))

        .route(
            "/api/v1/measurements",
            get(list_measurements).post(create_measurement),
        )
        .route("/api/v1/measurements/{id}", axum::routing::delete(delete_measurement))

        .route("/api/v1/analytics/summary", get(summary))
        .route("/api/v1/reports/analytics.pdf", get(analytics_pdf))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::new(state));

    let address = format!("{}:{}", env("HOST", "0.0.0.0"), env("PORT", "8080"));
    let listener = TcpListener::bind(&address).await?;
    info!("E-Nose cloud backend listening on http://{}", address);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok", "service": "enose-cloud"}))
}

async fn dashboard() -> Html<&'static str> {
    Html(include_str!("../dashboard.html"))
}

async fn list_measurements(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Measurement>>, ApiError> {
    let rows = sqlx::query_as::<_, Measurement>(
        "SELECT id, sample_id, CAST(score AS DOUBLE PRECISION) AS score,
         CAST(accuracy AS DOUBLE PRECISION) AS accuracy, source, device_id, features_json, captured_at, created_at
         FROM measurements ORDER BY captured_at DESC LIMIT 500",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn create_measurement(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateMeasurement>,
) -> Result<(StatusCode, Json<Measurement>), ApiError> {
    validate(&payload)?;
    let id = Uuid::new_v4().to_string();
    let captured_at = payload.captured_at.unwrap_or_else(Utc::now).to_rfc3339();
    let created_at = Utc::now().to_rfc3339();
    let features_json = payload
        .features
        .map(|features| serde_json::to_string(&features))
        .transpose()?;

    let insert_sql = if is_postgres() {
        "INSERT INTO measurements (id, sample_id, score, accuracy, source, device_id, features_json, captured_at, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
    } else {
        "INSERT INTO measurements (id, sample_id, score, accuracy, source, device_id, features_json, captured_at, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    };
    sqlx::query(insert_sql)
        .bind(&id)
        .bind(payload.sample_id)
        .bind(payload.score)
        .bind(payload.accuracy)
        .bind(payload.source.unwrap_or_else(|| "api".to_string()))
        .bind(&payload.device_id)
        .bind(&features_json)
        .bind(captured_at)
        .bind(created_at)
        .execute(&state.pool)
        .await?;

    let select_sql = if is_postgres() {
        "SELECT id, sample_id, CAST(score AS DOUBLE PRECISION) AS score,
         CAST(accuracy AS DOUBLE PRECISION) AS accuracy, source, device_id, features_json, captured_at, created_at
         FROM measurements WHERE id = $1"
    } else {
        "SELECT id, sample_id, score, accuracy, source, device_id, features_json, captured_at, created_at FROM measurements WHERE id = ?"
    };
    let measurement = sqlx::query_as::<_, Measurement>(select_sql)
        .bind(id)
        .fetch_one(&state.pool)
        .await?;
    Ok((StatusCode::CREATED, Json(measurement)))
}

async fn delete_measurement(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<StatusCode, ApiError> {
    let delete_sql = if is_postgres() {
        "DELETE FROM measurements WHERE id = $1"
    } else {
        "DELETE FROM measurements WHERE id = ?"
    };
    
    let result = sqlx::query(delete_sql)
        .bind(&id)
        .execute(&state.pool)
        .await?;
    
    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    
    Ok(StatusCode::NO_CONTENT)
}

async fn summary(State(state): State<Arc<AppState>>) -> Result<Json<AnalyticsSummary>, ApiError> {
    let total_measurements = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM measurements")
    .fetch_one(&state.pool)
    .await?;
    let average_score = sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(AVG(score), 0.0) FROM measurements",
    )
    .fetch_one(&state.pool)
    .await?;
    let average_accuracy = sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(AVG(accuracy), 0.0) FROM measurements",
    )
    .fetch_one(&state.pool)
    .await?;
    let latest_captured_at = sqlx::query_scalar::<_, Option<String>>(
        "SELECT MAX(captured_at) FROM measurements",
    )
    .fetch_one(&state.pool)
    .await?;
    let sample_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(DISTINCT sample_id) FROM measurements")
            .fetch_one(&state.pool)
            .await?;
    let by_sample = sqlx::query_as::<_, SampleSummary>(
        "SELECT sample_id, COUNT(*) AS measurement_count, AVG(score) AS average_score, AVG(accuracy) AS average_accuracy
         FROM measurements GROUP BY sample_id ORDER BY sample_id",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(AnalyticsSummary {
        total_measurements,
        sample_count,
        average_score,
        average_accuracy,
        latest_captured_at,
        by_sample,
    }))
}

async fn analytics_pdf(State(state): State<Arc<AppState>>) -> Result<Response, ApiError> {
    fn get_sample_name(id: i32) -> String {
        match id {
            1 => "Arabica Gayo".to_string(),
            2 => "Arabica Toraja".to_string(),
            3 => "Arabica Jawa Tengah".to_string(),
            4 => "Arabica Bali".to_string(),
            5 => "Arabica Flores".to_string(),
            6 => "Arabica Wamena".to_string(),
            7 => "Robusta Lampung".to_string(),
            8 => "Robusta Bengkulu".to_string(),
            9 => "Robusta Jawa Timur".to_string(),
            10 => "Low Grade A".to_string(),
            11 => "Low Grade B".to_string(),
            12 => "Low Grade C".to_string(),
            13 => "Medium Blend".to_string(),
            14 => "Premium Blend".to_string(),
            15 => "Arabica Gayo (Val)".to_string(),
            16 => "Arabica Toraja (Val)".to_string(),
            17 => "Arabica Jawa (Val)".to_string(),
            18 => "Arabica Bali (Val)".to_string(),
            _ => format!("Sample {:02}", id),
        }
    }
    
    let summary = summary(State(state)).await?.0;
    let (document, page, layer) = PdfDocument::new(
        "E-Nose Analytical Report",
        Mm(210.0),
        Mm(297.0),
        "Analytical Report",
    );
    let regular = document.add_builtin_font(BuiltinFont::Helvetica)?;
    let bold = document.add_builtin_font(BuiltinFont::HelveticaBold)?;
    let current_layer = document.get_page(page).get_layer(layer);
    let navy = Color::Rgb(Rgb::new(0.05, 0.19, 0.23, None));
    let teal = Color::Rgb(Rgb::new(0.15, 0.49, 0.51, None));
    let pale = Color::Rgb(Rgb::new(0.91, 0.96, 0.96, None));
    let line = Color::Rgb(Rgb::new(0.73, 0.82, 0.83, None));
    let ink = Color::Rgb(Rgb::new(0.10, 0.16, 0.18, None));
    let muted = Color::Rgb(Rgb::new(0.35, 0.43, 0.44, None));
    let white = Color::Rgb(Rgb::new(1.0, 1.0, 1.0, None));

    draw_rect(&current_layer, 0.0, 250.0, 210.0, 47.0, &navy);
    draw_rect(&current_layer, 0.0, 245.0, 210.0, 5.0, &teal);
    text(&current_layer, "E-NOSE", 11.0, 20.0, 284.0, &bold, &teal);
    text(
        &current_layer,
        "Analytical Report",
        24.0,
        20.0,
        270.0,
        &bold,
        &white,
    );
    text(
        &current_layer,
        "Coffee aroma detection and cloud data analysis",
        10.0,
        20.0,
        258.0,
        &regular,
        &pale,
    );
    text(
        &current_layer,
        "REPORT A-001",
        9.0,
        166.0,
        283.0,
        &bold,
        &white,
    );
    text(
        &current_layer,
        &format!("{}", Utc::now().format("%d-%b-%Y")),
        8.0,
        166.0,
        276.0,
        &regular,
        &pale,
    );
    text(
        &current_layer,
        &format!("{}", Utc::now().format("%H:%M UTC")),
        7.5,
        166.0,
        269.0,
        &regular,
        &pale,
    );

    section_title(
        &current_layer,
        "EXECUTIVE SUMMARY",
        20.0,
        233.0,
        170.0,
        &teal,
        &bold,
        &white,
    );
    let cards = [
        ("TOTAL MEASUREMENTS", summary.total_measurements.to_string()),
        ("SAMPLES DETECTED", format!("{} / 18", summary.sample_count)),
        ("AVERAGE SCORE", format!("{:.2}", summary.average_score)),
        (
            "AVERAGE ACCURACY",
            format!("{:.2}%", summary.average_accuracy),
        ),
    ];
    for (index, (label, value)) in cards.iter().enumerate() {
        let x = 20.0 + index as f64 * 43.0;
        draw_rect(&current_layer, x, 207.0, 39.0, 21.0, &pale);
        draw_rect(&current_layer, x, 226.0, 39.0, 2.0, &teal);
        text(&current_layer, label, 7.0, x + 3.0, 220.0, &bold, &muted);
        text(&current_layer, value, 16.0, x + 3.0, 211.0, &bold, &ink);
    }

    section_title(
        &current_layer,
        "SAMPLE PERFORMANCE",
        20.0,
        197.0,
        170.0,
        &teal,
        &bold,
        &white,
    );
    draw_rect(&current_layer, 20.0, 181.0, 170.0, 14.0, &pale);
    text(&current_layer, "SAMPLE", 8.0, 24.0, 187.0, &bold, &muted);
    text(
        &current_layer,
        "MEASUREMENTS",
        8.0,
        53.0,
        187.0,
        &bold,
        &muted,
    );
    text(&current_layer, "SCORE", 8.0, 102.0, 187.0, &bold, &muted);
    text(&current_layer, "ACCURACY", 8.0, 137.0, 187.0, &bold, &muted);
    text(&current_layer, "STATUS", 8.0, 169.0, 187.0, &bold, &muted);

    let mut y = 175.0;
    // Limit to latest 15 samples
    let samples_to_show: Vec<_> = summary.by_sample.iter().rev().take(15).collect();
    for sample in samples_to_show {
        if sample.sample_id % 2 == 0 {
            draw_rect(
                &current_layer,
                20.0,
                y - 2.0,
                170.0,
                8.0,
                Color::Rgb(Rgb::new(0.97, 0.98, 0.98, None)),
            );
        }
        text(
            &current_layer,
            &get_sample_name(sample.sample_id),
            8.5,
            24.0,
            y,
            &bold,
            &ink,
        );
        text(
            &current_layer,
            &sample.measurement_count.to_string(),
            8.5,
            59.0,
            y,
            &regular,
            &ink,
        );
        text(
            &current_layer,
            &format!("{:.2}", sample.average_score),
            8.5,
            105.0,
            y,
            &regular,
            &ink,
        );
        text(
            &current_layer,
            &format!("{:.2}%", sample.average_accuracy),
            8.5,
            140.0,
            y,
            &regular,
            &ink,
        );
        let status = if sample.average_accuracy >= 85.0 {
            "REVIEWED"
        } else {
            "MONITOR"
        };
        text(
            &current_layer,
            status,
            7.5,
            169.0,
            y,
            &bold,
            if status == "REVIEWED" { &teal } else { &muted },
        );
        draw_rule(&current_layer, 20.0, y - 3.0, 190.0, &line);
        y -= 8.0;
    }

    // Calculate where the table ends (minimum Y position after all samples)
    let table_end_y = y.min(100.0); // Ensure at least 100mm from top
    
    let high_count = summary
        .by_sample
        .iter()
        .filter(|sample| sample.sample_id <= 6)
        .count();
    let low_count = summary.by_sample.len().saturating_sub(high_count);
    
    // Position DATASET COVERAGE and REPORT NOTES below the table
    let coverage_y = table_end_y - 15.0;
    let boxes_y = table_end_y - 31.0;
    
    section_title(
        &current_layer,
        "DATASET COVERAGE",
        20.0,
        coverage_y,
        82.0,
        &teal,
        &bold,
        &white,
    );
    section_title(
        &current_layer,
        "REPORT NOTES",
        108.0,
        coverage_y,
        82.0,
        &teal,
        &bold,
        &white,
    );
    draw_rect(&current_layer, 20.0, boxes_y, 82.0, 21.0, &pale);
    draw_rect(&current_layer, 108.0, boxes_y, 82.0, 21.0, &pale);
    text(
        &current_layer,
        &format!("High grade: {} samples", high_count),
        9.0,
        24.0,
        boxes_y + 14.0,
        &regular,
        &ink,
    );
    text(
        &current_layer,
        &format!("Low grade: {} samples", low_count),
        9.0,
        24.0,
        boxes_y + 6.0,
        &regular,
        &ink,
    );
    text(
        &current_layer,
        "Dashboard-ready aggregate",
        8.0,
        112.0,
        boxes_y + 14.0,
        &regular,
        &ink,
    );
    text(
        &current_layer,
        "Feature vectors: 8 sensor channels",
        8.0,
        112.0,
        boxes_y + 6.0,
        &regular,
        &ink,
    );
    text(
        &current_layer,
        "Demo dataset score/accuracy values are analytical placeholders until Edge Impulse predictions are connected.",
        7.0,
        20.0,
        17.0,
        &regular,
        &muted,
    );
    let mut bytes = Vec::new();
    document.save(&mut BufWriter::new(std::io::Cursor::new(&mut bytes)))?;
    let mut response = bytes.into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=analytical-report.pdf"),
    );
    Ok(response)
}

fn text<C: Borrow<Color>>(
    layer: &PdfLayerReference,
    value: &str,
    size: f64,
    x: f64,
    y: f64,
    font: &printpdf::IndirectFontRef,
    color: C,
) {
    layer.set_fill_color(color.borrow().clone());
    layer.use_text(value, size as f32, Mm(x as f32), Mm(y as f32), font);
}

fn draw_rect<C: Borrow<Color>>(
    layer: &PdfLayerReference,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    color: C,
) {
    let polygon = Polygon {
        rings: vec![vec![
            (Point::new(Mm(x as f32), Mm(y as f32)), false),
            (Point::new(Mm(x as f32), Mm((y + height) as f32)), false),
            (
                Point::new(Mm((x + width) as f32), Mm((y + height) as f32)),
                false,
            ),
            (Point::new(Mm((x + width) as f32), Mm(y as f32)), false),
        ]],
        mode: PaintMode::Fill,
        winding_order: WindingOrder::NonZero,
    };
    layer.set_fill_color(color.borrow().clone());
    layer.add_polygon(polygon);
}

fn draw_rule<C: Borrow<Color>>(layer: &PdfLayerReference, x1: f64, y: f64, x2: f64, color: C) {
    let polygon = Polygon {
        rings: vec![vec![
            (Point::new(Mm(x1 as f32), Mm(y as f32)), false),
            (Point::new(Mm(x2 as f32), Mm(y as f32)), false),
            (Point::new(Mm(x2 as f32), Mm((y + 0.2) as f32)), false),
            (Point::new(Mm(x1 as f32), Mm((y + 0.2) as f32)), false),
        ]],
        mode: PaintMode::Fill,
        winding_order: WindingOrder::NonZero,
    };
    layer.set_fill_color(color.borrow().clone());
    layer.add_polygon(polygon);
}

fn section_title<C1: Borrow<Color>, C2: Borrow<Color>>(
    layer: &PdfLayerReference,
    title: &str,
    x: f64,
    y: f64,
    width: f64,
    color: C1,
    font: &printpdf::IndirectFontRef,
    text_color: C2,
) {
    draw_rect(layer, x, y, width, 8.0, color);
    text(layer, title, 9.5, x + 3.0, y + 2.3, font, text_color);
}

fn validate(payload: &CreateMeasurement) -> Result<(), ApiError> {
    if !(1..=18).contains(&payload.sample_id)
        || !(0.0..=100.0).contains(&payload.score)
        || !(0.0..=100.0).contains(&payload.accuracy)
    {
        return Err(ApiError::BadRequest(
            "sample_id must be 1..18 and score/accuracy must be 0..100".into(),
        ));
    }
    Ok(())
}

async fn mqtt_worker(state: AppState) {
    loop {
        info!("MQTT: Initializing connection...");
        
        let broker = env("MQTT_BROKER", "localhost");
        let port = env("MQTT_PORT", "1883").parse().unwrap_or(1883);
        let username = env::var("MQTT_USERNAME").ok();
        let password = env::var("MQTT_PASSWORD").ok();
        let topic = env("MQTT_TOPIC", "enose/+/measurement");
        let _tls_enabled = env("MQTT_TLS", "false").eq_ignore_ascii_case("true");
        
        let mut options = MqttOptions::new(
            env("MQTT_CLIENT_ID", "enose-cloud"),
            broker.clone(),
            port,
        );
        options.set_keep_alive(Duration::from_secs(30));
        
        // Set credentials if provided
        if let (Some(user), Some(pass)) = (username.as_ref(), password.as_ref()) {
            options.set_credentials(user, pass);
            info!("MQTT: Using authentication");
        }
        
        // TODO: TLS support can be added here when needed
        // if tls_enabled {
        //     use rumqttc::TlsConfiguration;
        //     let tls_config = TlsConfiguration::Simple { ... };
        //     options.set_tls_configuration(tls_config);
        // }
        
        let (client, mut event_loop) = AsyncClient::new(options, 10);
        
        // Subscribe to topic
        match client.subscribe(&topic, QoS::AtLeastOnce).await {
            Ok(_) => {
                info!(%topic, %broker, %port, "MQTT: Connected and subscribed");
            }
            Err(error) => {
                error!(%error, "MQTT: Subscribe failed, retrying in 10s");
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }
        }
        
        // Event loop
        loop {
            match event_loop.poll().await {
                Ok(Event::Incoming(Incoming::Publish(message))) => {
                    info!(
                        topic = %message.topic,
                        payload_len = message.payload.len(),
                        "MQTT: Message received"
                    );
                    
                    match serde_json::from_slice::<CreateMeasurement>(&message.payload) {
                        Ok(mut payload) => {
                            payload.source = Some("mqtt".to_string());
                            
                            if let Err(e) = validate(&payload) {
                                warn!(?e, "MQTT: Payload validation failed");
                                continue;
                            }
                            
                            let state_arc = Arc::new(state.clone());
                            match create_measurement(State(state_arc), Json(payload)).await {
                                Ok((status, json)) => {
                                    info!(
                                        measurement_id = %json.id,
                                        sample_id = json.sample_id,
                                        device_id = ?json.device_id,
                                        "MQTT: Measurement saved ({})",
                                        status.as_u16()
                                    );
                                }
                                Err(error) => {
                                    warn!(?error, "MQTT: Database insert failed");
                                }
                            }
                        }
                        Err(error) => {
                            warn!(
                                %error,
                                payload = ?String::from_utf8_lossy(&message.payload),
                                "MQTT: Invalid JSON payload"
                            );
                        }
                    }
                }
                Ok(Event::Incoming(Incoming::ConnAck(_))) => {
                    info!("MQTT: Connection acknowledged");
                }
                Ok(Event::Incoming(Incoming::SubAck(_))) => {
                    info!("MQTT: Subscription acknowledged");
                }
                Err(error) => {
                    error!(%error, "MQTT: Connection error, reconnecting in 10s");
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    break; // Break inner loop to reconnect
                }
                _ => {}
            }
        }
        
        // Reconnect delay before outer loop retries
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

fn env(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

fn is_postgres() -> bool {
    env::var("DATABASE_URL")
        .map(|url| url.starts_with("postgres://") || url.starts_with("postgresql://"))
        .unwrap_or(false)
}

#[derive(Debug)]
enum ApiError {
    BadRequest(String),
    NotFound,
    Internal(anyhow::Error),
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        Self::Internal(error.into())
    }
}
impl From<serde_json::Error> for ApiError {
    fn from(error: serde_json::Error) -> Self {
        Self::Internal(error.into())
    }
}
impl From<printpdf::Error> for ApiError {
    fn from(error: printpdf::Error) -> Self {
        Self::Internal(error.into())
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::BadRequest(message) => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": message})),
            )
                .into_response(),
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "not found"})),
            )
                .into_response(),
            Self::Internal(error) => {
                error!(%error, "request failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "internal server error"})),
                )
                    .into_response()
            }
        }
    }
}
