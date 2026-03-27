/**
 * Supabase Authentication
 * Handles OAuth (Google, GitHub) via Supabase Auth
 */

import { createClient } from './client'

const supabase = createClient()

/**
 * Sign in with Google OAuth
 */
export async function signInWithGoogle() {
  const { data, error } = await supabase.auth.signInWithOAuth({
    provider: 'google',
    options: {
      redirectTo: typeof window !== 'undefined' ? `${window.location.origin}/auth/callback` : '/auth/callback',
    },
  })

  if (error) {
    throw new Error(error.message)
  }

  return data
}

/**
 * Sign in with GitHub OAuth
 */
export async function signInWithGitHub() {
  const { data, error } = await supabase.auth.signInWithOAuth({
    provider: 'github',
    options: {
      redirectTo: typeof window !== 'undefined' ? `${window.location.origin}/auth/callback` : '/auth/callback',
    },
  })

  if (error) {
    throw new Error(error.message)
  }

  return data
}

/**
 * Sign up with email/password
 */
export async function signUpWithEmail(email: string, password: string, metadata?: {
  full_name?: string
  business_name?: string
}) {
  const { data, error } = await supabase.auth.signUp({
    email,
    password,
    options: {
      data: metadata,
      emailRedirectTo: typeof window !== 'undefined' ? `${window.location.origin}/auth/callback` : '/auth/callback',
    },
  })

  if (error) {
    throw new Error(error.message)
  }

  return data
}

/**
 * Sign in with email/password
 */
export async function signInWithEmail(email: string, password: string) {
  const { data, error } = await supabase.auth.signInWithPassword({
    email,
    password,
  })

  if (error) {
    throw new Error(error.message)
  }

  return data
}

/**
 * Sign out
 */
export async function signOut() {
  const { error } = await supabase.auth.signOut()
  
  if (error) {
    throw new Error(error.message)
  }
}

/**
 * Get current session
 */
export async function getSession() {
  const { data, error } = await supabase.auth.getSession()
  
  if (error) {
    throw new Error(error.message)
  }

  return data.session
}

/**
 * Get current user
 */
export async function getUser() {
  const { data, error } = await supabase.auth.getUser()
  
  if (error) {
    throw new Error(error.message)
  }

  return data.user
}

/**
 * Subscribe to auth changes
 */
export function onAuthStateChange(callback: (event: string, session: any) => void) {
  return supabase.auth.onAuthStateChange(callback)
}

/**
 * Resend email verification
 */
export async function resendVerificationEmail(email: string) {
  const { error } = await supabase.auth.resend({
    type: 'signup',
    email,
  })

  if (error) {
    throw new Error(error.message)
  }
}

/**
 * Send password reset email
 */
export async function sendPasswordResetEmail(email: string) {
  const { error } = await supabase.auth.resetPasswordForEmail(email, {
    redirectTo: typeof window !== 'undefined' ? `${window.location.origin}/auth/reset-password` : '/auth/reset-password',
  })

  if (error) {
    throw new Error(error.message)
  }
}

/**
 * Update user password
 */
export async function updatePassword(newPassword: string) {
  const { error } = await supabase.auth.updateUser({
    password: newPassword,
  })

  if (error) {
    throw new Error(error.message)
  }
}
