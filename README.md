# E-Nose IoT System - Group A2-C12

<div align="center">

### Electronic Nose System for Coffee Quality Classification
*Real-time sensor data acquisition, Edge Impulse ML inference, cloud backend, and monitoring dashboard*

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-15+-blue.svg)](https://www.postgresql.org/)
[![MQTT](https://img.shields.io/badge/MQTT-5.0-purple.svg)](https://mqtt.org/)
[![ESP32](https://img.shields.io/badge/ESP32--S3-no--std-green.svg)](https://www.espressif.com/)

</div>

---

## 👥 Team Members

### Kelas A (Group 2)
1. **Natasya Putri Regina** - `2042241026`
2. **Ghani Raihan Syakir** - `2042241036`

### Kelas C (Group 12)
3. **Evan Javier Firdausi Malik** - `2042241010`
4. **Mahendra Dwi Fahreza** - `2042241023`

---

## 📋 Project Overview

This **E-Nose (Electronic Nose)** system is designed to classify coffee quality using multiple gas sensors and machine learning. 

### 🎯 Key Components

| Component | Technology | Description |
|-----------|------------|-------------|
| **Edge Device** | ESP32-S3 (no-std) | Sensor data collection + ML inference |
| **ML Framework** | Edge Impulse | On-device coffee quality classification |
| **Cloud Backend** | Rust + Actix-web | REST API & MQTT ingestion server |
| **Database** | PostgreSQL | Persistent data storage |
| **Dashboard** | HTML/JS/Chart.js | Real-time monitoring & analytics |
| **Communication** | MQTT | Low-latency sensor data streaming |
| **Deployment** | Railway + Docker | Cloud-hosted production environment |

### ✨ Features

- 🔬 **Multi-sensor Integration** - DHT22 (temp/humidity) + 8× MQ gas sensors
- 🧠 **Edge ML Inference** - Real-time classification on ESP32
- ☁️ **Cloud Data Pipeline** - MQTT → Backend → Database → Dashboard
- 📊 **Live Analytics** - Real-time charts and historical trends
- 📄 **PDF Reports** - Downloadable analytical summaries
- 🔄 **OTA Updates** - Remote ESP32 firmware deployment

---

## 🏗️ System Architecture

```mermaid
graph TD
    A[ESP32-S3 Device] -->|WiFi/MQTT| B[MQTT Broker<br/>broker.emqx.io]
    B -->|Subscribe| C[Rust Backend<br/>Railway]
    C -->|Store| D[(PostgreSQL<br/>Database)]
    C -->|REST API| E[Web Dashboard]
    
    F[DHT22 Sensor] -->|I2C| A
    G[8× MQ Sensors] -->|ADC| A
    H[Edge Impulse Model] -->|Inference| A
    
    E -->|Analytics| I[📊 Charts]
    E -->|Export| J[📄 PDF Report]
    E -->|Upload| K[🔄 OTA Firmware]
    
    style A fill:#4CAF50
    style C fill:#FF9800
    style D fill:#2196F3
    style E fill:#9C27B0
```

### Data Flow Pipeline

```
Sensors → ESP32 Inference → MQTT Publish → Backend Ingestion → PostgreSQL Storage → Dashboard Visualization
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

### 📦 Prerequisites

```bash
# Required
✓ Rust 1.70+ and Cargo
✓ PostgreSQL 15+ (production) or SQLite (development)
✓ MQTT broker access (default: broker.emqx.io)

# Optional
○ Docker (for containerized deployment)
○ ESP32-S3 with USB-C cable (for firmware development)
```

### 🛠️ Installation

**1. Clone the repository**
```bash
git clone https://github.com/mahendrafhrz/group-a2-c12-iot.git
cd group-a2-c12-iot/enose-backend-12c
```

**2. Configure environment**
```bash
cp .env.example .env
# Edit .env with your database and MQTT settings
```

**3. Run the backend**
```bash
cargo run --release
```

**4. Access the dashboard**
```
🌐 http://localhost:8080/dashboard
```

---

## 📡 API Reference

### Endpoints

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| `GET` | `/health` | Health check status | - |
| `GET` | `/dashboard` | Web dashboard UI | - |
| `GET` | `/api/v1/measurements` | Retrieve all measurements | - |
| `POST` | `/api/v1/measurements` | Submit new measurement data | - |
| `GET` | `/api/v1/analytics/summary` | Get aggregated analytics | - |
| `GET` | `/api/v1/reports/analytics.pdf` | Download PDF report | - |

### 📤 Example: Submit Measurement

**Request**
```bash
curl -X POST http://localhost:8080/api/v1/measurements \
  -H "Content-Type: application/json" \
  -d '{
    "device_id": "esp32s3-001",
    "temperature": 28.5,
    "humidity": 65.2,
    "confidence": 0.89,
    "predicted_class": "high_grade",
    "sensor_data": [0.45, 0.32, 0.78, 0.91, 0.56, 0.67, 0.43, 0.88]
  }'
```

**Response**
```json
{
  "status": "success",
  "id": 1234,
  "timestamp": "2026-09-28T13:45:30Z"
}
```

---

## 🔌 MQTT Configuration

### Connection Settings

| Parameter | Value | Description |
|-----------|-------|-------------|
| **Broker** | `broker.emqx.io` | Public MQTT broker |
| **Port** | `1883` | TCP (no TLS) |
| **Protocol** | MQTT 3.1.1 | Standard MQTT |
| **Topic** | `enose/measurements` | Configurable via env |
| **QoS** | `1` | At least once delivery |
| **Authentication** | None | Public broker (no credentials) |

### 📨 Message Format

**Topic:** `enose/measurements`

**Payload (JSON):**
```json
{
  "device_id": "esp32s3-001",
  "temperature": 28.5,
  "humidity": 65.2,
  "confidence": 0.89,
  "predicted_class": "high_grade",
  "sensor_data": [0.45, 0.32, 0.78, 0.91, 0.56, 0.67, 0.43, 0.88],
  "timestamp": "2026-09-28T13:45:30Z"
}
```

> **Note:** ESP32 publishes to this topic after each inference cycle. Backend subscribes and ingests automatically.

---

## 🌐 Cloud Deployment

### Railway (Production)

**🔗 Live Dashboard:** [`https://enose-cloud-backend-production-facd.up.railway.app/dashboard`](https://enose-cloud-backend-production-facd.up.railway.app/dashboard)

#### Environment Variables

```env
# Database
DATABASE_URL=postgres://user:password@host:5432/enose

# MQTT Settings
MQTT_ENABLED=true
MQTT_BROKER=broker.emqx.io
MQTT_PORT=1883
MQTT_TOPIC=enose/measurements
MQTT_USERNAME=
MQTT_PASSWORD=
MQTT_TLS=false

# Server
HOST=0.0.0.0
PORT=8080
```

### 🐳 Docker Deployment

**Build image:**
```bash
docker build -t enose-backend:latest .
```

**Run container:**
```bash
docker run -d \
  --name enose-backend \
  -p 8080:8080 \
  -e DATABASE_URL="postgres://user:pass@host:5432/enose" \
  -e MQTT_ENABLED=true \
  -e MQTT_BROKER=broker.emqx.io \
  enose-backend:latest
```

**Docker Compose:**
```yaml
version: '3.8'
services:
  backend:
    build: .
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: postgres://user:pass@db:5432/enose
      MQTT_ENABLED: "true"
      MQTT_BROKER: broker.emqx.io
    depends_on:
      - db
  
  db:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: enose
      POSTGRES_USER: user
      POSTGRES_PASSWORD: pass
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

---

## 📊 Dashboard Features

<div align="center">

| Feature | Description | Status |
|---------|-------------|--------|
| 📈 **Real-time Charts** | Live sensor data visualization | ✅ Active |
| 🎯 **Classification Results** | ML confidence & predicted class | ✅ Active |
| 📉 **Historical Trends** | Time-series analysis | ✅ Active |
| 📊 **Analytics Summary** | Accuracy metrics & statistics | ✅ Active |
| 📄 **PDF Export** | Downloadable reports | ✅ Active |
| 🔄 **OTA Update UI** | Firmware binary upload | 🚧 Planned |

</div>

### Dashboard Sections

1. **Live Monitoring**
   - Real-time temperature & humidity
   - 8-channel gas sensor readings
   - Classification confidence scores

2. **Analytics Panel**
   - Total samples processed
   - Classification accuracy
   - High/Low grade distribution
   - Confidence trends over time

3. **Data Management**
   - Filter by date range
   - Search by device ID
   - Export to CSV/PDF
   - Historical data viewer

---

## 🔧 ESP32 Firmware

### Hardware Configuration

| Component | Model | Interface | Purpose |
|-----------|-------|-----------|---------|
| **MCU** | ESP32-S3 | - | Main controller |
| **Temp/Humidity** | DHT22 | I2C | Environmental sensing |
| **Gas Sensors** | 8× MQ Series | ADC | Coffee aroma detection |
| **WiFi** | Built-in | 802.11 b/g/n | MQTT communication |

### Firmware Features

- ⚡ **no-std Environment** - Bare-metal Rust for optimal performance
- 🧠 **Edge Impulse Integration** - On-device ML inference
- 📡 **MQTT Client** - Real-time data streaming
- 🔄 **OTA Support** - Remote firmware updates
- 💾 **Local Buffering** - Handles network interruptions

> **Note:** ESP32 firmware source code is maintained in separate `lancar-esp32-no-std` folder (not included in this repository).

---

## 📈 Data Flow

### Complete Pipeline

```
┌─────────────────────────────────────────────────────────────────┐
│                         ESP32-S3 Device                         │
│  ┌──────────┐  ┌──────────┐  ┌─────────────┐  ┌──────────┐   │
│  │ Sensors  │→│  ADC/I2C │→│ Edge Impulse│→│   WiFi   │   │
│  │ DHT22+MQ │  │  Reading │  │    Model    │  │   MQTT   │   │
│  └──────────┘  └──────────┘  └─────────────┘  └──────────┘   │
└─────────────────────────────────┬───────────────────────────────┘
                                  │ JSON Payload
                                  ↓
                    ┌─────────────────────────┐
                    │    MQTT Broker          │
                    │   broker.emqx.io:1883   │
                    └───────────┬─────────────┘
                                │ Subscribe
                                ↓
┌─────────────────────────────────────────────────────────────────┐
│                     Rust Backend (Railway)                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐      │
│  │   MQTT   │→│ Validator│→│ Database │→│  REST API│      │
│  │  Worker  │  │  & Parser│  │  Insert  │  │  Handler │      │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘      │
└─────────────────────────────────┬───────────────────────────────┘
                                  │ PostgreSQL
                                  ↓
                        ┌────────────────────┐
                        │   PostgreSQL DB    │
                        │   measurements     │
                        └──────────┬─────────┘
                                   │ Query
                                   ↓
┌─────────────────────────────────────────────────────────────────┐
│                      Web Dashboard                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐      │
│  │  Fetch   │→│  Chart   │  │ Analytics│  │   PDF    │      │
│  │   Data   │  │  Update  │  │  Summary │  │  Export  │      │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘      │
└─────────────────────────────────────────────────────────────────┘
```

### Sequence Diagram

```mermaid
sequenceDiagram
    participant ESP as ESP32-S3
    participant Broker as MQTT Broker
    participant Backend as Rust Backend
    participant DB as PostgreSQL
    participant Dashboard as Web Dashboard
    
    ESP->>ESP: Read sensors
    ESP->>ESP: ML Inference
    ESP->>Broker: Publish (JSON)
    Broker->>Backend: Forward message
    Backend->>Backend: Validate & parse
    Backend->>DB: INSERT measurement
    DB-->>Backend: Confirm
    Dashboard->>Backend: GET /api/v1/measurements
    Backend->>DB: SELECT latest
    DB-->>Backend: Return rows
    Backend-->>Dashboard: JSON response
    Dashboard->>Dashboard: Update charts
```

---

## 🛠️ Development

### Database Schema

```sql
CREATE TABLE measurements (
    id              SERIAL PRIMARY KEY,
    device_id       VARCHAR(50) NOT NULL,
    temperature     REAL,
    humidity        REAL,
    confidence      REAL,
    predicted_class VARCHAR(50),
    sensor_data     JSONB,
    timestamp       TIMESTAMPTZ DEFAULT NOW(),
    
    INDEX idx_device_timestamp (device_id, timestamp),
    INDEX idx_predicted_class (predicted_class)
);
```

### Build Commands

```bash
# 🏃 Run development server
cargo run

# 🔨 Build for production
cargo build --release

# 🧪 Run tests
cargo test

# 📋 Format code
cargo fmt

# 🔍 Lint with Clippy
cargo clippy

# 📦 Check dependencies
cargo tree

# 🧹 Clean build artifacts
cargo clean
```

### Project Structure

```
enose-backend-12c/
├── src/
│   ├── main.rs              # Entry point, server setup
│   ├── api/                 # REST API handlers
│   │   ├── measurements.rs  # Measurement endpoints
│   │   ├── analytics.rs     # Analytics endpoints
│   │   └── reports.rs       # PDF generation
│   ├── mqtt/                # MQTT client & worker
│   │   ├── client.rs        # Connection handling
│   │   └── handler.rs       # Message processing
│   ├── db/                  # Database layer
│   │   ├── models.rs        # Data structures
│   │   └── queries.rs       # SQL operations
│   └── utils/               # Utilities
│       ├── config.rs        # Environment configuration
│       └── logger.rs        # Logging setup
├── dashboard.html           # Web dashboard
├── Cargo.toml              # Dependencies
├── Dockerfile              # Container image
├── .env.example            # Environment template
└── README.md               # This file
```

---

## 🤝 Contributing

### For Team Members

1. **Create feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make changes and test**
   ```bash
   cargo test
   cargo fmt
   cargo clippy
   ```

3. **Commit with descriptive messages**
   ```bash
   git add .
   git commit -m "feat: add sensor calibration endpoint"
   ```

4. **Push and create Pull Request**
   ```bash
   git push origin feature/your-feature-name
   ```

5. **Request review** from at least one team member

### Commit Message Convention

```
feat: new feature
fix: bug fix
docs: documentation update
style: formatting changes
refactor: code restructuring
test: add/update tests
chore: maintenance tasks
```

---

## � Documentation

- **API Documentation:** `/api/docs` (Swagger UI)
- **Architecture Diagram:** See [System Architecture](#-system-architecture)
- **Deployment Guide:** See [Cloud Deployment](#-cloud-deployment)
- **ESP32 Setup:** Refer to `lancar-esp32-no-std` folder
- **Edge Impulse Project:** [Contact team for access]

---

## 🐛 Troubleshooting

<details>
<summary><b>MQTT connection refused</b></summary>

**Problem:** Backend cannot connect to MQTT broker

**Solutions:**
- Check `MQTT_BROKER` and `MQTT_PORT` in `.env`
- Verify broker is accessible: `telnet broker.emqx.io 1883`
- Ensure firewall allows outbound MQTT connections
- Try alternative public broker (e.g., `test.mosquitto.org`)
</details>

<details>
<summary><b>Database connection error</b></summary>

**Problem:** `DATABASE_URL` invalid or unreachable

**Solutions:**
- Verify PostgreSQL is running: `pg_isready`
- Check connection string format
- Ensure database `enose` exists
- For Railway: check environment variables in dashboard
</details>

<details>
<summary><b>ESP32 not sending data</b></summary>

**Problem:** No measurements appearing in dashboard

**Solutions:**
- Check ESP32 serial output for errors
- Verify WiFi credentials are correct
- Confirm MQTT topic matches backend subscription
- Check ESP32 is powered and sensors are connected
- Review firmware logs for connection issues
</details>

---

## 📞 Contact

For questions, issues, or collaboration:

- **Project Repository:** [github.com/mahendrafhrz/group-a2-c12-iot](https://github.com/mahendrafhrz/group-a2-c12-iot)
- **Team Communication:** University email or group channels
- **Issues:** Use GitHub Issues for bug reports and feature requests

---

## 📄 License

This project is developed for **academic purposes** as part of IoT coursework at Politeknik Elektronika Negeri Surabaya (PENS).

**Course:** Internet of Things  
**Academic Year:** 2026  
**Semester:** [Specify semester]

---

<div align="center">

**Made with ❤️ by Group A2-C12**

[![Rust](https://img.shields.io/badge/Built%20with-Rust-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![ESP32](https://img.shields.io/badge/Powered%20by-ESP32--S3-green?style=for-the-badge&logo=espressif)](https://www.espressif.com/)
[![MQTT](https://img.shields.io/badge/Protocol-MQTT-purple?style=for-the-badge&logo=mqtt)](https://mqtt.org/)

**⭐ Star this repo if you find it useful!**

</div>

---

**Last Updated:** September 28, 2026  
**Version:** 1.0.0
