# E-Nose IoT System - Group A2-C12

> **Electronic Nose System for Coffee Quality Classification**  
> Real-time sensor data acquisition, Edge Impulse ML inference, cloud backend, and monitoring dashboard

---

## 👥 Team Members

**Kelas A (Group 2)**
1. **Natasya Putri Regina** - 2042241026
2. **Ghani Raihan Syakir** - 2042241036

**Kelas C (Group 12)**
3. **Evan Javier Firdausi Malik** - 2042241010
4. **Mahendra Dwi Fahreza** - 2042241023

---

## 📋 Project Overview

This E-Nose (Electronic Nose) system is designed to classify coffee quality using multiple gas sensors and machine learning. The system consists of:

- **ESP32-S3 Edge Device** - Sensor data collection with on-device ML inference using Edge Impulse
- **Cloud Backend** - Rust-based API server with PostgreSQL database and MQTT support
- **Web Dashboard** - Real-time monitoring and analytics visualization
- **OTA Firmware Update** - Remote firmware update capability for continuous model improvement

---

## 🏗️ System Architecture

```
┌─────────────────┐
│   ESP32-S3      │  ← DHT22 + 8x MQ Sensors
│   (no-std)      │  ← Edge Impulse ML Model
└────────┬────────┘
         │ MQTT (broker.emqx.io:1883)
         ↓
┌─────────────────┐
│  Cloud Backend  │  ← Rust + Actix-web
│  (Railway)      │  ← PostgreSQL Database
└────────┬────────┘
         │ REST API
         ↓
┌─────────────────┐
│  Web Dashboard  │  ← Real-time Charts
│  (dashboard.html)│  ← Analytics & Reports
└─────────────────┘
```

---

## 🗂️ Repository Structure

```
group-a2-c12-iot/
└── enose-backend-12c/          # Cloud Backend & Dashboard
    ├── src/                    # Rust source code
    │   ├── main.rs            # API server & MQTT worker
    │   ├── db.rs              # Database operations
    │   ├── models.rs          # Data structures
    │   └── ...
    ├── dashboard.html          # Web monitoring interface
    ├── Cargo.toml              # Rust dependencies
    ├── Dockerfile              # Container deployment
    ├── enose.db                # SQLite database (dev)
    ├── CREDENTIALS.txt         # Deployment credentials
    └── Raw Data Kopi Rafi/     # Training dataset
        ├── high_grade/         # High quality samples
        └── low_grade/          # Low quality samples
```

---

## 🚀 Quick Start

### Prerequisites
- Rust 1.70+ and Cargo
- PostgreSQL (production) or SQLite (development)
- MQTT broker access (default: `broker.emqx.io`)

### Installation

1. **Clone repository**
   ```bash
   git clone https://github.com/mahendrafhrz/group-a2-c12-iot.git
   cd group-a2-c12-iot/enose-backend-12c
   ```

2. **Setup environment**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

3. **Run backend**
   ```bash
   cargo run
   ```

4. **Access dashboard**
   ```
   http://localhost:8080/dashboard
   ```

---

## 📡 API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Health check |
| `GET` | `/dashboard` | Web dashboard interface |
| `GET` | `/api/v1/measurements` | Get all sensor measurements |
| `POST` | `/api/v1/measurements` | Submit new measurement |
| `GET` | `/api/v1/analytics/summary` | Get analytics summary |
| `GET` | `/api/v1/reports/analytics.pdf` | Download PDF report |

### Example: Submit Measurement

```bash
curl -X POST http://localhost:8080/api/v1/measurements \
  -H "Content-Type: application/json" \
  -d '{
    "device_id": "esp32s3-001",
    "temperature": 28.5,
    "humidity": 65.2,
    "confidence": 0.89,
    "predicted_class": "high_grade",
    "sensor_data": [0.45, 0.32, 0.78, 0.91, ...]
  }'
```

---

## 🔌 MQTT Configuration

**Broker:** `broker.emqx.io`  
**Port:** `1883` (TCP, no TLS)  
**Topic:** `enose/measurements` (configurable via `MQTT_TOPIC`)

**Authentication:** None (public broker)

### MQTT Message Format

```json
{
  "device_id": "esp32s3-001",
  "temperature": 28.5,
  "humidity": 65.2,
  "confidence": 0.89,
  "predicted_class": "high_grade",
  "sensor_data": [0.45, 0.32, 0.78, 0.91, 0.56, 0.67, 0.43, 0.88]
}
```

---

## 🌐 Cloud Deployment

### Railway Deployment

**Live URL:** `https://enose-cloud-backend-production-facd.up.railway.app/dashboard`

**Environment Variables:**
```env
DATABASE_URL=postgres://user:pass@host:5432/enose
MQTT_ENABLED=true
MQTT_BROKER=broker.emqx.io
MQTT_PORT=1883
MQTT_TOPIC=enose/measurements
HOST=0.0.0.0
PORT=8080
```

### Docker Deployment

```bash
docker build -t enose-backend .
docker run -p 8080:8080 \
  -e DATABASE_URL=postgres://... \
  -e MQTT_ENABLED=true \
  enose-backend
```

---

## 📊 Dashboard Features

- **Real-time Monitoring** - Live sensor data visualization
- **Classification Results** - ML prediction confidence and class detection
- **Historical Data** - Time-series charts and trends
- **Analytics Summary** - Accuracy metrics and sample statistics
- **PDF Export** - Downloadable analytical reports
- **OTA Update Interface** - Firmware binary upload for ESP32 updates

---

## 🔧 ESP32 Firmware (Separate Repository)

The ESP32-S3 firmware runs in **no-std** environment with:
- **Sensors:** DHT22 (temperature/humidity) + 8x MQ gas sensors
- **ML Inference:** Edge Impulse model (on-device)
- **Communication:** MQTT over WiFi
- **OTA Support:** Remote firmware updates

**Note:** ESP32 firmware is maintained in `lancar-esp32-no-std` folder (not in this repository).

---

## 📈 Data Flow

1. **Sensor Reading** → ESP32 collects data from DHT22 and MQ sensors
2. **ML Inference** → Edge Impulse model classifies coffee quality on-device
3. **MQTT Publish** → Results sent to cloud broker
4. **Backend Ingestion** → Rust backend receives and stores data in PostgreSQL
5. **Dashboard Display** → Web interface shows real-time updates and analytics

---

## 🛠️ Development

### Database Schema

```sql
CREATE TABLE measurements (
    id SERIAL PRIMARY KEY,
    device_id VARCHAR(50) NOT NULL,
    temperature REAL,
    humidity REAL,
    confidence REAL,
    predicted_class VARCHAR(50),
    sensor_data JSONB,
    timestamp TIMESTAMPTZ DEFAULT NOW()
);
```

### Build Commands

```bash
# Development
cargo run

# Production build
cargo build --release

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

---

## 📝 License

This project is developed for academic purposes as part of IoT coursework.

---

## 🤝 Contributing

For team members:
1. Create feature branch from `main`
2. Make changes and test locally
3. Commit with descriptive messages
4. Push and create Pull Request
5. Request review from team members

---

## 📞 Contact

For questions or issues, contact team members via university email or project communication channels.

---

**Last Updated:** September 28, 2026  
**Repository:** https://github.com/mahendrafhrz/group-a2-c12-iot
