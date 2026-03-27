'use client'

import { motion } from 'framer-motion'
import { TrendingUp, TrendingDown, DollarSign, ShoppingCart, Package, Users, ArrowUpRight } from 'lucide-react'

const stats = [
  {
    name: 'Total Revenue',
    value: '$12,450',
    change: '+12%',
    changeType: 'positive',
    trend: 'up',
    icon: DollarSign,
    color: 'primary',
    bgGradient: 'from-primary-500/10 to-primary-600/5',
  },
  {
    name: 'Transactions',
    value: '156',
    change: '+8%',
    changeType: 'positive',
    trend: 'up',
    icon: ShoppingCart,
    color: 'secondary',
    bgGradient: 'from-secondary-500/10 to-secondary-600/5',
  },
  {
    name: 'Products Sold',
    value: '423',
    change: '-3%',
    changeType: 'negative',
    trend: 'down',
    icon: Package,
    color: 'accent',
    bgGradient: 'from-accent-500/10 to-accent-600/5',
  },
  {
    name: 'Active Customers',
    value: '89',
    change: '+5%',
    changeType: 'positive',
    trend: 'up',
    icon: Users,
    color: 'primary',
    bgGradient: 'from-primary-500/10 to-primary-600/5',
  },
]

export function StatsCards() {
  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
      {stats.map((stat, index) => {
        const Icon = stat.icon
        const TrendIcon = stat.trend === 'up' ? TrendingUp : TrendingDown

        return (
          <motion.div
            key={stat.name}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: index * 0.1 }}
            className={`
              relative overflow-hidden rounded-2xl p-6 
              bg-gradient-to-br ${stat.bgGradient}
              border border-slate-100
              hover:shadow-lg hover:-translate-y-1 transition-all duration-300
              group cursor-pointer
            `}
          >
            {/* Background Pattern */}
            <div className="absolute top-0 right-0 w-32 h-32 bg-white/30 rounded-full -translate-y-1/2 translate-x-1/2 blur-2xl group-hover:scale-150 transition-transform duration-500" />
            
            <div className="relative">
              <div className="flex items-start justify-between mb-4">
                <div className={`p-3 rounded-xl bg-white shadow-sm`}>
                  <Icon className={`w-6 h-6 text-${stat.color}-600`} />
                </div>
                <div className={`flex items-center gap-1 px-2 py-1 rounded-lg text-xs font-semibold ${
                  stat.changeType === 'positive' 
                    ? 'bg-secondary-100 text-secondary-700' 
                    : 'bg-red-100 text-red-700'
                }`}>
                  <TrendIcon className="w-3 h-3" />
                  {stat.change}
                </div>
              </div>
              
              <div>
                <p className="text-sm text-slate-500 mb-1">{stat.name}</p>
                <p className="text-2xl font-bold text-slate-900">{stat.value}</p>
              </div>

              {/* Hover Action */}
              <div className="absolute bottom-0 right-0 opacity-0 group-hover:opacity-100 transition-opacity">
                <div className={`p-2 rounded-lg bg-${stat.color}-100`}>
                  <ArrowUpRight className={`w-4 h-4 text-${stat.color}-600`} />
                </div>
              </div>
            </div>
          </motion.div>
        )
      })}
    </div>
  )
}
