'use client'

import { useEffect, useState } from 'react'
import { useRouter } from 'next/navigation'
import { motion } from 'framer-motion'
import { Loader2, CheckCircle, XCircle } from 'lucide-react'
import { createClient } from '@/lib/supabase/client'
import { useAuthStore } from '@/stores/authStore'

export default function AuthCallbackPage() {
  const router = useRouter()
  const [status, setStatus] = useState<'loading' | 'success' | 'error'>('loading')
  const [message, setMessage] = useState('Completing sign in...')
  const { setUser } = useAuthStore()

  useEffect(() => {
    const handleCallback = async () => {
      try {
        const supabase = createClient()
        
        // Exchange the auth code for a session
        const { data: { session }, error } = await supabase.auth.getSession()
        
        if (error) {
          throw error
        }

        if (session) {
          // Get user details
          const { data: { user } } = await supabase.auth.getUser()
          
          if (user) {
            // Store in our auth store
            setUser({
              id: user.id,
              email: user.email!,
              full_name: user.user_metadata?.full_name,
              business_name: user.user_metadata?.business_name,
              created_at: user.created_at || new Date().toISOString(),
            }, session.access_token)
            
            setStatus('success')
            setMessage('Sign in successful! Redirecting...')
            
            // Redirect to dashboard after a short delay
            setTimeout(() => {
              router.push('/dashboard')
            }, 1500)
          } else {
            throw new Error('User not found')
          }
        } else {
          // Check if there's a hash fragment (OAuth redirect)
          const hash = window.location.hash
          if (hash) {
            // Supabase handles the hash automatically
            const { data: { session }, error } = await supabase.auth.getSession()
            
            if (error) throw error
            
            if (session) {
              const { data: { user } } = await supabase.auth.getUser()
              
              if (user) {
                setUser({
                  id: user.id,
                  email: user.email!,
                  full_name: user.user_metadata?.full_name,
                  business_name: user.user_metadata?.business_name,
                  created_at: user.created_at || new Date().toISOString(),
                }, session.access_token)
                
                setStatus('success')
                setMessage('Sign in successful! Redirecting...')
                
                setTimeout(() => {
                  router.push('/dashboard')
                }, 1500)
                return
              }
            }
          }
          
          throw new Error('No session found')
        }
      } catch (error) {
        console.error('Auth callback error:', error)
        setStatus('error')
        setMessage(error instanceof Error ? error.message : 'Authentication failed')
        
        // Redirect to login after error
        setTimeout(() => {
          router.push('/auth/login?error=auth_failed')
        }, 3000)
      }
    }

    handleCallback()
  }, [router, setUser])

  return (
    <div className="min-h-screen bg-slate-50 flex items-center justify-center p-6">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className="bg-white rounded-3xl shadow-soft-lg p-12 border border-slate-100 text-center max-w-md w-full"
      >
        {status === 'loading' && (
          <>
            <div className="w-16 h-16 bg-primary-100 rounded-full flex items-center justify-center mx-auto mb-6">
              <Loader2 className="w-8 h-8 text-primary-600 animate-spin" />
            </div>
            <h2 className="text-xl font-bold text-slate-900 mb-2">Completing Sign In</h2>
            <p className="text-slate-500">{message}</p>
          </>
        )}

        {status === 'success' && (
          <>
            <div className="w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-6">
              <CheckCircle className="w-8 h-8 text-green-600" />
            </div>
            <h2 className="text-xl font-bold text-slate-900 mb-2">Welcome Back!</h2>
            <p className="text-slate-500">{message}</p>
          </>
        )}

        {status === 'error' && (
          <>
            <div className="w-16 h-16 bg-red-100 rounded-full flex items-center justify-center mx-auto mb-6">
              <XCircle className="w-8 h-8 text-red-600" />
            </div>
            <h2 className="text-xl font-bold text-slate-900 mb-2">Sign In Failed</h2>
            <p className="text-slate-500 mb-4">{message}</p>
            <p className="text-sm text-slate-400">Redirecting to login...</p>
          </>
        )}
      </motion.div>
    </div>
  )
}
