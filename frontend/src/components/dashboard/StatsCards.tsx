'use client'

import { TrendingUp, TrendingDown, DollarSign, ShoppingCart, Package, Users } from 'lucide-react'

const stats = [
  {
    name: 'Total Revenue',
    value: '$12,450',
    change: '+12%',
    trend: 'up',
    icon: DollarSign,
  },
  {
    name: 'Transactions',
    value: '156',
    change: '+8%',
    trend: 'up',
    icon: ShoppingCart,
  },
  {
    name: 'Products Sold',
    value: '423',
    change: '-3%',
    trend: 'down',
    icon: Package,
  },
  {
    name: 'Active Customers',
    value: '89',
    change: '+5%',
    trend: 'up',
    icon: Users,
  },
]

export function StatsCards() {
  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
      {stats.map((stat) => {
        const Icon = stat.icon
        const TrendIcon = stat.trend === 'up' ? TrendingUp : TrendingDown

        return (
          <div key={stat.name} className="bg-white p-6 rounded-xl shadow-sm border">
            <div className="flex items-center justify-between">
              <div className="p-2 bg-primary-50 rounded-lg">
                <Icon className="w-5 h-5 text-primary-600" />
              </div>
              <div className={`flex items-center gap-1 text-sm ${
                stat.trend === 'up' ? 'text-green-600' : 'text-red-600'
              }`}>
                <TrendIcon className="w-4 h-4" />
                {stat.change}
              </div>
            </div>
            <div className="mt-4">
              <p className="text-sm text-gray-500">{stat.name}</p>
              <p className="text-2xl font-bold">{stat.value}</p>
            </div>
          </div>
        )
      })}
    </div>
  )
}
