/**
 * @jest-environment jsdom
 */

import { render, screen } from '@testing-library/react'
import '@testing-library/jest-dom'
import { StatsCards } from '@/components/dashboard/StatsCards'

describe('Dashboard Components', () => {
  test('StatsCards renders correctly', () => {
    render(<StatsCards />)
    
    // Check if all stat cards are rendered
    expect(screen.getByText('Total Revenue')).toBeInTheDocument()
    expect(screen.getByText('Transactions')).toBeInTheDocument()
    expect(screen.getByText('Products Sold')).toBeInTheDocument()
    expect(screen.getByText('Active Customers')).toBeInTheDocument()
  })
})
