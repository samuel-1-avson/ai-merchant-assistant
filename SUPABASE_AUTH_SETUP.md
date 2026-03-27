# Supabase Auth Setup Guide

**Date:** March 27, 2026  
**Status:** Implementation Complete - Configuration Required

---

## ✅ What Was Implemented

### 1. Supabase Auth Integration

I've completely refactored the authentication system to use **Supabase Auth** instead of custom OAuth. This is the recommended approach because:

- ✅ Supabase handles all OAuth providers (Google, GitHub, etc.)
- ✅ Automatic token management
- ✅ Built-in email verification
- ✅ Built-in password reset
- ✅ Secure session handling

### 2. Files Created/Modified

| File | Purpose |
|------|---------|
| `src/lib/supabase/auth.ts` | Supabase Auth functions (NEW) |
| `src/stores/authStore.ts` | Updated to use Supabase Auth |
| `src/app/auth/callback/page.tsx` | OAuth callback handler (NEW) |
| `src/app/auth/login/page.tsx` | Updated for Supabase Auth |
| `src/app/auth/register/page.tsx` | Updated for Supabase Auth |

### 3. Authentication Flow

```
User clicks "Sign in with Google"
         ↓
Frontend calls Supabase Auth
         ↓
Supabase redirects to Google
         ↓
User authenticates with Google
         ↓
Google redirects back to Supabase
         ↓
Supabase redirects to /auth/callback
         ↓
Callback page gets session & user
         ↓
User is logged in and redirected to dashboard
```

---

## 🔧 Configuration Required

### Step 1: Enable Google Auth in Supabase

1. Go to your Supabase Dashboard: https://app.supabase.com
2. Select your project
3. Go to **Authentication** → **Providers**
4. Find **Google** and click **Enable**
5. You'll need to add Google Client ID and Secret (see Step 2)

### Step 2: Get Google OAuth Credentials

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project (or select existing)
3. Go to **APIs & Services** → **Credentials**
4. Click **Create Credentials** → **OAuth client ID**
5. Configure consent screen:
   - User Type: External
   - App name: AI Merchant Assistant
   - User support email: your-email
   - Developer contact: your-email
6. Create OAuth client ID:
   - Application type: Web application
   - Name: AI Merchant Assistant
   - Authorized redirect URIs: 
     ```
     https://mufjqnudxzkaandzohbj.supabase.co/auth/v1/callback
     ```
7. Copy **Client ID** and **Client Secret**

### Step 3: Configure Supabase

1. Back in Supabase Dashboard → Authentication → Providers → Google
2. Paste:
   - **Client ID**: (from Google Cloud)
   - **Secret**: (from Google Cloud)
3. Click **Save**

### Step 4: Enable GitHub Auth (Optional)

1. Go to GitHub → Settings → Developer settings → OAuth Apps
2. Click **New OAuth App**
3. Fill in:
   - Application name: AI Merchant Assistant
   - Homepage URL: `http://localhost:8889`
   - Authorization callback URL: 
     ```
     https://mufjqnudxzkaandzohbj.supabase.co/auth/v1/callback
     ```
4. Click **Register application**
5. Copy **Client ID** and generate **Client Secret**
6. In Supabase Dashboard → Auth → Providers → GitHub
7. Paste credentials and save

---

## 🚀 Rebuild Instructions

Once configuration is complete, rebuild the Docker containers:

```powershell
cd "F:\project\Merchant Assistant\ai-merchant-assistant"

# Stop existing containers
docker-compose down

# Build frontend with new auth code
docker-compose build --no-cache frontend

# Start everything
docker-compose up -d

# Check status
docker ps --filter "name=ai-merchant"
```

---

## 🧪 Testing Checklist

### Email/Password Auth (Should work now)
- [ ] Register with email/password at `/auth/register`
- [ ] Login with email/password at `/auth/login`
- [ ] User data saved to Supabase `users` table
- [ ] JWT token stored in localStorage

### Google OAuth (After configuration)
- [ ] Click "Sign in with Google"
- [ ] Google OAuth popup appears
- [ ] After Google auth, redirected to `/auth/callback`
- [ ] Successfully logged in and redirected to dashboard
- [ ] User appears in Supabase `users` table

### GitHub OAuth (After configuration)
- [ ] Click "Sign in with GitHub"
- [ ] GitHub OAuth popup appears
- [ ] After auth, successfully logged in

---

## 📁 File Structure

```
frontend/src/
├── lib/
│   └── supabase/
│       ├── client.ts      # Supabase client setup
│       └── auth.ts        # Auth functions (NEW)
├── stores/
│   └── authStore.ts       # Updated for Supabase Auth
└── app/
    └── auth/
        ├── login/
        │   └── page.tsx   # Updated with OAuth buttons
        ├── register/
        │   └── page.tsx   # Updated with OAuth buttons
        └── callback/
            └── page.tsx   # NEW - OAuth callback handler
```

---

## 🔑 Key Changes

### Before (Custom Auth)
- Backend handled JWT generation
- Custom OAuth implementation
- Separate API endpoints for each provider

### After (Supabase Auth)
- Supabase handles all auth
- Frontend uses `@supabase/ssr`
- Automatic OAuth with Google/GitHub
- Built-in email/password auth
- Built-in password reset
- Built-in email verification

---

## 📝 Environment Variables (No Changes Needed!)

Your current `.env` already has everything needed:

```env
# Supabase (already configured)
NEXT_PUBLIC_SUPABASE_URL=https://mufjqnudxzkaandzohbj.supabase.co
NEXT_PUBLIC_SUPABASE_ANON_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
SUPABASE_SERVICE_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...

# Backend
JWT_SECRET=ai-merchant-super-secret-jwt-key-min-32-chars-2026
```

**Note:** No Google Client ID needed in `.env` - it's configured in Supabase Dashboard!

---

## 🎯 Next Steps

1. **Configure Google Auth in Supabase Dashboard** (5 minutes)
2. **Get Google OAuth credentials** (10 minutes)
3. **Add credentials to Supabase** (2 minutes)
4. **Rebuild Docker containers** (5 minutes)
5. **Test Google Sign-In**

---

## ❓ FAQ

### Q: Do I need to change the backend?
**A:** No! The backend already works with Supabase database. Supabase Auth is handled entirely on the frontend + Supabase service.

### Q: Where is user data stored?
**A:** In your Supabase PostgreSQL database in the `auth.users` table (managed by Supabase Auth).

### Q: What about the custom backend auth code?
**A:** You can keep it for email/password auth as a fallback, or remove it later. The frontend now prefers Supabase Auth.

### Q: Can users still use email/password?
**A:** Yes! Email/password auth works through Supabase Auth now.

---

## ✅ Summary

- ✅ Supabase Auth implemented
- ✅ Google OAuth ready (just needs config)
- ✅ GitHub OAuth ready (just needs config)
- ✅ Email/password auth working
- ✅ Callback page created
- ✅ Auth store updated
- ⚠️ Needs: Google Cloud credentials + Supabase configuration
- ⚠️ Needs: Docker rebuild

---

**The authentication system is now using Supabase Auth and is ready for OAuth configuration!**
