'use client'

import { useState } from 'react'
import Link from 'next/link'
import { 
  Search, 
  Mic, 
  ShoppingCart, 
  Package, 
  BarChart3, 
  Bell,
  MessageCircle,
  Mail,
  Phone,
  ChevronDown,
  ChevronUp,
  ArrowLeft,
  BookOpen,
  Video,
  FileText
} from 'lucide-react'

interface FAQItem {
  question: string
  answer: string
}

const faqs: FAQItem[] = [
  {
    question: 'How do I record a sale using voice?',
    answer: 'Go to the Dashboard and click the microphone button. Speak naturally about your sale, for example: "Sold 3 shirts for $45 each" or "Customer bought 2 caps, total $50". The AI will automatically understand and record the transaction.'
  },
  {
    question: 'Can I add products manually?',
    answer: 'Yes! Go to the Products page and click "Add Product". Fill in the product details including name, price, stock quantity, and low stock threshold. You can also edit products anytime.'
  },
  {
    question: 'How do I know when stock is running low?',
    answer: 'The system automatically monitors your stock levels. When a product reaches its low stock threshold, you\'ll receive an alert in the Alerts page and via email if enabled in your notification settings.'
  },
  {
    question: 'Can I export my transaction data?',
    answer: 'Yes, go to the Transactions page and click the "Export" button. You can download all your transactions as a CSV file for use in Excel or other accounting software.'
  },
  {
    question: 'How accurate is the voice recognition?',
    answer: 'Our AI uses advanced speech recognition optimized for sales vocabulary. It works best with clear speech in a quiet environment. You can always edit transactions if the AI makes a mistake.'
  },
  {
    question: 'Is my data secure?',
    answer: 'Absolutely. We use industry-standard encryption and security practices. Your data is stored securely in the cloud and never shared with third parties. We use Supabase for authentication and database security.'
  }
]

const guides = [
  {
    title: 'Getting Started',
    description: 'Learn the basics of using AI Merchant Assistant',
    icon: BookOpen,
    color: 'blue'
  },
  {
    title: 'Voice Commands',
    description: 'Tips for effective voice recording',
    icon: Mic,
    color: 'primary'
  },
  {
    title: 'Video Tutorials',
    description: 'Watch step-by-step video guides',
    icon: Video,
    color: 'green'
  },
  {
    title: 'API Documentation',
    description: 'Technical docs for developers',
    icon: FileText,
    color: 'purple'
  }
]

function FAQ({ item }: { item: FAQItem }) {
  const [isOpen, setIsOpen] = useState(false)

  return (
    <div className="border-b border-slate-200 last:border-0">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="w-full flex items-center justify-between py-4 text-left"
      >
        <span className="font-medium text-slate-900">{item.question}</span>
        {isOpen ? (
          <ChevronUp className="w-5 h-5 text-slate-400 flex-shrink-0" />
        ) : (
          <ChevronDown className="w-5 h-5 text-slate-400 flex-shrink-0" />
        )}
      </button>
      {isOpen && (
        <p className="pb-4 text-slate-600 leading-relaxed">
          {item.answer}
        </p>
      )}
    </div>
  )
}

