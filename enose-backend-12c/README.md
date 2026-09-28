# E-Nose Cloud Backend

Bagian B untuk project E-Nose: REST API, MQTT ingestion, penyimpanan measurement, analytical summary, dan download analytical report PDF.

## Menjalankan

```powershell
Copy-Item .env.example .env
cargo run
```

API berjalan di `http://localhost:8080`.

- `GET /dashboard` - dashboard analytical berbasis web
- `GET /health`
- `GET /api/v1/analytics/summary`
- `GET /api/v1/measurements`
- `POST /api/v1/measurements`
- `GET /api/v1/reports/analytics.pdf`

## Mengisi data dummy dashboard

Dataset CSV terlampir dapat diagregasi dengan seeder Rust:

```powershell
$env:DATABASE_URL="sqlite://./enose.db"
$env:DATASET_ROOT="Raw Data Kopi Rafi\Raw Data Kopi Rafi"
cargo run --bin seed_dataset
```

Seeder membaca 8 kanal sensor dari 12 folder (6 `high_grade` dan 6 `low_grade`), menghitung rerata kanal, lalu membuat satu record dashboard per folder. Record tersebut memakai `device_id` `dummy-dataset-high/low` agar mudah dibedakan dari data ESP32. Nilai score dan accuracy pada record dummy adalah nilai demo untuk visualisasi dashboard, bukan hasil prediksi Edge Impulse.

Contoh ingest:

```powershell
Invoke-RestMethod -Method Post http://localhost:8080/api/v1/measurements -ContentType 'application/json' -Body '{"sample_id":1,"score":86.5,"accuracy":94.2,"device_id":"esp32s3-lab-01","features":[0.12,0.31,0.44]}'
```

## MQTT payload

Publish JSON yang sama ke topic pada `MQTT_TOPIC`. MQTT worker memakai service ingestion yang sama dengan REST API.

## Cloud deployment

Mode default memakai SQLite untuk development. Backend sekarang mendukung SQLite dan PostgreSQL melalui SQLx `AnyPool`.

### Azure Database for PostgreSQL

1. Buat Azure Database for PostgreSQL Flexible Server dan database `enose`.
2. Izinkan koneksi dari service Azure yang akan menjalankan backend.
3. Set environment variable berikut pada App Service/Container Apps, bukan di source code:

```text
DATABASE_URL=postgres://USER:PASSWORD@SERVER.postgres.database.azure.com:5432/enose?sslmode=require
HOST=0.0.0.0
PORT=8080
```

4. Jalankan backend. Schema `measurements` dibuat otomatis saat startup.

Untuk menjalankan seeder ke database Azure, gunakan `DATABASE_URL` PostgreSQL yang sama dan tetap simpan password melalui Azure secret/environment setting:

```powershell
$env:DATABASE_URL="postgres://USER:PASSWORD@SERVER.postgres.database.azure.com:5432/enose?sslmode=require"
cargo run --bin seed_dataset
```

Jangan commit `.env` atau connection string yang berisi password. Dashboard frontend tetap memakai endpoint REST yang sama, sehingga tidak perlu mengetahui credential database.

### Deploy backend dan dashboard ke Azure Container Apps

Database sudah berada di Azure setelah seeder berhasil. Agar dashboard dan backend juga cloud-hosted, gunakan Azure Container Apps. Jalankan Azure CLI dari folder project setelah login:

```powershell
az login
az account set --subscription "SUBSCRIPTION_ID"
az extension add --name containerapp --upgrade
az provider register --namespace Microsoft.App
az provider register --namespace Microsoft.OperationalInsights
az group create --name rg-enose --location indonesiacentral
az acr create --resource-group rg-enose --name enosecloudacr --sku Basic
az acr build --registry enosecloudacr --image enose-cloud:latest .
```

Buat Container Apps environment dan aplikasi:

```powershell
az containerapp env create --name enose-env --resource-group rg-enose --location indonesiacentral

$pgSecret = Read-Host "PostgreSQL DATABASE_URL" -AsSecureString
$pgUrl = (New-Object System.Management.Automation.PSCredential("unused", $pgSecret)).GetNetworkCredential().Password
$acrPassword = az acr credential show --name enosecloudacr --query "passwords[0].value" -o tsv

az containerapp create `
	--name enose-cloud-app `
	--resource-group rg-enose `
	--environment enose-env `
	--image enosecloudacr.azurecr.io/enose-cloud:latest `
	--registry-server enosecloudacr.azurecr.io `
	--registry-username enosecloudacr `
	--registry-password $acrPassword `
	--target-port 8080 `
	--ingress external `
	--secrets "database-url=$pgUrl" `
	--env-vars "DATABASE_URL=secretref:database-url" "HOST=0.0.0.0" "PORT=8080" "MQTT_ENABLED=false"
```

Ambil URL cloud:

```powershell
az containerapp show --name enose-cloud-app --resource-group rg-enose --query properties.configuration.ingress.fqdn -o tsv
```

Buka `https://URL_HASIL/dashboard`. Container Apps menjalankan API dan dashboard yang sama; dashboard tidak lagi bergantung pada komputer lokal. Untuk production, simpan `DATABASE_URL` di Azure Key Vault/secret management dan jangan menampilkannya di terminal history.
