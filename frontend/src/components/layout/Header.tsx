'use client'

import { Menu, Search, Plus, Sparkles, Settings } from 'lucide-react'
import { useState } from 'react'
import { useRouter, usePathname } from 'next/navigation'
import { NotificationBell } from '@/components/realtime/NotificationBell'
import { AIAssistantPanel } from '@/components/ai/AIAssistantPanel'
import { SettingsModal } from '@/components/ui/SettingsModal'

export function Header() {
  const [searchFocused, setSearchFocused] = useState(false)
  const [aiOpen, setAiOpen] = useState(false)
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [mobileSearchOpen, setMobileSearchOpen] = useState(false)
  const router = useRouter()
  const pathname = usePathname()

  const handleNewSale = () => {
    if (pathname === '/dashboard') {
      const quickSaleSection = document.querySelector('[data-section="quick-sale"]')
      if (quickSaleSection) {
        quickSaleSection.scrollIntoView({ behavior: 'smooth', block: 'center' })
        setTimeout(() => {
          const recorder = document.querySelector('[data-recorder="voice"]') as HTMLElement
          if (recorder) recorder.click()
        }, 500)
      }
    } else {
      router.push('/dashboard')
    }
  }

  return (
    <>
      <header className="bg-white border-b border-slate-200 sticky top-0 z-30">
        <div className="flex items-center justify-between h-16 px-6">
          {/* Left Section */}
          <div className="flex items-center gap-4">
            <button className="md:hidden p-2 hover:bg-slate-100 rounded-xl transition-colors">
              <Menu className="w-5 h-5 text-slate-600" />
            </button>

            {/* Search Bar — desktop */}
            <div className={`relative hidden sm:block transition-all duration-300 ${
              searchFocused ? 'w-80' : 'w-64'
            }`}>
              <Search className={`absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 transition-colors ${
                searchFocused ? 'text-primary-500' : 'text-slate-400'
              }`} />
              <input
                type="text"
                placeholder="Search transactions, products..."
                className="w-full pl-10 pr-4 py-2.5 bg-slate-50 border border-slate-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 focus:bg-white transition-all"
                onFocus={() => setSearchFocused(true)}
                onBlur={() => setSearchFocused(false)}
              />
              <kbd className="absolute right-3 top-1/2 -translate-y-1/2 px-2 py-0.5 text-xs font-semibold text-slate-400 bg-slate-100 rounded hidden lg:block">
                ⌘K
              </kbd>
            </div>

            {/* Mobile search input (shown when icon tapped) */}
            {mobileSearchOpen && (
              <div className="sm:hidden relative flex-1">
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400" />
                <input
                  autoFocus
                  type="text"
                  placeholder="Search..."
                  className="w-full pl-10 pr-4 py-2.5 bg-slate-50 border border-slate-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 focus:bg-white"
                  onBlur={() => setMobileSearchOpen(false)}
                />
              </div>
            )}
          </div>

          {/* Right Section */}
          <div className="flex items-center gap-3">
            {/* New Sale */}
            <button
              onClick={handleNewSale}
              className="hidden sm:flex items-center gap-2 px-4 py-2 bg-primary-600 text-white text-sm font-medium rounded-xl hover:bg-primary-700 transition-colors shadow-glow hover:shadow-lg"
            >
              <Plus className="w-4 h-4" />
              <span>New Sale</span>
            </button>

            {/* AI Assistant */}
            <button
              onClick={() => setAiOpen(true)}
              className="hidden lg:flex items-center gap-2 px-3 py-2 text-sm font-medium text-primary-700 bg-primary-50 rounded-xl hover:bg-primary-100 transition-colors"
            >
              <Sparkles className="w-4 h-4" />
              <span>AI Assistant</span>
            </button>

            {/* Settings */}
            <button
              onClick={() => setSettingsOpen(true)}
              className="p-2.5 text-slate-500 hover:text-slate-700 hover:bg-slate-100 rounded-xl transition-colors"
              title="Settings"
            >
              <Settings className="w-5 h-5" />
            </button>

            {/* Notifications — uses the existing self-contained NotificationBell */}
            <NotificationBell />

            {/* Mobile Search icon */}
            {!mobileSearchOpen && (
              <button
                onClick={() => setMobileSearchOpen(true)}
                className="sm:hidden p-2.5 text-slate-500 hover:text-slate-700 hover:bg-slate-100 rounded-xl transition-colors"
              >
                <Search className="w-5 h-5" />
              </button>
            )}
          </div>
        </div>
      </header>

      {/* Panels rendered outside the header so they can overlay full page */}
      <AIAssistantPanel isOpen={aiOpen} onClose={() => setAiOpen(false)} />
      <SettingsModal isOpen={settingsOpen} onClose={() => setSettingsOpen(false)} />
    </>
  )
}