export default function HelpPage() {
  const [searchQuery, setSearchQuery] = useState('')

  const filteredFaqs = faqs.filter(faq =>
    faq.question.toLowerCase().includes(searchQuery.toLowerCase()) ||
    faq.answer.toLowerCase().includes(searchQuery.toLowerCase())
  )

  return (
    <div className="min-h-screen bg-slate-50">
      {/* Header */}
      <header className="bg-white border-b border-slate-200">
        <div className="max-w-5xl mx-auto px-6 py-4">
          <Link 
            href="/dashboard"
            className="inline-flex items-center gap-2 text-slate-600 hover:text-slate-900 transition-colors"
          >
            <ArrowLeft className="w-4 h-4" />
            Back to Dashboard
          </Link>
        </div>
      </header>

      <div className="max-w-5xl mx-auto px-6 py-12">
        {/* Hero */}
        <div className="text-center mb-12">
          <h1 className="text-4xl font-bold text-slate-900 mb-4">
            How can we help you?
          </h1>
          <p className="text-lg text-slate-500 mb-8">
            Search our help center or browse the categories below
          </p>

          {/* Search */}
          <div className="max-w-2xl mx-auto relative">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-slate-400" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search for help..."
              className="w-full pl-12 pr-4 py-4 text-lg border border-slate-200 rounded-xl focus:ring-2 focus:ring-primary-500 focus:border-transparent shadow-sm"
            />
          </div>
        </div>

        {/* Guides Grid */}
        {!searchQuery && (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-12">
            {guides.map((guide) => {
              const Icon = guide.icon
              return (
                <button
                  key={guide.title}
                  className="bg-white p-6 rounded-xl border border-slate-200 hover:border-primary-300 hover:shadow-md transition-all text-left"
                >
                  <div className={`w-12 h-12 bg-${guide.color}-100 rounded-xl flex items-center justify-center mb-4`}>
                    <Icon className={`w-6 h-6 text-${guide.color}-600`} />
                  </div>
                  <h3 className="font-semibold text-slate-900 mb-1">{guide.title}</h3>
                  <p className="text-sm text-slate-500">{guide.description}</p>
                </button>
              )
            })}
          </div>
        )}

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          {/* FAQ Section */}
          <div className="lg:col-span-2">
            <div className="bg-white rounded-xl border border-slate-200 p-6">
              <h2 className="text-xl font-bold text-slate-900 mb-6">
                {searchQuery ? 'Search Results' : 'Frequently Asked Questions'}
              </h2>

              {filteredFaqs.length > 0 ? (
                <div className="divide-y divide-slate-200">
                  {filteredFaqs.map((faq, index) => (
                    <FAQ key={index} item={faq} />
                  ))}
                </div>
              ) : (
                <div className="text-center py-12">
                  <p className="text-slate-500">No results found for &quot;{searchQuery}&quot;</p>
                  <button 
                    onClick={() => setSearchQuery('')}
                    className="text-primary-600 hover:text-primary-700 font-medium mt-2"
                  >
                    Clear search
                  </button>
                </div>
              )}
            </div>

            {/* Quick Links */}
            {!searchQuery && (
              <div className="mt-6 grid grid-cols-1 sm:grid-cols-2 gap-4">
                <Link 
                  href="/dashboard"
                  className="bg-white p-4 rounded-xl border border-slate-200 hover:border-primary-300 transition-colors flex items-center gap-4"
                >
                  <div className="w-10 h-10 bg-primary-100 rounded-lg flex items-center justify-center">
                    <Mic className="w-5 h-5 text-primary-600" />
                  </div>
                  <div>
                    <p className="font-medium text-slate-900">Record a Sale</p>
                    <p className="text-sm text-slate-500">Use voice to log transactions</p>
                  </div>
                </Link>

                <Link 
                  href="/dashboard/products"
                  className="bg-white p-4 rounded-xl border border-slate-200 hover:border-primary-300 transition-colors flex items-center gap-4"
                >
                  <div className="w-10 h-10 bg-secondary-100 rounded-lg flex items-center justify-center">
                    <Package className="w-5 h-5 text-secondary-600" />
                  </div>
                  <div>
                    <p className="font-medium text-slate-900">Manage Products</p>
                    <p className="text-sm text-slate-500">Add or edit your inventory</p>
                  </div>
                </Link>

                <Link 
                  href="/dashboard/transactions"
                  className="bg-white p-4 rounded-xl border border-slate-200 hover:border-primary-300 transition-colors flex items-center gap-4"
                >
                  <div className="w-10 h-10 bg-accent-100 rounded-lg flex items-center justify-center">
                    <ShoppingCart className="w-5 h-5 text-accent-600" />
                  </div>
                  <div>
                    <p className="font-medium text-slate-900">View Transactions</p>
                    <p className="text-sm text-slate-500">See all your sales history</p>
                  </div>
                </Link>

                <Link 
                  href="/dashboard/analytics"
                  className="bg-white p-4 rounded-xl border border-slate-200 hover:border-primary-300 transition-colors flex items-center gap-4"
                >
                  <div className="w-10 h-10 bg-purple-100 rounded-lg flex items-center justify-center">
                    <BarChart3 className="w-5 h-5 text-purple-600" />
                  </div>
                  <div>
                    <p className="font-medium text-slate-900">View Analytics</p>
                    <p className="text-sm text-slate-500">Insights and reports</p>
                  </div>
                </Link>
              </div>
            )}
          </div>

          {/* Contact Section */}
          <div className="space-y-6">
            <div className="bg-white rounded-xl border border-slate-200 p-6">
              <h3 className="font-semibold text-slate-900 mb-4">Contact Support</h3>
              <div className="space-y-4">
                <button className="w-full flex items-center gap-3 p-3 rounded-lg hover:bg-slate-50 transition-colors text-left">
                  <div className="w-10 h-10 bg-primary-100 rounded-lg flex items-center justify-center">
                    <MessageCircle className="w-5 h-5 text-primary-600" />
                  </div>
                  <div>
                    <p className="font-medium text-slate-900">Live Chat</p>
                    <p className="text-sm text-slate-500">Available 9am - 5pm</p>
                  </div>
                </button>

                <button className="w-full flex items-center gap-3 p-3 rounded-lg hover:bg-slate-50 transition-colors text-left">
                  <div className="w-10 h-10 bg-secondary-100 rounded-lg flex items-center justify-center">
                    <Mail className="w-5 h-5 text-secondary-600" />
                  </div>
                  <div>
                    <p className="font-medium text-slate-900">Email Us</p>
                    <p className="text-sm text-slate-500">support@aimerchant.com</p>
                  </div>
                </button>

                <button className="w-full flex items-center gap-3 p-3 rounded-lg hover:bg-slate-50 transition-colors text-left">
                  <div className="w-10 h-10 bg-accent-100 rounded-lg flex items-center justify-center">
                    <Phone className="w-5 h-5 text-accent-600" />
                  </div>
                  <div>
                    <p className="font-medium text-slate-900">Call Us</p>
                    <p className="text-sm text-slate-500">+1 (555) 123-4567</p>
                  </div>
                </button>
              </div>
            </div>

            <div className="bg-gradient-to-br from-primary-500 to-primary-700 rounded-xl p-6 text-white">
              <h3 className="font-semibold mb-2">Still need help?</h3>
              <p className="text-primary-100 text-sm mb-4">
                Our support team is always ready to assist you with any questions.
              </p>
              <button className="w-full py-2 bg-white text-primary-600 rounded-lg font-medium hover:bg-primary-50 transition-colors">
                Contact Support
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Footer */}
      <footer className="border-t border-slate-200 bg-white mt-12">
        <div className="max-w-5xl mx-auto px-6 py-8">
          <div className="flex flex-col md:flex-row items-center justify-between gap-4">
            <p className="text-slate-500 text-sm">
              © 2026 AI Merchant Assistant. All rights reserved.
            </p>
            <div className="flex items-center gap-6">
              <Link href="#" className="text-slate-500 hover:text-slate-700 text-sm">Privacy Policy</Link>
              <Link href="#" className="text-slate-500 hover:text-slate-700 text-sm">Terms of Service</Link>
            </div>
          </div>
        </div>
      </footer>
    </div>
  )
}
