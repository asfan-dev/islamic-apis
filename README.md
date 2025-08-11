# Islamic APIs - Comprehensive Islamic Services Platform

A professional, scalable suite of Islamic APIs built with Rust, providing prayer times, Qibla direction, Dua collections, and Zakat calculations. Designed for high performance, reliability, and ease of use.

## 🌟 Features

### 🕌 Prayer Times API
- **Accurate Calculations**: Multiple calculation methods (MWL, ISNA, Karachi, etc.)
- **Global Coverage**: Works for any location worldwide
- **Flexible Timespans**: Daily, monthly, yearly calculations
- **High Latitude Support**: Special rules for polar regions
- **Custom Methods**: Define your own calculation parameters

### 🧭 Qibla Direction API
- **Precise Direction**: Accurate Qibla direction from any location
- **Detailed Information**: Distance to Kaaba, compass directions
- **Validation**: Coordinate validation with helpful suggestions
- **Multiple Formats**: Support for both GET and POST requests

### 📿 Dua & Supplications API
- **Comprehensive Database**: Extensive collection of authentic Islamic duas
- **Full-Text Search**: Search by Arabic text, transliteration, or translation
- **Categorized**: Organized by categories (Daily, Food, Travel, etc.)
- **Multilingual**: Arabic text with transliteration and translations
- **Audio Support**: Optional audio URL support for recitations

### 💰 Zakat Calculator API
- **Multiple Asset Types**: Wealth, Gold, Silver, Business, Livestock, Crops
- **Current Rates**: Real-time nisab calculations based on current market prices
- **Multiple Currencies**: Support for 12+ major currencies
- **Islamic Guidelines**: Built-in Islamic references and recommendations
- **Calculation History**: Save and track calculations for registered users

## 🏗️ Architecture

### Technology Stack
- **Language**: Rust (1.75+)
- **Web Framework**: Axum
- **Database**: PostgreSQL
- **Cache**: Redis
- **Reverse Proxy**: Nginx
- **Monitoring**: Prometheus + Grafana
- **Containerization**: Docker + Docker Compose

### System Design
```
┌─────────────────┐    ┌──────────────┐    ┌─────────────────┐
│     Client      │────│    Nginx     │────│   API Gateway   │
└─────────────────┘    │ Load Balancer│    └─────────────────┘
                       └──────────────┘             │
                                                    │
        ┌───────────────────────────────────────────┼───────────────────────────────────────────┐
        │                                           │                                           │
        ▼                                           ▼                                           ▼
┌─────────────────┐                        ┌─────────────────┐                        ┌─────────────────┐
│ Prayer Times API│                        │   Qibla API     │                        │    Dua API      │
│     Port 3001   │                        │   Port 3002     │                        │   Port 3003     │
└─────────────────┘                        └─────────────────┘                        └─────────────────┘
        │                                           │                                           │
        └───────────────────────────────────────────┼───────────────────────────────────────────┘
                                                    │
                                                    ▼
                                           ┌─────────────────┐
                                           │   Zakat API     │
                                           │   Port 3004     │
                                           └─────────────────┘
                                                    │
        ┌───────────────────────────────────────────┼───────────────────────────────────────────┐
        │                                           │                                           │
        ▼                                           ▼                                           ▼
┌─────────────────┐                        ┌─────────────────┐                        ┌─────────────────┐
│   PostgreSQL    │                        │      Redis      │                        │   Prometheus    │
│   Port 5432     │                        │    Port 6379    │                        │   Port 9090     │
└─────────────────┘                        └─────────────────┘                        └─────────────────┘
```

## 🚀 Quick Start

### Prerequisites
- Docker & Docker Compose
- Git
- 4GB+ RAM recommended
- 10GB+ free disk space

### 1. Clone the Repository
```bash
git clone https://github.com/your-org/islamic-apis.git
cd islamic-apis
```

### 2. Environment Configuration
```bash
# Copy environment template
cp .env.example .env

# Edit configuration (update database credentials, API keys, etc.)
nano .env
```

### 3. Deploy with Docker Compose
```bash
# Make deployment script executable
chmod +x deploy.sh

# Run full deployment
./deploy.sh deploy
```

### 4. Verify Deployment
```bash
# Check service status
./deploy.sh status

# View logs
./deploy.sh logs

# Test APIs
curl http://localhost/health
```

## 📚 API Documentation

### Prayer Times API

