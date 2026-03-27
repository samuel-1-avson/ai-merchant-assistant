# AI Merchant Assistant

A voice-driven business intelligence platform for small-to-medium merchants. Built with Rust backend, Next.js frontend, and powered by AI.

## 🎯 Features

- 🎤 **Voice-First Interface** - Record sales transactions with natural speech
- 🤖 **AI-Powered** - Cloud-based AI for transcription, entity extraction, and insights
- 📸 **Receipt OCR** - Scan and process receipts automatically
- 📊 **Real-time Analytics** - Track sales, inventory, and business metrics
- 🔮 **Advanced Forecasting** - Prophet-like time-series forecasting
- 💰 **Price Optimization** - Dynamic pricing suggestions
- 👥 **Customer Analytics** - Cohort analysis and LTV calculations
- 🔔 **Smart Alerts** - Get notified about low stock and sales trends
- 🌍 **Multi-Language** - 6 languages supported (EN, ES, FR, DE, ZH, AR)
- 📱 **Mobile PWA** - Works offline, installable app
- 🌐 **Web-Based** - Access from any device with a browser

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         AI MERCHANT ASSISTANT                           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────────────┐   │
│  │  Next.js 14  │────▶│  Rust/Axum   │────▶│   Supabase (PostgreSQL)│  │
│  │  Frontend    │     │  Backend     │     │   Database             │  │
│  └──────────────┘     └──────┬───────┘     └──────────────────────┘   │
│                              │                                          │
│                              ▼                                          │
│                   ┌─────────────────────┐                              │
│                   │  Cloud AI Services  │                              │
│                   │  • HuggingFace      │                              │
│                   │  • Whisper (STT)    │                              │
│                   │  • Llama 3.1 (LLM)  │                              │
│                   │  • MeloTTS (TTS)    │                              │
│                   │  • EasyOCR          │                              │
│                   └─────────────────────┘                              │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

## 🚀 Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (1.75+)
- [Node.js](https://nodejs.org/) (20+)
- [PostgreSQL](https://www.postgresql.org/) (15+)
- [HuggingFace](https://huggingface.co/) account (free tier)

### Local Development

```bash
# 1. Clone repository
git clone https://github.com/your-org/ai-merchant-assistant.git
cd ai-merchant-assistant

# 2. Setup environment
cp backend/.env.example backend/.env
cp frontend/.env.example frontend/.env.local
# Edit both files with your credentials

# 3. Start with Docker Compose
docker-compose up --build

# 4. Access application
# Frontend: http://localhost:3001
# Backend API: http://localhost:3000
```

### Production Deployment

See [DEPLOYMENT.md](DEPLOYMENT.md) for detailed production deployment instructions.

```bash
# Deploy to production
./scripts/deploy.sh production

# Or use Kubernetes
kubectl apply -f k8s/
```

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [DEPLOYMENT.md](DEPLOYMENT.md) | Production deployment guide |
| [OPERATIONS.md](OPERATIONS.md) | Operations runbook |
| [TESTING.md](TESTING.md) | Testing guide |
| [PHASE2_FIXES_AND_TESTS.md](PHASE2_FIXES_AND_TESTS.md) | Phase 2 summary |
| [PHASE3_SUMMARY.md](PHASE3_SUMMARY.md) | Phase 3 features |

## 📡 API Endpoints

### Authentication
- `POST /api/v1/auth/register` - Register new user
- `POST /api/v1/auth/login` - Login user

### Transactions
- `GET /api/v1/transactions` - List transactions
- `POST /api/v1/transactions` - Create transaction
- `POST /api/v1/transactions/voice` - Create from voice

### Products
- `GET /api/v1/products` - List products
- `POST /api/v1/products` - Create product

### Analytics
- `GET /api/v1/analytics/summary` - Get analytics summary
- `GET /api/v1/analytics/trends` - Get trends
- `GET /api/v1/analytics/forecast` - Get forecast
- `GET /api/v1/analytics/insights` - Get AI insights
- `GET /api/v1/analytics/prophet-forecast` - Prophet forecast
- `GET /api/v1/analytics/customer-ltv` - Customer LTV
- `GET /api/v1/analytics/price-optimization` - Price recommendations

### Alerts
- `GET /api/v1/alerts` - List alerts
- `POST /api/v1/alerts/:id/read` - Mark alert as read
- `POST /api/v1/alerts/check` - Trigger alert check

### OCR
- `POST /api/v1/ocr/receipt` - Process receipt image
- `POST /api/v1/ocr/product` - Scan product

### i18n
- `GET /api/v1/i18n/translations` - Get translations
- `GET /api/v1/i18n/languages` - List supported languages

### WebSocket
- `GET /ws` - Real-time updates

## 📊 Development Phases

### Phase 1: Foundation ✅
- ✅ Project structure
- ✅ Rust backend with Axum
- ✅ Next.js 14+ frontend
- ✅ Database schema & migrations
- ✅ Core API endpoints
- ✅ Voice recording
- ✅ Cloud AI integration

### Phase 2: AI Agents & Analytics ✅
- ✅ Master AI Orchestrator
- ✅ Analytics Engine with materialized views
- ✅ Prediction & Forecasting Agent
- ✅ Alert & Notification System
- ✅ WebSocket real-time updates
- ✅ Comprehensive test suite (34 tests)

### Phase 3: Advanced Features ✅
- ✅ Receipt OCR with EasyOCR
- ✅ Multi-language i18n (6 languages)
- ✅ Mobile PWA with offline support
- ✅ Prophet-like forecasting
- ✅ Price Optimization Engine
- ✅ Customer Analytics (Cohort, LTV)

### Phase 4: Production & Deployment ✅
- ✅ Security hardening
- ✅ Production Docker configuration
- ✅ Kubernetes manifests
- ✅ CI/CD pipeline (GitHub Actions)
- ✅ Monitoring (Prometheus/Grafana)
- ✅ Backup & recovery scripts
- ✅ Operations runbook

## 🧪 Testing

```bash
# Backend tests
cd backend && cargo test

# Frontend tests
cd frontend && npm test

# Integration tests
docker-compose -f docker-compose.test.yml up
```

## 🔒 Security

- JWT authentication
- Rate limiting
- SQL injection protection
- XSS protection
- CSRF tokens
- Security headers (HSTS, CSP, etc.)
- Non-root Docker containers
- Secrets management

## 📈 Monitoring

- Prometheus metrics
- Grafana dashboards
- Health checks
- Error tracking (Sentry)
- Log aggregation

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details.

## 📝 License

MIT License - See [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [Next.js](https://nextjs.org/) - React framework
- [Supabase](https://supabase.io/) - Database & Auth
- [HuggingFace](https://huggingface.co/) - AI models

---

Built with ❤️ for merchants everywhere.
