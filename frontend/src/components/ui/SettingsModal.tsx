'use client'

import { useState } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { X, Bell, Globe, Shield, Database, ChevronRight } from 'lucide-react'

interface SettingsModalProps {
  isOpen: boolean
  onClose: () => void
}

export function SettingsModal({ isOpen, onClose }: SettingsModalProps) {
  const [notifications, setNotifications] = useState(true)
  const [soundAlerts, setSoundAlerts] = useState(false)
  const [language, setLanguage] = useState('en')

  return (
    <AnimatePresence>
      {isOpen && (
        <>
          {/* Backdrop */}
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 bg-black/40 z-50"
            onClick={onClose}
          />

          {/* Modal */}
          <motion.div
            initial={{ opacity: 0, scale: 0.95, y: -10 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: -10 }}
            transition={{ type: 'spring', stiffness: 400, damping: 30 }}
            className="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-md bg-white rounded-2xl shadow-2xl z-50 overflow-hidden"
          >
            {/* Header */}
            <div className="flex items-center justify-between px-6 py-4 border-b border-slate-100">
              <h2 className="text-base font-semibold text-slate-900">Settings</h2>
              <button
                onClick={onClose}
                className="p-2 hover:bg-slate-100 rounded-xl transition-colors text-slate-500"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="p-6 space-y-6">
              {/* Notifications */}
              <section>
                <div className="flex items-center gap-2 mb-3">
                  <Bell className="w-4 h-4 text-slate-400" />
                  <h3 className="text-sm font-semibold text-slate-700">Notifications</h3>
                </div>
                <div className="space-y-3">
                  <label className="flex items-center justify-between cursor-pointer group">
                    <div>
                      <p className="text-sm text-slate-800 font-medium">Push Notifications</p>
                      <p className="text-xs text-slate-500">Alerts for low stock and new sales</p>
                    </div>
                    <button
                      onClick={() => setNotifications(!notifications)}
                      className={`relative w-10 h-6 rounded-full transition-colors ${notifications ? 'bg-primary-600' : 'bg-slate-200'}`}
                    >
                      <span className={`absolute top-1 w-4 h-4 bg-white rounded-full shadow transition-transform ${notifications ? 'translate-x-5' : 'translate-x-1'}`} />
                    </button>
                  </label>
                  <label className="flex items-center justify-between cursor-pointer group">
                    <div>
                      <p className="text-sm text-slate-800 font-medium">Sound Alerts</p>
                      <p className="text-xs text-slate-500">Play a sound for critical alerts</p>
                    </div>
                    <button
                      onClick={() => setSoundAlerts(!soundAlerts)}
                      className={`relative w-10 h-6 rounded-full transition-colors ${soundAlerts ? 'bg-primary-600' : 'bg-slate-200'}`}
                    >
                      <span className={`absolute top-1 w-4 h-4 bg-white rounded-full shadow transition-transform ${soundAlerts ? 'translate-x-5' : 'translate-x-1'}`} />
                    </button>
                  </label>
                </div>
              </section>

              {/* Language */}
              <section>
                <div className="flex items-center gap-2 mb-3">
                  <Globe className="w-4 h-4 text-slate-400" />
                  <h3 className="text-sm font-semibold text-slate-700">Language & Region</h3>
                </div>
                <select
                  value={language}
                  onChange={e => setLanguage(e.target.value)}
                  className="w-full px-3 py-2 border border-slate-200 rounded-xl text-sm text-slate-800 bg-slate-50 focus:outline-none focus:ring-2 focus:ring-primary-500/20 focus:border-primary-400"
                >
                  <option value="en">English (US)</option>
                  <option value="en-gb">English (UK)</option>
                  <option value="es">Español</option>
                  <option value="fr">Français</option>
                  <option value="de">Deutsch</option>
                  <option value="sw">Swahili</option>
                  <option value="ar">العربية</option>
                </select>
              </section>

              {/* Data & Privacy */}
              <section>
                <div className="flex items-center gap-2 mb-3">
                  <Shield className="w-4 h-4 text-slate-400" />
                  <h3 className="text-sm font-semibold text-slate-700">Data & Privacy</h3>
                </div>
                <div className="space-y-1">
                  {[
                    { label: 'Export My Data', sub: 'Download all your transaction data' },
                    { label: 'Privacy Policy', sub: 'How we handle your data' },
                  ].map(({ label, sub }) => (
                    <button
                      key={label}
                      className="w-full flex items-center justify-between px-3 py-2.5 rounded-xl hover:bg-slate-50 transition-colors text-left group"
                    >
                      <div>
                        <p className="text-sm text-slate-800 font-medium">{label}</p>
                        <p className="text-xs text-slate-500">{sub}</p>
                      </div>
                      <ChevronRight className="w-4 h-4 text-slate-400 group-hover:text-slate-600" />
                    </button>
                  ))}
                </div>
              </section>

              {/* App Info */}
              <section className="pt-2 border-t border-slate-100">
                <div className="flex items-center gap-2 mb-2">
                  <Database className="w-4 h-4 text-slate-400" />
                  <h3 className="text-sm font-semibold text-slate-700">App Info</h3>
                </div>
                <div className="text-xs text-slate-500 space-y-1 px-1">
                  <p>AI Merchant Assistant v1.0.0</p>
                  <p>Backend: Rust / Axum · AI: Llama 3.1 + Whisper + MeloTTS + LLaVA</p>
                </div>
              </section>
            </div>

            <div className="px-6 pb-5">
              <button
                onClick={onClose}
                className="w-full py-2.5 bg-primary-600 text-white text-sm font-medium rounded-xl hover:bg-primary-700 transition-colors"
              >
                Save & Close
              </button>
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  )
}