#### Calculate Prayer Times
```http
POST /api/v1/prayer-times
Content-Type: application/json

{
  "latitude": 40.7128,
  "longitude": -74.0060,
  "timezone": "America/New_York",
  "method": "isna",
  "timespan": {
    "daysfromtoday": 7
  }
}
```

**Response:**
```json
{
  "qibla_direction": 58.48,
  "next": {
    "name": "Fajr",
    "time": "15/12/2024 05:30"
  },
  "prayers": [
    {
      "imsak": "15/12/2024 05:20",
      "fajr": "15/12/2024 05:30",
      "sunrise": "15/12/2024 07:15",
      "dhuhr": "15/12/2024 11:55",
      "asr": "15/12/2024 14:30",
      "sunset": "15/12/2024 16:35",
      "maghrib": "15/12/2024 16:35",
      "isha": "15/12/2024 18:00",
      "midnight": "15/12/2024 23:55",
      "first_third": "15/12/2024 21:15",
      "last_third": "16/12/2024 02:35",
      "date": "15/12/2024",
      "hijri": "14/06/1446"
    }
  ],
  "meta": {
    "method": "isna",
    "settings": {
      "fajr": 15.0,
      "isha": {"angle": 15.0},
      "school": "standard"
    },
    "timezone": "America/New_York",
    "coordinates": {
      "latitude": 40.7128,
      "longitude": -74.0060,
      "elevation": 0.0
    }
  }
}
```

### Qibla Direction API

#### Get Qibla Direction
```http
POST /api/v1/qibla
Content-Type: application/json

{
  "latitude": 40.7128,
  "longitude": -74.0060,
  "elevation": 10.0
}
```

**Or via GET:**
```http
GET /api/v1/qibla?lat=40.7128&lng=-74.0060&elevation=10.0&detailed=true
```

**Response:**
```json
{
  "qibla_direction": 58.481234,
  "qibla_direction_compass": "ENE (58.5°)",
  "distance_km": 11041.32,
  "location": {
    "latitude": 40.7128,
    "longitude": -74.0060,
    "elevation": 10.0,
    "description": "40.7128°N, 74.0060°W (Elevation: 10m)"
  },
  "kaaba_location": {
    "latitude": 21.4224779,
    "longitude": 39.8251832,
    "elevation": 333.0,
    "description": "Holy Kaaba, Masjid al-Haram, Mecca, Saudi Arabia"
  },
  "calculation_method": "Great Circle Method (Haversine Formula)",
  "calculation_time": "2024-12-15T10:30:00Z"
}
```

### Dua API

#### Search Duas
```http
GET /api/v1/duas/search?q=morning&category=daily&page=1&limit=10
```

#### Get All Duas
```http
GET /api/v1/duas?page=1&limit=20&sort=createddesc
```

#### Create New Dua
```http
POST /api/v1/duas
Content-Type: application/json

{
  "title": "Before Eating",
  "arabic_text": "بِسْمِ اللَّهِ",
  "transliteration": "Bismillah",
  "translation": "In the name of Allah",
  "reference": "Bukhari",
  "category": "Food & Drink",
  "tags": ["eating", "food", "basic"]
}
```

### Zakat API

#### Calculate Zakat
```http
POST /api/v1/zakat/calculate
Content-Type: application/json

{
  "calculation_type": "wealth",
  "amount": 10000.00,
  "currency": "usd"
}
```

#### Get Nisab Rates
```http
GET /api/v1/zakat/nisab
```

#### Get Zakat Information
```http
GET /api/v1/zakat/info
```

## 🔧 Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE__URL` | PostgreSQL connection string | Required |
| `REDIS__URL` | Redis connection string | Required |
| `SERVER__HOST` | Server bind address | 0.0.0.0 |
| `SERVER__PORT` | Server port | 3000 |
| `RATE_LIMIT__REQUESTS_PER_MINUTE` | Rate limit per minute | 100 |
| `RUST_LOG` | Logging level | info |

### Rate Limiting

Each API has specific rate limits to ensure fair usage:
- **Prayer Times API**: 200 requests/minute
- **Qibla API**: 300 requests/minute
- **Dua API**: 150 requests/minute
- **Zakat API**: 100 requests/minute

### Caching Strategy

- **Prayer Times**: 1 hour cache TTL
- **Qibla Direction**: 24 hour cache TTL
- **Dua Searches**: 30 minutes cache TTL
- **Zakat Rates**: 1 hour cache TTL

## 🛠️ Development

### Local Development Setup

1. **Install Rust:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. **Start dependencies:**
```bash
docker-compose up postgres redis
```

