# Supabase Setup Guide

## ✅ Configuration Complete!

Your application is now configured to use Supabase PostgreSQL.

---

## 🚀 Quick Start

### 1. Set Up Database Schema

1. Go to your Supabase Dashboard: https://supabase.com/dashboard/project/mufjqnudxzkaandzohbj
2. Click **"SQL Editor"** (left sidebar)
3. Click **"New Query"**
4. Copy the contents of `supabase_schema.sql` file
5. Paste and click **"Run"**

### 2. Run the Application

```bash
docker-compose up --build
```

---

## 📁 Environment Files Updated

Your `.env` files now contain:

| Variable | Source | Usage |
|----------|--------|-------|
| `DATABASE_URL` | Supabase → Connect | Backend database connection |
| `SUPABASE_URL` | Supabase → API | Project URL |
| `SUPABASE_SERVICE_KEY` | Supabase → API → service_role | Backend admin access |
| `SUPABASE_ANON_KEY` | Supabase → API → anon | Frontend client access |

---

## 🔐 Security Notes

### What Was Exposed in Chat
- ✅ Database password
- ✅ Service role key
- ✅ Anon key

### You Should Do This (Recommended)

1. **Rotate Database Password** (optional but safer):
   - Supabase Dashboard → Database → Reset Password
   - Update `.env` files with new password

2. **Regenerate API Keys** (optional but safer):
   - Supabase Dashboard → Project Settings → API
   - Regenerate service_role and anon keys
   - Update `.env` files

3. **Never commit `.env` files**:
   ```bash
   # Already protected by .gitignore
   git status  # Should not show .env files
   ```

---

## 🧪 Testing the Connection

### Test Backend
```bash
curl http://localhost:3000/health
```
Expected: `{"status":"ok"}`

### Check Database Tables
```bash
# In Supabase SQL Editor, run:
SELECT * FROM users;
```

---

## 📊 Supabase Resources

- **Dashboard:** https://supabase.com/dashboard/project/mufjqnudxzkaandzohbj
- **Database:** PostgreSQL 15 on AWS eu-west-1
- **Connection Pooler:** Port 6543 (Transaction mode)

---

## 🐛 Troubleshooting

### "Connection refused" error
- Check if `.env` file exists in project root
- Verify DATABASE_URL is correct

### "relation does not exist" error
- Run `supabase_schema.sql` in SQL Editor first

### JWT errors
- Get JWT Secret from: Project Settings → API → JWT Settings
- Update `SUPABASE_JWT_SECRET` in `.env`

---

## ✅ Next Steps

1. ⬜ Run the SQL schema in Supabase
2. ⬜ Build and start: `docker-compose up --build`
3. ⬜ Test: http://localhost:3001
4. ⬜ (Optional) Rotate credentials for extra security
