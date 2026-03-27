/**
 * Authentication Store - Zustand
 * Uses Supabase Auth for authentication
 */

import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import { User } from '@/types'
import {
  signInWithEmail,
  signUpWithEmail,
  signInWithGoogle,
  signInWithGitHub,
  signOut as supabaseSignOut,
  getSession,
  getUser,
  onAuthStateChange,
  sendPasswordResetEmail,
} from '@/lib/supabase/auth'

interface AuthState {
  // State
  user: User | null
  token: string | null
  isAuthenticated: boolean
  isLoading: boolean
  error: string | null

  // Actions
  login: (email: string, password: string) => Promise<boolean>
  register: (data: {
    email: string
    password: string
    full_name?: string
    business_name?: string
  }) => Promise<boolean>
  googleLogin: () => Promise<void>
  githubLogin: () => Promise<void>
  logout: () => Promise<void>
  clearError: () => void
  setUser: (user: User, token: string) => void
  forgotPassword: (email: string) => Promise<boolean>
  initialize: () => Promise<void>
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      // Initial state
      user: null,
      token: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,

      // Initialize auth state from Supabase
      initialize: async () => {
        try {
          const session = await getSession()
          
          if (session) {
            const user = await getUser()
            
            if (user) {
              set({
                user: {
                  id: user.id,
                  email: user.email!,
                  full_name: user.user_metadata?.full_name,
                  business_name: user.user_metadata?.business_name,
                  created_at: user.created_at || new Date().toISOString(),
                },
                token: session.access_token,
                isAuthenticated: true,
                isLoading: false,
              })
            }
          }
        } catch (error) {
          console.error('Auth initialization error:', error)
        }
      },

      // Login action
      login: async (email: string, password: string) => {
        set({ isLoading: true, error: null })

        try {
          const data = await signInWithEmail(email, password)

          if (data.user && data.session) {
            set({
              user: {
                id: data.user.id,
                email: data.user.email!,
                full_name: data.user.user_metadata?.full_name,
                business_name: data.user.user_metadata?.business_name,
                created_at: data.user.created_at || new Date().toISOString(),
              },
              token: data.session.access_token,
              isAuthenticated: true,
              isLoading: false,
              error: null,
            })
            return true
          } else {
            set({
              isLoading: false,
              error: 'Login failed',
            })
            return false
          }
        } catch (error) {
          set({
            isLoading: false,
            error: error instanceof Error ? error.message : 'Login failed',
          })
          return false
        }
      },

      // Register action
      register: async (data) => {
        set({ isLoading: true, error: null })

        try {
          const result = await signUpWithEmail(data.email, data.password, {
            full_name: data.full_name,
            business_name: data.business_name,
          })

          if (result.user && result.session) {
            // Auto-login after registration
            set({
              user: {
                id: result.user.id,
                email: result.user.email!,
                full_name: result.user.user_metadata?.full_name,
                business_name: result.user.user_metadata?.business_name,
                created_at: result.user.created_at || new Date().toISOString(),
              },
              token: result.session.access_token,
              isAuthenticated: true,
              isLoading: false,
              error: null,
            })
            return true
          } else if (result.user) {
            // Email confirmation required
            set({
              isLoading: false,
              error: 'Please check your email to confirm your account',
            })
            return false
          } else {
            set({
              isLoading: false,
              error: 'Registration failed',
            })
            return false
          }
        } catch (error) {
          set({
            isLoading: false,
            error: error instanceof Error ? error.message : 'Registration failed',
          })
          return false
        }
      },

      // Google OAuth Login
      googleLogin: async () => {
        set({ isLoading: true, error: null })

        try {
          await signInWithGoogle()
          // The redirect happens automatically, user will return to /auth/callback
        } catch (error) {
          set({
            isLoading: false,
            error: error instanceof Error ? error.message : 'Google login failed',
          })
        }
      },

      // GitHub OAuth Login
      githubLogin: async () => {
        set({ isLoading: true, error: null })

        try {
          await signInWithGitHub()
          // The redirect happens automatically, user will return to /auth/callback
        } catch (error) {
          set({
            isLoading: false,
            error: error instanceof Error ? error.message : 'GitHub login failed',
          })
        }
      },

      // Logout action
      logout: async () => {
        try {
          await supabaseSignOut()
        } catch (error) {
          console.error('Logout error:', error)
        }
        
        set({
          user: null,
          token: null,
          isAuthenticated: false,
          error: null,
        })
      },

      // Set user (used by callback page)
      setUser: (user: User, token: string) => {
        set({
          user,
          token,
          isAuthenticated: true,
          isLoading: false,
          error: null,
        })
      },

      // Forgot password
      forgotPassword: async (email: string) => {
        set({ isLoading: true, error: null })

        try {
          await sendPasswordResetEmail(email)
          set({ isLoading: false, error: null })
          return true
        } catch (error) {
          set({
            isLoading: false,
            error: error instanceof Error ? error.message : 'Failed to send reset email',
          })
          return false
        }
      },

      // Clear error
      clearError: () => {
        set({ error: null })
      },
    }),
    {
      name: 'auth-storage',
      partialize: (state) => ({
        user: state.user,
        token: state.token,
        isAuthenticated: state.isAuthenticated,
      }),
    }
  )
)

// Set up auth state listener
if (typeof window !== 'undefined') {
  onAuthStateChange((event, session) => {
    console.log('Auth state changed:', event)
    
    if (event === 'SIGNED_OUT') {
      useAuthStore.getState().logout()
    } else if (event === 'SIGNED_IN' && session) {
      // This is handled by the callback page
    }
  })
}
