import type { Metadata, Viewport } from 'next'
import { Inter } from 'next/font/google'
import './globals.css'
import { ServiceWorkerRegister } from '@/components/ServiceWorkerRegister'

const inter = Inter({ 
  subsets: ['latin'],
  display: 'swap',
  variable: '--font-inter',
})

export const metadata: Metadata = {
  title: 'AI Merchant Assistant - Voice-Driven Business Intelligence',
  description: 'Transform your business operations with AI-powered voice recording, real-time analytics, and intelligent insights for small-to-medium merchants.',
  keywords: ['voice recording', 'business analytics', 'AI', 'merchant', 'transactions', 'dashboard'],
  manifest: '/manifest.json',
  appleWebApp: {
    capable: true,
    statusBarStyle: 'default',
    title: 'AI Merchant',
  },
  openGraph: {
    title: 'AI Merchant Assistant',
    description: 'Voice-driven business intelligence for modern merchants',
    type: 'website',
  },
}

export const viewport: Viewport = {
  themeColor: [
    { media: '(prefers-color-scheme: light)', color: '#ffffff' },
    { media: '(prefers-color-scheme: dark)', color: '#0f172a' },
  ],
  width: 'device-width',
  initialScale: 1,
  maximumScale: 1,
  userScalable: false,
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en" className={inter.variable}>
      <head>
        <link rel="apple-touch-icon" href="/icons/icon-192x192.png" />
        <link rel="mask-icon" href="/icons/icon-192x192.png" color="#4f46e5" />
        <meta name="apple-mobile-web-app-title" content="AI Merchant" />
      </head>
      <body className={`${inter.className} antialiased`}>
        {children}
        <ServiceWorkerRegister />
      </body>
    </html>
  )
}
