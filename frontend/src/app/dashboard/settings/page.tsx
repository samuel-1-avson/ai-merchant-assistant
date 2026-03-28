'use client'

import { useState, useEffect } from 'react'
import { Layout } from '@/components/layout/Layout'
import { useAuthStore } from '@/stores/authStore'
import { useToast } from '@/components/ui/Toast'
import { 
  User, 
  Store, 
  Bell, 
  Shield, 
  CreditCard,
  Save,
  Loader2,
  CheckCircle
} from 'lucide-react'

type Tab = 'profile' | 'business' | 'notifications' | 'security' | 'billing'

export default function SettingsPage() {
  const { user } = useAuthStore()
  const { success, error } = useToast()
  const [activeTab, setActiveTab] = useState<Tab>('profile')
  const [isSaving, setIsSaving] = useState(false)
  
  // Profile form
  const [profile, setProfile] = useState({
    full_name: '',
    email: '',
    phone: '',
  })
  
  // Business form
  const [business, setBusiness] = useState({
    business_name: '',
    business_address: '',
    business_phone: '',
    currency: 'USD',
    tax_rate: '',
  })
  
  // Notifications form
  const [notifications, setNotifications] = useState({
    email_alerts: true,
    low_stock_alerts: true,
    daily_summary: false,
    new_features: true,
  })

  useEffect(() => {
    if (user) {
      setProfile({
        full_name: user.full_name || '',
        email: user.email || '',
        phone: '',
      })
      setBusiness(prev => ({
        ...prev,
        business_name: user.business_name || '',
      }))
    }
  }, [user])

  const handleSave = async () => {
    setIsSaving(true)
    
    // Simulate API call
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    success('Settings saved successfully')
    setIsSaving(false)
  }

  const tabs = [
    { id: 'profile' as Tab, label: 'Profile', icon: User },
    { id: 'business' as Tab, label: 'Business', icon: Store },
    { id: 'notifications' as Tab, label: 'Notifications', icon: Bell },
    { id: 'security' as Tab, label: 'Security', icon: Shield },
    { id: 'billing' as Tab, label: 'Billing', icon: CreditCard },
  ]

  return (
    <Layout>
      <div className="space-y-6">
        {/* Header */}
        <div>
          <h1 className="text-2xl font-bold text-slate-900">Settings</h1>
          <p className="text-slate-500 mt-1">
            Manage your account and business preferences
          </p>
        </div>

        <div className="flex flex-col lg:flex-row gap-6">
          {/* Sidebar Tabs */}
          <div className="lg:w-64 flex-shrink-0">
            <div className="bg-white rounded-xl border border-slate-200 overflow-hidden">
              {tabs.map((tab) => {
                const Icon = tab.icon
                return (
                  <button
                    key={tab.id}
                    onClick={() => setActiveTab(tab.id)}
                    className={`w-full flex items-center gap-3 px-4 py-3 text-left transition-colors ${
                      activeTab === tab.id
                        ? 'bg-primary-50 text-primary-700 border-l-4 border-primary-500'
                        : 'text-slate-600 hover:bg-slate-50 border-l-4 border-transparent'
                    }`}
                  >
                    <Icon className="w-5 h-5" />
                    <span className="font-medium">{tab.label}</span>
                  </button>
                )
              })}
            </div>
          </div>

          {/* Content */}
          <div className="flex-1">
            <div className="bg-white rounded-xl border border-slate-200 p-6">
              {/* Profile Tab */}
              {activeTab === 'profile' && (
                <div className="space-y-6">
                  <div>
                    <h2 className="text-lg font-semibold text-slate-900">Profile Information</h2>
                    <p className="text-slate-500">Update your personal details</p>
                  </div>

                  <div className="space-y-4">
                    <div>
                      <label className="block text-sm font-medium text-slate-700 mb-1">
                        Full Name
                      </label>
                      <input
                        type="text"
                        value={profile.full_name}
                        onChange={(e) => setProfile({ ...profile, full_name: e.target.value })}
                        className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                      />
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-slate-700 mb-1">
                        Email Address
                      </label>
                      <input
                        type="email"
                        value={profile.email}
                        disabled
                        className="w-full px-4 py-2 border border-slate-200 rounded-lg bg-slate-50 text-slate-500"
                      />
                      <p className="text-xs text-slate-500 mt-1">Email cannot be changed</p>
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-slate-700 mb-1">
                        Phone Number
                      </label>
                      <input
                        type="tel"
                        value={profile.phone}
                        onChange={(e) => setProfile({ ...profile, phone: e.target.value })}
                        className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                        placeholder="+1 (555) 123-4567"
                      />
                    </div>
                  </div>
                </div>
              )}

              {/* Business Tab */}
              {activeTab === 'business' && (
                <div className="space-y-6">
                  <div>
                    <h2 className="text-lg font-semibold text-slate-900">Business Information</h2>
                    <p className="text-slate-500">Manage your business details</p>
                  </div>

                  <div className="space-y-4">
                    <div>
                      <label className="block text-sm font-medium text-slate-700 mb-1">
                        Business Name
                      </label>
                      <input
                        type="text"
                        value={business.business_name}
                        onChange={(e) => setBusiness({ ...business, business_name: e.target.value })}
                        className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                      />
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-slate-700 mb-1">
                        Business Address
                      </label>
                      <textarea
                        value={business.business_address}
                        onChange={(e) => setBusiness({ ...business, business_address: e.target.value })}
                        rows={3}
                        className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                      />
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-slate-700 mb-1">
                        Business Phone
                      </label>
                      <input
                        type="tel"
                        value={business.business_phone}
                        onChange={(e) => setBusiness({ ...business, business_phone: e.target.value })}
                        className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                      />
                    </div>

                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <label className="block text-sm font-medium text-slate-700 mb-1">
                          Currency
                        </label>
                        <select
                          value={business.currency}
                          onChange={(e) => setBusiness({ ...business, currency: e.target.value })}
                          className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                        >
                          <option value="USD">USD - US Dollar</option>
                          <option value="EUR">EUR - Euro</option>
                          <option value="GBP">GBP - British Pound</option>
                          <option value="IDR">IDR - Indonesian Rupiah</option>
                        </select>
                      </div>
                      <div>
                        <label className="block text-sm font-medium text-slate-700 mb-1">
                          Tax Rate (%)
                        </label>
                        <input
                          type="number"
                          step="0.01"
                          value={business.tax_rate}
                          onChange={(e) => setBusiness({ ...business, tax_rate: e.target.value })}
                          className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                          placeholder="0.00"
                        />
                      </div>
                    </div>
                  </div>
                </div>
              )}

              {/* Notifications Tab */}
              {activeTab === 'notifications' && (
                <div className="space-y-6">
                  <div>
                    <h2 className="text-lg font-semibold text-slate-900">Notification Preferences</h2>
                    <p className="text-slate-500">Choose what notifications you receive</p>
                  </div>

                  <div className="space-y-4">
                    {[
                      { key: 'email_alerts', label: 'Email Alerts', description: 'Receive important alerts via email' },
                      { key: 'low_stock_alerts', label: 'Low Stock Alerts', description: 'Get notified when products are running low' },
                      { key: 'daily_summary', label: 'Daily Summary', description: 'Receive a daily summary of your sales' },
                      { key: 'new_features', label: 'New Features', description: 'Be the first to know about new features' },
                    ].map((item) => (
                      <div 
                        key={item.key}
                        className="flex items-center justify-between py-3 border-b border-slate-100 last:border-0"
                      >
                        <div>
                          <p className="font-medium text-slate-900">{item.label}</p>
                          <p className="text-sm text-slate-500">{item.description}</p>
                        </div>
                        <button
                          onClick={() => setNotifications(prev => ({
                            ...prev,
                            [item.key]: !prev[item.key as keyof typeof notifications]
                          }))}
                          className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                            notifications[item.key as keyof typeof notifications]
                              ? 'bg-primary-600'
                              : 'bg-slate-200'
                          }`}
                        >
                          <span
                            className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                              notifications[item.key as keyof typeof notifications]
                                ? 'translate-x-6'
                                : 'translate-x-1'
                            }`}
                          />
                        </button>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Security Tab */}
              {activeTab === 'security' && (
                <div className="space-y-6">
                  <div>
                    <h2 className="text-lg font-semibold text-slate-900">Security</h2>
                    <p className="text-slate-500">Manage your account security</p>
                  </div>

                  <div className="space-y-4">
                    <div className="p-4 bg-slate-50 rounded-lg">
                      <h3 className="font-medium text-slate-900 mb-2">Change Password</h3>
                      <div className="space-y-3">
                        <input
                          type="password"
                          placeholder="Current Password"
                          className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                        />
                        <input
                          type="password"
                          placeholder="New Password"
                          className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                        />
                        <input
                          type="password"
                          placeholder="Confirm New Password"
                          className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                        />
                        <button className="btn-outline text-sm">
                          Update Password
                        </button>
                      </div>
                    </div>

                    <div className="p-4 bg-slate-50 rounded-lg">
                      <h3 className="font-medium text-slate-900 mb-2">Two-Factor Authentication</h3>
                      <p className="text-sm text-slate-500 mb-3">
                        Add an extra layer of security to your account
                      </p>
                      <button className="btn-outline text-sm">
                        Enable 2FA
                      </button>
                    </div>

                    <div className="p-4 bg-red-50 rounded-lg border border-red-200">
                      <h3 className="font-medium text-red-900 mb-2">Danger Zone</h3>
                      <p className="text-sm text-red-600 mb-3">
                        Once you delete your account, there is no going back.
                      </p>
                      <button className="text-red-600 hover:text-red-700 font-medium text-sm">
                        Delete Account
                      </button>
                    </div>
                  </div>
                </div>
              )}

              {/* Billing Tab */}
              {activeTab === 'billing' && (
                <div className="space-y-6">
                  <div>
                    <h2 className="text-lg font-semibold text-slate-900">Billing & Subscription</h2>
                    <p className="text-slate-500">Manage your subscription and billing</p>
                  </div>

                  <div className="p-6 bg-primary-50 rounded-xl border border-primary-200">
                    <div className="flex items-center justify-between mb-4">
                      <div>
                        <p className="text-sm text-primary-600 font-medium">Current Plan</p>
                        <p className="text-2xl font-bold text-primary-900">Free Trial</p>
                      </div>
                      <span className="px-3 py-1 bg-primary-200 text-primary-800 rounded-full text-sm font-medium">
                        Active
                      </span>
                    </div>
                    <p className="text-primary-700 mb-4">
                      Your free trial includes unlimited transactions and basic analytics.
                    </p>
                    <button className="btn-primary">
                      Upgrade to Pro
                    </button>
                  </div>

                  <div>
                    <h3 className="font-medium text-slate-900 mb-3">Payment Method</h3>
                    <div className="p-4 border border-slate-200 rounded-lg flex items-center justify-between">
                      <div className="flex items-center gap-3">
                        <div className="w-10 h-6 bg-slate-200 rounded" />
                        <span className="text-slate-500">No payment method added</span>
                      </div>
                      <button className="btn-ghost text-sm">
                        Add Card
                      </button>
                    </div>
                  </div>

                  <div>
                    <h3 className="font-medium text-slate-900 mb-3">Billing History</h3>
                    <div className="text-center py-8 text-slate-500">
                      <p>No billing history yet</p>
                    </div>
                  </div>
                </div>
              )}

              {/* Save Button */}
              {activeTab !== 'billing' && activeTab !== 'security' && (
                <div className="flex items-center justify-end gap-3 pt-6 border-t border-slate-200 mt-6">
                  <button
                    onClick={handleSave}
                    disabled={isSaving}
                    className="btn-primary"
                  >
                    {isSaving ? (
                      <>
                        <Loader2 className="w-4 h-4 animate-spin" />
                        Saving...
                      </>
                    ) : (
                      <>
                        <Save className="w-4 h-4" />
                        Save Changes
                      </>
                    )}
                  </button>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </Layout>
  )
}