3. **Run migrations:**
```bash
sqlx migrate run --database-url "postgresql://postgres:postgres123@localhost:5432/islamic_apis"
```

4. **Run individual services:**
```bash
# Prayer Times API
cd prayer-times-api
cargo run

# Qibla API
cd qibla-api
cargo run

# Dua API
cd dua-api
cargo run

# Zakat API
cd zakat-api
cargo run
```

### Testing

```bash
# Run all tests
cargo test

# Run tests for specific service
cargo test --package prayer-times-api

# Run with coverage
cargo tarpaulin --out Html
```

### Database Migrations

```bash
# Create new migration
sqlx migrate add create_new_table

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

## 🐳 Docker Deployment

### Production Deployment

1. **Update environment:**
```bash
cp .env.example .env.production
# Edit production values
```

2. **Deploy with production config:**
```bash
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

3. **Scale services:**
```bash
docker-compose up -d --scale prayer-times-api=3 --scale qibla-api=2
```

### Health Monitoring

All services include health check endpoints:
- Individual: `http://localhost:300X/health`
- Gateway: `http://localhost/health`
- Prometheus metrics: `http://localhost:9090`
- Grafana dashboards: `http://localhost:3000`

## 📊 Monitoring & Observability

### Metrics

- **Request/Response metrics**: Latency, throughput, error rates
- **System metrics**: CPU, memory, disk usage
- **Database metrics**: Connection pool, query performance
- **Cache metrics**: Hit/miss ratios, memory usage

### Logging

- **Structured logging** with JSON format
- **Distributed tracing** for request correlation
- **Log levels**: Error, Warn, Info, Debug, Trace
- **Log rotation** and retention policies

### Alerts

Key alerts configured:
- API response time > 2 seconds
- Error rate > 5%
- Database connection failures
- Redis connection failures
- High memory usage (>80%)

## 🔒 Security

### Authentication & Authorization
- **Rate limiting** on all endpoints
- **CORS** configuration for web access
- **Input validation** on all requests
- **SQL injection** prevention with prepared statements

### Data Protection
- **Encryption** at rest and in transit
- **Secure headers** (HSTS, CSP, etc.)
- **No sensitive data** in logs
- **Database credentials** via environment variables

### Best Practices
- **Principle of least privilege**
- **Regular security updates**
- **Container security scanning**
- **Network segmentation**

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Process
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Update documentation
6. Submit a pull request

### Code Standards
- **Rust formatting**: Use `cargo fmt`
- **Linting**: Use `cargo clippy`
- **Testing**: Maintain >80% test coverage
- **Documentation**: Document all public APIs

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Islamic Society of North America (ISNA)** for calculation methods
- **Muslim World League (MWL)** for prayer time standards
- **University of Islamic Sciences, Karachi** for Hanafi calculations
- **Contributors** who helped build and improve these APIs

## 🆘 Support

### Getting Help
- **Documentation**: Check this README and API docs
- **Issues**: Report bugs on GitHub Issues
- **Discussions**: Join GitHub Discussions for questions
- **Email**: support@islamic-apis.com

### Troubleshooting

#### Common Issues

**Services won't start:**
```bash
# Check logs
./deploy.sh logs

# Verify configuration
./deploy.sh status
```

**Database connection errors:**
```bash
# Check database status
docker-compose exec postgres pg_isready -U postgres

# Restart database
docker-compose restart postgres
```

**Cache connection errors:**
```bash
# Check Redis status
docker-compose exec redis redis-cli ping

# Restart Redis
docker-compose restart redis
```

#### Performance Issues

**High response times:**
- Check database connection pool settings
- Verify Redis cache is working
- Monitor system resources

**Rate limit errors:**
- Adjust rate limit settings in environment
- Implement request queuing on client side
- Consider API key system for higher limits

## 🗺️ Roadmap

### v1.1 (Q1 2024)
- [ ] GraphQL API support
- [ ] WebSocket real-time updates
- [ ] Mobile SDK (React Native)
- [ ] Advanced search in Dua API

### v1.2 (Q2 2024)
- [ ] Multi-language support
- [ ] Islamic calendar integration
- [ ] Hadith database API
- [ ] Advanced Zakat scenarios

### v2.0 (Q3 2024)
- [ ] Microservices refactoring
- [ ] Kubernetes support
- [ ] Machine learning prayer time optimization
- [ ] Advanced analytics dashboard

---

**Made with ❤️ for the Muslim community worldwide**

For more information, visit our [website](https://islamic-apis.com) or contact us at [info@islamic-apis.com](mailto:info@islamic-apis.com).