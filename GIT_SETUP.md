# Git Setup Guide

## Step 1: Create GitHub Repository

1. Go to https://github.com/new
2. Enter repository name: `ai-merchant-assistant`
3. Add description: "Voice-driven business intelligence platform for merchants"
4. Choose "Public" (or Private if preferred)
5. **DO NOT** initialize with README (we already have one)
6. Click "Create repository"

## Step 2: Configure Git (One-time setup)

```bash
# Set your name and email
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"
```

## Step 3: Initialize and Push (Run in project directory)

```bash
# Navigate to project directory
cd ai-merchant-assistant

# Initialize git (if not already done)
git init

# Add .gitignore
git add .gitignore

# Add all project files (excluding target/ and node_modules/ due to .gitignore)
git add .

# Commit the files
git commit -m "Initial commit: AI Merchant Assistant - Complete project

Features:
- Voice-driven sales recording
- AI-powered analytics and forecasting
- Receipt OCR
- Multi-language support (6 languages)
- Mobile PWA
- Price optimization
- Customer analytics
- Production-ready with K8s, CI/CD, monitoring"

# Add remote repository (replace with your actual URL)
git remote add origin https://github.com/YOUR_USERNAME/ai-merchant-assistant.git

# Push to GitHub
git push -u origin main
```

## Alternative: Using GitHub CLI

If you have GitHub CLI installed:

```bash
# Create repo and push in one command
cd ai-merchant-assistant
gh repo create ai-merchant-assistant --public --source=. --push
```

## Post-Push Verification

1. Visit your repository: `https://github.com/YOUR_USERNAME/ai-merchant-assistant`
2. Verify all files are present
3. Check that README.md displays correctly
4. Review the file structure

## Repository Structure on GitHub

```
ai-merchant-assistant/
├── .github/
│   └── workflows/         # CI/CD pipelines
├── backend/               # Rust backend
│   ├── src/              # Source code
│   └── tests/            # Test files
├── frontend/              # Next.js frontend
│   ├── src/              # Source code
│   └── public/           # Static assets
├── k8s/                   # Kubernetes manifests
├── monitoring/            # Prometheus & Grafana
├── scripts/               # Deployment scripts
├── docs/                  # Phase summaries
├── docker-compose*.yml    # Docker configs
├── Dockerfile.*          # Dockerfiles
└── README.md             # Main documentation
```

## Next Steps After Push

1. **Enable GitHub Actions**
   - Go to repository → Actions tab
   - Enable workflows

2. **Set up Secrets** (for CI/CD)
   - Go to Settings → Secrets and variables → Actions
   - Add secrets:
     - `DOCKER_USERNAME`
     - `DOCKER_PASSWORD`
     - `KUBE_CONFIG` (if deploying to K8s)
     - `SLACK_WEBHOOK` (for notifications)

3. **Configure Branch Protection**
   - Go to Settings → Branches
   - Add rule for `main` branch:
     - Require pull request reviews
     - Require status checks to pass
     - Require linear history

4. **Add Topics/Tags**
   - Click "Manage topics"
   - Add: `rust`, `nextjs`, `ai`, `merchant`, `voice`, `analytics`

5. **Create Releases**
   - After initial push, create v1.0.0 release
   - Document key features in release notes

## Common Issues

### Issue: "index.lock exists"
```bash
# Remove the lock file
rm -f .git/index.lock
```

### Issue: Large file rejection
```bash
# If target/ or node_modules/ were accidentally committed
# Remove them from git history
git rm -r --cached backend/target
git rm -r --cached frontend/node_modules
git commit -m "Remove build artifacts"
git push
```

### Issue: Authentication
```bash
# Use personal access token instead of password
# Generate at: https://github.com/settings/tokens
git remote set-url origin https://TOKEN@github.com/USERNAME/REPO.git
```

## Repository Stats

| Metric | Count |
|--------|-------|
| Files | 285 |
| Languages | Rust, TypeScript, SQL, YAML |
| Tests | 46+ |
| Commits | Starting with initial commit |
| Contributors | You! |

## Badges for README

Add these badges to your README.md:

```markdown
[![CI](https://github.com/YOUR_USERNAME/ai-merchant-assistant/actions/workflows/test.yml/badge.svg)](https://github.com/YOUR_USERNAME/ai-merchant-assistant/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Next.js](https://img.shields.io/badge/Next.js-14-black.svg)](https://nextjs.org)
```

## Need Help?

- GitHub Docs: https://docs.github.com
- Git Cheat Sheet: https://git-scm.com/docs/gittutorial
- Contact: Create an issue in the repository
