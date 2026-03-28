'use client'

import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { 
  LayoutDashboard, 
  ShoppingCart, 
  Package, 
  BarChart3, 
  Settings,
  Mic,
  Bell,
  LogOut,
  ChevronLeft,
  ChevronRight,
  HelpCircle,
} from 'lucide-react'
import { useState } from 'react'
import { useAuthStore } from '@/stores/authStore'
import { useAlertUpdates } from '@/hooks/useWebSocket'

const navigation = [
  { name: 'Dashboard', href: '/dashboard', icon: LayoutDashboard },
  { name: 'Transactions', href: '/dashboard/transactions', icon: ShoppingCart },
  { name: 'Products', href: '/dashboard/products', icon: Package },
  { name: 'Analytics', href: '/dashboard/analytics', icon: BarChart3 },
  { name: 'Alerts', href: '/dashboard/alerts', icon: Bell },
]

const secondaryNavigation = [
  { name: 'Settings', href: '/dashboard/settings', icon: Settings },
  { name: 'Help & Support', href: '/help', icon: HelpCircle },
]

export function Sidebar() {
  const pathname = usePathname()
  const [isCollapsed, setIsCollapsed] = useState(false)
  const { user, logout } = useAuthStore()
  const { unreadCount } = useAlertUpdates()

  // Get user initials for avatar
  const getInitials = () => {
    if (user?.full_name) {
      return user.full_name.split(' ').map(n => n[0]).join('').toUpperCase().slice(0, 2)
    }
    if (user?.email) {
      return user.email.slice(0, 2).toUpperCase()
    }
    return 'MU'
  }

  return (
    <div className={`hidden md:flex flex-col bg-white border-r border-slate-200 transition-all duration-300 ${
      isCollapsed ? 'w-20' : 'w-64'
    }`}>
      {/* Logo Section */}
      <div className="flex items-center justify-between h-16 px-4 border-b border-slate-100">
        <Link href="/dashboard" className="flex items-center gap-3 overflow-hidden">
          <div className="w-10 h-10 bg-gradient-to-br from-primary-500 to-primary-700 rounded-xl flex items-center justify-center shadow-glow flex-shrink-0">
            <Mic className="w-5 h-5 text-white" />
          </div>
          {!isCollapsed && (
            <span className="text-lg font-bold bg-clip-text text-transparent bg-gradient-to-r from-primary-600 to-primary-800 whitespace-nowrap">
              AI Merchant
            </span>
          )}
        </Link>
        <button 
          onClick={() => setIsCollapsed(!isCollapsed)}
          className="p-1.5 hover:bg-slate-100 rounded-lg transition-colors"
        >
          {isCollapsed ? (
            <ChevronRight className="w-4 h-4 text-slate-400" />
          ) : (
            <ChevronLeft className="w-4 h-4 text-slate-400" />
          )}
        </button>
      </div>

      {/* Main Navigation */}
      <nav className="flex-1 p-3 space-y-1 overflow-y-auto">
        {!isCollapsed && (
          <p className="px-3 py-2 text-xs font-semibold text-slate-400 uppercase tracking-wider">
            Menu
          </p>
        )}
        {navigation.map((item) => {
          const isActive = pathname === item.href || pathname?.startsWith(`${item.href}/`)
          const isAlerts = item.name === 'Alerts'
          
          return (
            <Link
              key={item.name}
              href={item.href}
              className={`
                flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm font-medium transition-all duration-200 group relative
                ${isActive 
                  ? 'bg-primary-50 text-primary-700' 
                  : 'text-slate-600 hover:bg-slate-50 hover:text-slate-900'
                }
              `}
            >
              <div className={`p-1.5 rounded-lg transition-colors ${
                isActive ? 'bg-primary-100' : 'group-hover:bg-slate-200'
              }`}>
                <item.icon className={`w-5 h-5 ${isActive ? 'text-primary-600' : 'text-slate-500'}`} />
              </div>
              {!isCollapsed && (
                <>
                  <span className="flex-1">{item.name}</span>
                  {isAlerts && unreadCount > 0 && (
                    <span className={`px-2 py-0.5 text-xs font-semibold rounded-full ${
                      isActive 
                        ? 'bg-primary-200 text-primary-800' 
                        : 'bg-red-100 text-red-700'
                    }`}>
                      {unreadCount}
                    </span>
                  )}
                </>
              )}
              {isCollapsed && isAlerts && unreadCount > 0 && (
                <span className="absolute top-1 right-1 w-4 h-4 bg-red-500 rounded-full flex items-center justify-center text-[10px] text-white font-semibold">
                  {unreadCount > 9 ? '9+' : unreadCount}
                </span>
              )}
            </Link>
          )
        })}

        {!isCollapsed && (
          <p className="px-3 py-2 mt-6 text-xs font-semibold text-slate-400 uppercase tracking-wider">
            Support
          </p>
        )}
        {secondaryNavigation.map((item) => {
          const isActive = pathname === item.href
          
          return (
            <Link
              key={item.name}
              href={item.href}
              className={`
                flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm font-medium transition-all duration-200 group
                ${isActive 
                  ? 'bg-primary-50 text-primary-700' 
                  : 'text-slate-600 hover:bg-slate-50 hover:text-slate-900'
                }
              `}
            >
              <div className={`p-1.5 rounded-lg transition-colors ${
                isActive ? 'bg-primary-100' : 'group-hover:bg-slate-200'
              }`}>
                <item.icon className={`w-5 h-5 ${isActive ? 'text-primary-600' : 'text-slate-500'}`} />
              </div>
              {!isCollapsed && <span>{item.name}</span>}
            </Link>
          )
        })}
      </nav>

      {/* User Section */}
      <div className="p-3 border-t border-slate-100">
        <div className={`flex items-center gap-3 px-3 py-3 rounded-xl bg-slate-50 ${
          isCollapsed ? 'justify-center' : ''
        }`}>
          <div className="relative">
            <div className="w-10 h-10 rounded-full bg-gradient-to-br from-primary-400 to-primary-600 flex items-center justify-center text-white font-semibold flex-shrink-0">
              {getInitials()}
            </div>
            <span className="absolute -bottom-0.5 -right-0.5 w-3 h-3 bg-green-500 border-2 border-white rounded-full" />
          </div>
          {!isCollapsed && (
            <div className="flex-1 min-w-0">
              <p className="text-sm font-semibold text-slate-900 truncate">
                {user?.full_name || user?.email?.split('@')[0] || 'Merchant User'}
              </p>
              <p className="text-xs text-slate-500 truncate">{user?.email || 'user@example.com'}</p>
            </div>
          )}
          {!isCollapsed && (
            <button 
              onClick={logout}
              className="p-2 hover:bg-slate-200 rounded-lg transition-colors text-slate-500"
              title="Logout"
            >
              <LogOut className="w-4 h-4" />
            </button>
          )}
        </div>
      </div>
    </div>
  )
}
