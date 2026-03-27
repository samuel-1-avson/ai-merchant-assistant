# AI Merchant Assistant - Implementation Plan

**Date:** March 27, 2026
**Status:** In Progress

---

## ✅ Completed Features

### 1. Database Layer (Supabase)
| Feature | Status | Notes |
|---------|--------|-------|
| Supabase PostgreSQL Connection | ✅ Complete | Connected and working |
| User Repository | ✅ Complete | CRUD operations implemented |
| Transaction Repository | ✅ Complete | Real database queries |
| Product Repository | ✅ Complete | Real database queries |

### 2. Authentication (Email/Password)
| Feature | Status | Notes |
|---------|--------|-------|
| User Registration | ✅ Complete | `/api/v1/auth/register` working |
| User Login | ✅ Complete | `/api/v1/auth/login` working |
| JWT Token Generation | ✅ Complete | 24-hour expiration |
| Password Hashing (Argon2) | ✅ Complete | Secure hashing implemented |
| Frontend Login Page | ✅ Complete | `/auth/login` accessible |
| Frontend Register Page | ✅ Complete | `/auth/register` created |

### 3. Core Application
| Feature | Status | Notes |
|---------|--------|-------|
| Dashboard | ✅ Complete | Real-time analytics |
| Voice Recording | ✅ Complete | AI pipeline integrated |
| WebSocket Real-time | ✅ Complete | Live updates working |
| Transaction Management | ✅ Complete | Create, list transactions |

---

## 🚧 Pending Implementation

### 1. Google OAuth Authentication
**Priority:** HIGH
**Estimated Time:** 4-6 hours

#### Backend Tasks:
- [ ] Add Google OAuth client configuration
- [ ] Create `/api/v1/auth/google` endpoint
- [ ] Implement OAuth callback handler
- [ ] Link Google account to existing user or create new user
- [ ] Generate JWT token after OAuth success

#### Frontend Tasks:
- [ ] Add Google Sign-In button functionality
- [ ] Handle OAuth popup/redirect flow
- [ ] Send OAuth token to backend
- [ ] Handle successful authentication

#### Configuration Required:
```env
# Add to .env
GOOGLE_CLIENT_ID=your-google-client-id
GOOGLE_CLIENT_SECRET=your-google-client-secret
GOOGLE_REDIRECT_URI=http://localhost:8888/api/v1/auth/google/callback
```

### 2. GitHub OAuth Authentication
**Priority:** MEDIUM
**Estimated Time:** 3-4 hours

Similar to Google OAuth but using GitHub's OAuth flow.

### 3. Email Verification
**Priority:** HIGH
**Estimated Time:** 3-4 hours

#### Backend Tasks:
- [ ] Add `email_verified` field to users table
- [ ] Create email verification token generation
- [ ] Create `/api/v1/auth/verify-email` endpoint
- [ ] Integrate email service (SendGrid/AWS SES)

#### Frontend Tasks:
- [ ] Create verification pending page
- [ ] Create email verified success page
- [ ] Show verification status in profile

### 4. Forgot Password / Password Reset
**Priority:** HIGH
**Estimated Time:** 3-4 hours

#### Backend Tasks:
- [ ] Create password reset token generation
- [ ] Create `/api/v1/auth/forgot-password` endpoint
- [ ] Create `/api/v1/auth/reset-password` endpoint
- [ ] Send reset email with token

#### Frontend Tasks:
- [ ] Create forgot password page
- [ ] Create reset password page
- [ ] Handle token validation

### 5. Missing Pages
| Page | Status | Priority |
|------|--------|----------|
| `/auth/forgot-password` | ❌ Not Created | HIGH |
| `/auth/reset-password` | ❌ Not Created | HIGH |
| `/auth/verify-email` | ❌ Not Created | HIGH |
| `/dashboard/profile` | ❌ Not Created | MEDIUM |
| `/dashboard/settings` | ❌ Not Created | MEDIUM |
| `/dashboard/products` | ❌ Not Created | MEDIUM |

---

## 📊 Implementation Timeline

### Week 1: OAuth & Security
- Day 1-2: Google OAuth implementation
- Day 3: GitHub OAuth implementation
- Day 4: Email verification system
- Day 5: Forgot/reset password

### Week 2: UI/UX Enhancement
- Day 1-2: Create missing auth pages
- Day 3-4: Profile & Settings pages
- Day 5: Products management page

---

## 🔧 Technical Details

### Current Architecture
```
Frontend (Next.js)  ←→  Backend (Rust/Axum)  ←→  Supabase PostgreSQL
       ↑                                          ↑
       └───────────  WebSocket  ─────────────────┘
```

### OAuth Flow Design
```
1. User clicks "Sign in with Google"
2. Frontend opens Google OAuth popup
3. User authenticates with Google
4. Google returns OAuth token
5. Frontend sends token to backend
6. Backend validates with Google
7. Backend creates/links user account
8. Backend returns JWT token
9. Frontend stores token and redirects to dashboard
```

### Database Schema (Users Table)
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255), -- NULL for OAuth users
    full_name VARCHAR(255),
    business_name VARCHAR(255),
    google_id VARCHAR(255) UNIQUE, -- For Google OAuth
    github_id VARCHAR(255) UNIQUE, -- For GitHub OAuth
    email_verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

---

## 🎯 Next Steps

### Immediate Actions:
1. **Set up Google OAuth credentials** in Google Cloud Console
2. **Add OAuth environment variables** to `.env`
3. **Implement backend OAuth endpoints**
4. **Connect frontend OAuth buttons**

### Testing Checklist:
- [ ] Register with email/password works
- [ ] Login with email/password works
- [ ] Google OAuth sign-in works
- [ ] Email verification sent
- [ ] Password reset flow works
- [ ] All pages accessible

---

## 📚 Resources

### Google OAuth Setup:
1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project
3. Enable Google+ API
4. Create OAuth 2.0 credentials
5. Add authorized redirect URIs:
   - `http://localhost:8888/api/v1/auth/google/callback` (development)
   - `https://yourdomain.com/api/v1/auth/google/callback` (production)

### Environment Variables:
```env
# Database
DATABASE_URL=postgresql://...

# Supabase
SUPABASE_URL=https://...
SUPABASE_SERVICE_KEY=...

# JWT
JWT_SECRET=your-super-secret-jwt-key

# Google OAuth
GOOGLE_CLIENT_ID=your-client-id.apps.googleusercontent.com
GOOGLE_CLIENT_SECRET=your-client-secret
GOOGLE_REDIRECT_URI=http://localhost:8888/api/v1/auth/google/callback

# Frontend
NEXT_PUBLIC_API_URL=http://localhost:8888/api/v1
```

---

## 📝 Notes

- The database is **already connected** to Supabase and working
- Email/password authentication is **fully functional**
- The main gap is **OAuth integration** and **email verification**
- All Docker deployment issues have been resolved

---

**Last Updated:** March 27, 2026
**Next Review:** After OAuth Implementation
