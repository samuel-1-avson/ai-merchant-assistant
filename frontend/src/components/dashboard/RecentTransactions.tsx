'use client'

import { useEffect, useState } from 'react'

interface Transaction {
  id: string
  product_name: string
  quantity: number
  unit: string
  price: number
  total: number
  created_at: string
}

export function RecentTransactions() {
  const [transactions, setTransactions] = useState<Transaction[]>([])
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    // Fetch transactions from API
    fetch('http://localhost:3000/api/v1/transactions')
      .then(res => res.json())
      .then(data => {
        if (data.success) {
          setTransactions(data.data)
        }
        setIsLoading(false)
      })
      .catch(() => setIsLoading(false))
  }, [])

  if (isLoading) {
    return (
      <div className="flex justify-center py-8">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
      </div>
    )
  }

  if (transactions.length === 0) {
    return (
      <div className="text-center py-8 text-gray-500">
        <p>No transactions yet</p>
        <p className="text-sm mt-1">Record your first sale using the voice recorder!</p>
      </div>
    )
  }

  return (
    <div className="space-y-4">
      {transactions.map((transaction) => (
        <div
          key={transaction.id}
          className="flex items-center justify-between p-4 bg-gray-50 rounded-lg"
        >
          <div>
            <p className="font-medium">{transaction.product_name}</p>
            <p className="text-sm text-gray-500">
              {transaction.quantity} {transaction.unit} × ${transaction.price}
            </p>
          </div>
          <div className="text-right">
            <p className="font-semibold">${transaction.total}</p>
            <p className="text-xs text-gray-500">
              {new Date(transaction.created_at).toLocaleTimeString()}
            </p>
          </div>
        </div>
      ))}
    </div>
  )
}
