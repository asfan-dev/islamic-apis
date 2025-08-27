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
