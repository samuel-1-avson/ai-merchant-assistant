'use client'

import { useState, useEffect, useCallback } from 'react'
import { Layout } from '@/components/layout/Layout'
import { productsApi } from '@/lib/api/client'
import { useToast } from '@/components/ui/Toast'
import { Product } from '@/types'
import { formatCurrency } from '@/lib/utils'
import { 
  Package, 
  Search, 
  Plus, 
  Edit2, 
  Trash2,
  TrendingUp,
  TrendingDown,
  AlertTriangle,
  RefreshCw,
  X,
  Save
} from 'lucide-react'

interface ProductFormData {
  name: string
  description: string
  sku: string
  default_price: string
  cost_price: string
  unit: string
  stock_quantity: string
  low_stock_threshold: string
}

export default function ProductsPage() {
  const [products, setProducts] = useState<Product[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [searchQuery, setSearchQuery] = useState('')
  const [showAddModal, setShowAddModal] = useState(false)
  const [editingProduct, setEditingProduct] = useState<Product | null>(null)
  const { success, error } = useToast()

  const [formData, setFormData] = useState<ProductFormData>({
    name: '',
    description: '',
    sku: '',
    default_price: '',
    cost_price: '',
    unit: 'pcs',
    stock_quantity: '',
    low_stock_threshold: '10'
  })

  const fetchProducts = useCallback(async () => {
    setIsLoading(true)
    try {
      const result = await productsApi.list(searchQuery)
      if (result.success && result.data) {
        setProducts(result.data.products)
      } else {
        error(result.error || 'Failed to fetch products')
      }
    } catch (err) {
      error('An error occurred while fetching products')
      console.error(err)
    } finally {
      setIsLoading(false)
    }
  }, [searchQuery, error])

  useEffect(() => {
    fetchProducts()
  }, [fetchProducts])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    try {
      const data = {
        name: formData.name,
        description: formData.description || undefined,
        sku: formData.sku || undefined,
        default_price: formData.default_price ? parseFloat(formData.default_price) : undefined,
        cost_price: formData.cost_price ? parseFloat(formData.cost_price) : undefined,
        unit: formData.unit || 'pcs',
        stock_quantity: formData.stock_quantity ? parseInt(formData.stock_quantity) : 0,
        low_stock_threshold: formData.low_stock_threshold ? parseInt(formData.low_stock_threshold) : 10
      }

      const result = await productsApi.create(data)
      
      if (result.success) {
        success('Product created successfully')
        setShowAddModal(false)
        resetForm()
        fetchProducts()
      } else {
        error(result.error || 'Failed to create product')
      }
    } catch (err) {
      error('An error occurred while creating the product')
      console.error(err)
    }
  }

  const resetForm = () => {
    setFormData({
      name: '',
      description: '',
      sku: '',
      default_price: '',
      cost_price: '',
      unit: 'pcs',
      stock_quantity: '',
      low_stock_threshold: '10'
    })
  }

  const getStockStatus = (product: Product) => {
    if (product.stock_quantity === undefined || product.low_stock_threshold === undefined) {
      return { label: 'Unknown', color: 'gray', icon: AlertTriangle }
    }
    
    if (product.stock_quantity <= 0) {
      return { label: 'Out of Stock', color: 'red', icon: AlertTriangle }
    }
    if (product.stock_quantity <= product.low_stock_threshold) {
      return { label: 'Low Stock', color: 'yellow', icon: TrendingDown }
    }
    return { label: 'In Stock', color: 'green', icon: TrendingUp }
  }

  const totalProducts = (products || []).length
  const lowStockProducts = (products || []).filter(p => 
    p.stock_quantity !== undefined && 
    p.low_stock_threshold !== undefined && 
    p.stock_quantity <= p.low_stock_threshold &&
    p.stock_quantity > 0
  ).length
  const outOfStockProducts = (products || []).filter(p => 
    p.stock_quantity !== undefined && p.stock_quantity <= 0
  ).length

  return (
    <Layout>
      <div className="space-y-6">
        {/* Header */}
        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div>
            <h1 className="text-2xl font-bold text-slate-900">Products</h1>
            <p className="text-slate-500 mt-1">
              Manage your product inventory
            </p>
          </div>
          
          <div className="flex items-center gap-3">
            <button 
              onClick={fetchProducts}
              className="btn-ghost text-sm"
            >
              <RefreshCw className="w-4 h-4" />
              Refresh
            </button>
            <button 
              onClick={() => setShowAddModal(true)}
              className="btn-primary text-sm"
            >
              <Plus className="w-4 h-4" />
              Add Product
            </button>
          </div>
        </div>

        {/* Stats Cards */}
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
          <div className="bg-white p-4 rounded-xl border border-slate-200">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 bg-blue-100 rounded-lg flex items-center justify-center">
                <Package className="w-5 h-5 text-blue-600" />
              </div>
              <div>
                <p className="text-sm text-slate-500">Total Products</p>
                <p className="text-2xl font-bold text-slate-900">{totalProducts}</p>
              </div>
            </div>
          </div>
          <div className="bg-white p-4 rounded-xl border border-slate-200">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 bg-yellow-100 rounded-lg flex items-center justify-center">
                <TrendingDown className="w-5 h-5 text-yellow-600" />
              </div>
              <div>
                <p className="text-sm text-slate-500">Low Stock</p>
                <p className="text-2xl font-bold text-slate-900">{lowStockProducts}</p>
              </div>
            </div>
          </div>
          <div className="bg-white p-4 rounded-xl border border-slate-200">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 bg-red-100 rounded-lg flex items-center justify-center">
                <AlertTriangle className="w-5 h-5 text-red-600" />
              </div>
              <div>
                <p className="text-sm text-slate-500">Out of Stock</p>
                <p className="text-2xl font-bold text-slate-900">{outOfStockProducts}</p>
              </div>
            </div>
          </div>
        </div>

        {/* Search */}
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-slate-400" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search products by name..."
            className="w-full pl-10 pr-4 py-3 border border-slate-200 rounded-xl focus:ring-2 focus:ring-primary-500 focus:border-transparent"
          />
        </div>

        {/* Products Grid */}
        {isLoading ? (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {[...Array(6)].map((_, i) => (
              <div key={i} className="h-48 bg-slate-200 rounded-xl animate-pulse" />
            ))}
          </div>
        ) : (!products || products.length === 0) ? (
          <div className="text-center py-12 bg-white rounded-xl border border-slate-200">
            <Package className="w-12 h-12 text-slate-300 mx-auto mb-4" />
            <h3 className="text-lg font-semibold text-slate-900 mb-2">No products found</h3>
            <p className="text-slate-500 mb-4">
              {searchQuery ? 'No products match your search.' : "You haven't added any products yet."}
            </p>
            <button 
              onClick={() => setShowAddModal(true)}
              className="btn-primary"
            >
              <Plus className="w-4 h-4" />
              Add Your First Product
            </button>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {(products || []).map((product) => {
              const status = getStockStatus(product)
              const StatusIcon = status.icon
              
              return (
                <div 
                  key={product.id}
                  className="bg-white p-5 rounded-xl border border-slate-200 hover:border-primary-300 transition-colors"
                >
                  <div className="flex items-start justify-between mb-4">
                    <div className="w-12 h-12 bg-primary-100 rounded-xl flex items-center justify-center">
                      <Package className="w-6 h-6 text-primary-600" />
                    </div>
                    <span className={`inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium ${
                      status.color === 'green' ? 'bg-green-100 text-green-700' :
                      status.color === 'yellow' ? 'bg-yellow-100 text-yellow-700' :
                      status.color === 'red' ? 'bg-red-100 text-red-700' :
                      'bg-gray-100 text-gray-700'
                    }`}>
                      <StatusIcon className="w-3 h-3" />
                      {status.label}
                    </span>
                  </div>

                  <h3 className="font-semibold text-slate-900 mb-1">{product.name}</h3>
                  {product.description && (
                    <p className="text-sm text-slate-500 mb-3 line-clamp-2">{product.description}</p>
                  )}

                  <div className="space-y-2 text-sm">
                    <div className="flex justify-between">
                      <span className="text-slate-500">Price:</span>
                      <span className="font-medium">
                        {product.default_price 
                          ? formatCurrency(product.default_price, 'USD').replace('US', '')
                          : 'Not set'}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-slate-500">Stock:</span>
                      <span className="font-medium">
                        {product.stock_quantity ?? 'N/A'} {product.unit}
                      </span>
                    </div>
                    {product.sku && (
                      <div className="flex justify-between">
                        <span className="text-slate-500">SKU:</span>
                        <span className="font-medium text-slate-400">{product.sku}</span>
                      </div>
                    )}
                  </div>

                  <div className="flex items-center gap-2 mt-4 pt-4 border-t border-slate-100">
                    <button 
                      className="flex-1 btn-ghost text-xs py-2"
                      onClick={() => {
                        setEditingProduct(product)
                        setFormData({
                          name: product.name,
                          description: product.description || '',
                          sku: product.sku || '',
                          default_price: product.default_price?.toString() || '',
                          cost_price: product.cost_price?.toString() || '',
                          unit: product.unit || 'pcs',
                          stock_quantity: product.stock_quantity?.toString() || '',
                          low_stock_threshold: product.low_stock_threshold?.toString() || '10'
                        })
                        setShowAddModal(true)
                      }}
                    >
                      <Edit2 className="w-3 h-3" />
                      Edit
                    </button>
                  </div>
                </div>
              )
            })}
          </div>
        )}

        {/* Add/Edit Product Modal */}
        {showAddModal && (
          <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
            <div className="bg-white rounded-2xl w-full max-w-lg max-h-[90vh] overflow-y-auto">
              <div className="flex items-center justify-between p-6 border-b border-slate-200">
                <h2 className="text-xl font-bold text-slate-900">
                  {editingProduct ? 'Edit Product' : 'Add New Product'}
                </h2>
                <button 
                  onClick={() => {
                    setShowAddModal(false)
                    setEditingProduct(null)
                    resetForm()
                  }}
                  className="p-2 hover:bg-slate-100 rounded-lg"
                >
                  <X className="w-5 h-5" />
                </button>
              </div>

              <form onSubmit={handleSubmit} className="p-6 space-y-4">
                <div>
                  <label className="block text-sm font-medium text-slate-700 mb-1">
                    Product Name *
                  </label>
                  <input
                    type="text"
                    required
                    value={formData.name}
                    onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                    className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                    placeholder="e.g., Blue T-Shirt"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-slate-700 mb-1">
                    Description
                  </label>
                  <textarea
                    value={formData.description}
                    onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                    className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                    rows={3}
                    placeholder="Brief description of the product"
                  />
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium text-slate-700 mb-1">
                      SKU
                    </label>
                    <input
                      type="text"
                      value={formData.sku}
                      onChange={(e) => setFormData({ ...formData, sku: e.target.value })}
                      className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                      placeholder="e.g., SHIRT-001"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-slate-700 mb-1">
                      Unit
                    </label>
                    <select
                      value={formData.unit}
                      onChange={(e) => setFormData({ ...formData, unit: e.target.value })}
                      className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                    >
                      <option value="pcs">Pieces</option>
                      <option value="kg">Kilograms</option>
                      <option value="g">Grams</option>
                      <option value="l">Liters</option>
                      <option value="m">Meters</option>
                      <option value="box">Boxes</option>
                      <option value="pack">Packs</option>
                    </select>
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium text-slate-700 mb-1">
                      Selling Price
                    </label>
                    <input
                      type="number"
                      step="0.01"
                      min="0"
                      value={formData.default_price}
                      onChange={(e) => setFormData({ ...formData, default_price: e.target.value })}
                      className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                      placeholder="0.00"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-slate-700 mb-1">
                      Cost Price
                    </label>
                    <input
                      type="number"
                      step="0.01"
                      min="0"
                      value={formData.cost_price}
                      onChange={(e) => setFormData({ ...formData, cost_price: e.target.value })}
                      className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                      placeholder="0.00"
                    />
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium text-slate-700 mb-1">
                      Current Stock
                    </label>
                    <input
                      type="number"
                      min="0"
                      value={formData.stock_quantity}
                      onChange={(e) => setFormData({ ...formData, stock_quantity: e.target.value })}
                      className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                      placeholder="0"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-slate-700 mb-1">
                      Low Stock Alert At
                    </label>
                    <input
                      type="number"
                      min="0"
                      value={formData.low_stock_threshold}
                      onChange={(e) => setFormData({ ...formData, low_stock_threshold: e.target.value })}
                      className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                      placeholder="10"
                    />
                  </div>
                </div>

                <div className="flex items-center justify-end gap-3 pt-4 border-t border-slate-200">
                  <button
                    type="button"
                    onClick={() => {
                      setShowAddModal(false)
                      setEditingProduct(null)
                      resetForm()
                    }}
                    className="btn-ghost"
                  >
                    Cancel
                  </button>
                  <button
                    type="submit"
                    className="btn-primary"
                  >
                    <Save className="w-4 h-4" />
                    {editingProduct ? 'Save Changes' : 'Create Product'}
                  </button>
                </div>
              </form>
            </div>
          </div>
        )}
      </div>
    </Layout>
  )
}
