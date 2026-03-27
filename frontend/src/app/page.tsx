'use client'

import Link from 'next/link'
import { 
  Mic, 
  BarChart3, 
  Zap, 
  Shield, 
  Globe, 
  Smartphone,
  ArrowRight,
  CheckCircle2,
  Play
} from 'lucide-react'
import { useState } from 'react'

export default function Home() {
  const [isVideoPlaying, setIsVideoPlaying] = useState(false)

  return (
    <div className="min-h-screen bg-slate-50">
      {/* Navigation */}
      <nav className="fixed top-0 left-0 right-0 z-50 glass border-b border-slate-200/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 bg-gradient-to-br from-primary-500 to-primary-700 rounded-xl flex items-center justify-center shadow-glow">
                <Mic className="w-5 h-5 text-white" />
              </div>
              <span className="text-xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-primary-600 to-primary-800">
                AI Merchant
              </span>
            </div>
            
            <div className="hidden md:flex items-center gap-8">
              <a href="#features" className="text-sm font-medium text-slate-600 hover:text-primary-600 transition-colors">Features</a>
              <a href="#how-it-works" className="text-sm font-medium text-slate-600 hover:text-primary-600 transition-colors">How it Works</a>
              <a href="#pricing" className="text-sm font-medium text-slate-600 hover:text-primary-600 transition-colors">Pricing</a>
            </div>

            <div className="flex items-center gap-3">
              <Link href="/auth/login" className="btn-ghost text-sm px-4 py-2">
                Sign in
              </Link>
              <Link href="/dashboard" className="btn-primary text-sm px-4 py-2">
                Get Started
              </Link>
            </div>
          </div>
        </div>
      </nav>

      {/* Hero Section */}
      <section className="relative pt-32 pb-20 lg:pt-40 lg:pb-32 overflow-hidden">
        {/* Background Decorations */}
        <div className="absolute inset-0 -z-10">
          <div className="absolute top-0 right-0 w-[800px] h-[800px] bg-primary-400/20 rounded-full blur-3xl -translate-y-1/2 translate-x-1/3" />
          <div className="absolute bottom-0 left-0 w-[600px] h-[600px] bg-secondary-400/20 rounded-full blur-3xl translate-y-1/3 -translate-x-1/4" />
          <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[1000px] h-[1000px] bg-gradient-to-r from-primary-500/5 to-secondary-500/5 rounded-full blur-3xl" />
        </div>

        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid lg:grid-cols-2 gap-12 lg:gap-8 items-center">
            {/* Hero Content */}
            <div className="text-center lg:text-left animate-fade-in-up">
              <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-primary-100 text-primary-700 text-sm font-semibold mb-6">
                <span className="flex h-2 w-2 rounded-full bg-primary-500 animate-pulse" />
                Now with AI-powered insights
              </div>
              
              <h1 className="text-4xl sm:text-5xl lg:text-6xl font-extrabold text-slate-900 leading-tight mb-6">
                Voice-Driven Business{' '}
                <span className="gradient-text">Intelligence</span>
                {' '}for Modern Merchants
              </h1>
              
              <p className="text-lg sm:text-xl text-slate-600 mb-8 max-w-2xl mx-auto lg:mx-0 leading-relaxed">
                Transform your business operations with AI-powered voice recording, 
                real-time analytics, and intelligent insights designed specifically 
                for small-to-medium merchants.
              </p>

              <div className="flex flex-col sm:flex-row gap-4 justify-center lg:justify-start mb-12">
                <Link 
                  href="/dashboard" 
                  className="btn-primary text-base px-8 py-4 group"
                >
                  Start Free Trial
                  <ArrowRight className="w-4 h-4 group-hover:translate-x-1 transition-transform" />
                </Link>
                <button 
                  onClick={() => setIsVideoPlaying(true)}
                  className="btn-outline text-base px-8 py-4 group"
                >
                  <Play className="w-4 h-4 fill-current" />
                  Watch Demo
                </button>
              </div>

              {/* Trust Badges */}
              <div className="flex flex-wrap items-center gap-6 justify-center lg:justify-start text-sm text-slate-500">
                <div className="flex items-center gap-2">
                  <CheckCircle2 className="w-4 h-4 text-secondary-500" />
                  <span>No credit card required</span>
                </div>
                <div className="flex items-center gap-2">
                  <CheckCircle2 className="w-4 h-4 text-secondary-500" />
                  <span>14-day free trial</span>
                </div>
                <div className="flex items-center gap-2">
                  <CheckCircle2 className="w-4 h-4 text-secondary-500" />
                  <span>Cancel anytime</span>
                </div>
              </div>
            </div>

            {/* Hero Illustration */}
            <div className="relative animate-fade-in animation-delay-200">
              <div className="relative bg-white rounded-3xl shadow-soft-lg p-8 border border-slate-100">
                {/* Mock Dashboard Preview */}
                <div className="space-y-6">
                  {/* Stats Row */}
                  <div className="grid grid-cols-3 gap-4">
                    {[
                      { label: 'Revenue', value: '$24.5k', change: '+12%', positive: true },
                      { label: 'Orders', value: '1,284', change: '+8%', positive: true },
                      { label: 'Growth', value: '23%', change: '+5%', positive: true },
                    ].map((stat, i) => (
                      <div key={i} className="bg-slate-50 rounded-xl p-4">
                        <p className="text-xs text-slate-500 mb-1">{stat.label}</p>
                        <p className="text-lg font-bold text-slate-900">{stat.value}</p>
                        <p className={`text-xs ${stat.positive ? 'text-secondary-500' : 'text-red-500'}`}>
                          {stat.change}
                        </p>
                      </div>
                    ))}
                  </div>
                  
                  {/* Voice Recording Demo */}
                  <div className="bg-gradient-to-br from-primary-500 to-primary-700 rounded-2xl p-6 text-white">
                    <div className="flex items-center gap-4 mb-4">
                      <div className="w-12 h-12 bg-white/20 rounded-full flex items-center justify-center animate-pulse">
                        <Mic className="w-6 h-6" />
                      </div>
                      <div>
                        <p className="font-semibold">Recording...</p>
                        <p className="text-sm text-white/70">"Sold 5 shirts for $125"</p>
                      </div>
                    </div>
                    <div className="h-2 bg-white/20 rounded-full overflow-hidden">
                      <div className="h-full w-2/3 bg-white rounded-full animate-pulse" />
                    </div>
                  </div>

                  {/* Recent Transactions */}
                  <div className="space-y-3">
                    <p className="text-sm font-semibold text-slate-700">Recent Transactions</p>
                    {[
                      { item: 'Product Sale', amount: '$125.00', time: '2 min ago' },
                      { item: 'Bulk Order', amount: '$450.00', time: '15 min ago' },
                      { item: 'Refund', amount: '-$35.00', time: '1 hour ago' },
                    ].map((tx, i) => (
                      <div key={i} className="flex items-center justify-between p-3 bg-slate-50 rounded-xl">
                        <div className="flex items-center gap-3">
                          <div className="w-8 h-8 bg-secondary-100 rounded-lg flex items-center justify-center">
                            <BarChart3 className="w-4 h-4 text-secondary-600" />
                          </div>
                          <div>
                            <p className="text-sm font-medium text-slate-900">{tx.item}</p>
                            <p className="text-xs text-slate-500">{tx.time}</p>
                          </div>
                        </div>
                        <p className="text-sm font-semibold text-slate-900">{tx.amount}</p>
                      </div>
                    ))}
                  </div>
                </div>
              </div>

              {/* Floating Elements */}
              <div className="absolute -top-4 -right-4 bg-white rounded-2xl shadow-soft-lg p-4 animate-float">
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 bg-accent-100 rounded-full flex items-center justify-center">
                    <Zap className="w-5 h-5 text-accent-600" />
                  </div>
                  <div>
                    <p className="text-xs text-slate-500">AI Insight</p>
                    <p className="text-sm font-semibold text-slate-900">Sales up 23%</p>
                  </div>
                </div>
              </div>

              <div className="absolute -bottom-4 -left-4 bg-white rounded-2xl shadow-soft-lg p-4 animate-float animation-delay-500">
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 bg-secondary-100 rounded-full flex items-center justify-center">
                    <CheckCircle2 className="w-5 h-5 text-secondary-600" />
                  </div>
                  <div>
                    <p className="text-xs text-slate-500">Transaction</p>
                    <p className="text-sm font-semibold text-slate-900">Recorded!</p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section id="features" className="py-24 bg-white">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center max-w-3xl mx-auto mb-16">
            <p className="text-primary-600 font-semibold mb-3">Features</p>
            <h2 className="text-3xl sm:text-4xl font-bold text-slate-900 mb-4">
              Everything You Need to Run Your Business
            </h2>
            <p className="text-lg text-slate-600">
              Powerful tools designed to save you time and help you make better decisions.
            </p>
          </div>

          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
            {[
              {
                icon: Mic,
                title: 'Voice Recording',
                description: 'Record transactions hands-free with natural language. Just speak and let AI handle the rest.',
                color: 'primary',
              },
              {
                icon: BarChart3,
                title: 'AI Analytics',
                description: 'Get intelligent insights and predictions about your sales trends, inventory, and customer behavior.',
                color: 'secondary',
              },
              {
                icon: Zap,
                title: 'Real-time Updates',
                description: 'See your data update instantly across all devices. Stay informed wherever you are.',
                color: 'accent',
              },
              {
                icon: Shield,
                title: 'Secure & Private',
                description: 'Enterprise-grade security with end-to-end encryption. Your business data stays safe.',
                color: 'primary',
              },
              {
                icon: Globe,
                title: 'Multi-language Support',
                description: 'Works in 50+ languages. Perfect for diverse teams and international operations.',
                color: 'secondary',
              },
              {
                icon: Smartphone,
                title: 'Mobile First',
                description: 'Optimized for mobile devices. Manage your business on the go with our responsive app.',
                color: 'accent',
              },
            ].map((feature, i) => (
              <div 
                key={i} 
                className="group card-hover border border-slate-100"
              >
                <div className={`w-14 h-14 rounded-2xl bg-${feature.color}-100 flex items-center justify-center mb-5 group-hover:scale-110 transition-transform`}>
                  <feature.icon className={`w-7 h-7 text-${feature.color}-600`} />
                </div>
                <h3 className="text-xl font-bold text-slate-900 mb-3">{feature.title}</h3>
                <p className="text-slate-600 leading-relaxed">{feature.description}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* How It Works */}
      <section id="how-it-works" className="py-24 bg-slate-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center max-w-3xl mx-auto mb-16">
            <p className="text-primary-600 font-semibold mb-3">How It Works</p>
            <h2 className="text-3xl sm:text-4xl font-bold text-slate-900 mb-4">
              Get Started in Three Simple Steps
            </h2>
          </div>

          <div className="grid md:grid-cols-3 gap-8">
            {[
              {
                step: '01',
                title: 'Record Your Sale',
                description: 'Simply speak your transaction details. "Sold 3 t-shirts for $45 each."',
              },
              {
                step: '02',
                title: 'AI Processes Data',
                description: 'Our AI automatically extracts product info, quantities, and prices.',
              },
              {
                step: '03',
                title: 'Get Insights',
                description: 'View real-time analytics and receive AI-powered business recommendations.',
              },
            ].map((item, i) => (
              <div key={i} className="relative">
                <div className="card border border-slate-100 relative z-10">
                  <span className="text-5xl font-bold text-slate-200">{item.step}</span>
                  <h3 className="text-xl font-bold text-slate-900 mt-4 mb-3">{item.title}</h3>
                  <p className="text-slate-600">{item.description}</p>
                </div>
                {i < 2 && (
                  <div className="hidden md:block absolute top-1/2 left-full w-full h-0.5 bg-gradient-to-r from-primary-300 to-transparent -translate-y-1/2 z-0" />
                )}
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-24">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="bg-gradient-to-br from-primary-600 via-primary-700 to-primary-800 rounded-3xl p-12 text-center text-white relative overflow-hidden">
            {/* Background decoration */}
            <div className="absolute inset-0 -z-10">
              <div className="absolute top-0 right-0 w-64 h-64 bg-white/10 rounded-full blur-3xl" />
              <div className="absolute bottom-0 left-0 w-64 h-64 bg-white/10 rounded-full blur-3xl" />
            </div>

            <h2 className="text-3xl sm:text-4xl font-bold mb-4">
              Ready to Transform Your Business?
            </h2>
            <p className="text-lg text-primary-100 mb-8 max-w-2xl mx-auto">
              Join thousands of merchants who are saving hours every day with AI-powered voice recording and analytics.
            </p>
            <div className="flex flex-col sm:flex-row gap-4 justify-center">
              <Link 
                href="/dashboard" 
                className="inline-flex items-center justify-center gap-2 px-8 py-4 bg-white text-primary-700 font-semibold rounded-xl hover:bg-primary-50 transition-colors"
              >
                Get Started Free
                <ArrowRight className="w-4 h-4" />
              </Link>
              <Link 
                href="/auth/login" 
                className="inline-flex items-center justify-center gap-2 px-8 py-4 bg-primary-600 text-white font-semibold rounded-xl border-2 border-white/30 hover:bg-primary-500 transition-colors"
              >
                Sign In
              </Link>
            </div>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="bg-slate-900 text-slate-400 py-12">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid md:grid-cols-4 gap-8 mb-8">
            <div className="col-span-2">
              <div className="flex items-center gap-3 mb-4">
                <div className="w-10 h-10 bg-gradient-to-br from-primary-500 to-primary-700 rounded-xl flex items-center justify-center">
                  <Mic className="w-5 h-5 text-white" />
                </div>
                <span className="text-xl font-bold text-white">
                  AI Merchant
                </span>
              </div>
              <p className="max-w-sm">
                Voice-driven business intelligence for modern merchants. 
                Save time, gain insights, grow your business.
              </p>
            </div>
            <div>
              <h4 className="text-white font-semibold mb-4">Product</h4>
              <ul className="space-y-2">
                <li><a href="#" className="hover:text-white transition-colors">Features</a></li>
                <li><a href="#" className="hover:text-white transition-colors">Pricing</a></li>
                <li><a href="#" className="hover:text-white transition-colors">API</a></li>
              </ul>
            </div>
            <div>
              <h4 className="text-white font-semibold mb-4">Support</h4>
              <ul className="space-y-2">
                <li><a href="#" className="hover:text-white transition-colors">Help Center</a></li>
                <li><a href="#" className="hover:text-white transition-colors">Contact</a></li>
                <li><a href="#" className="hover:text-white transition-colors">Privacy</a></li>
              </ul>
            </div>
          </div>
          <div className="pt-8 border-t border-slate-800 flex flex-col md:flex-row justify-between items-center gap-4">
            <p>© 2024 AI Merchant Assistant. All rights reserved.</p>
            <div className="flex gap-6">
              <a href="#" className="hover:text-white transition-colors">Terms</a>
              <a href="#" className="hover:text-white transition-colors">Privacy</a>
              <a href="#" className="hover:text-white transition-colors">Cookies</a>
            </div>
          </div>
        </div>
      </footer>
    </div>
  )
}
