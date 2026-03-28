'use client'

import { motion, AnimatePresence } from 'framer-motion'
import { CheckCircle2, XCircle, AlertTriangle, Clock, Mic, Package, Tag, Hash } from 'lucide-react'
import { PendingConfirmation } from '@/types'
import { useDashboardStore } from '@/stores/dashboardStore'

interface PendingConfirmationCardProps {
  confirmation: PendingConfirmation
  onConfirmed?: () => void
  onRejected?: () => void
}

/**
 * PendingConfirmationCard
 *
 * Shown when the AI voice pipeline returns a low-confidence transaction that
 * requires the user to review and approve or reject before it is committed.
 *
 * Fix 3: This component was previously missing entirely, leaving the
 * confirmation workflow implemented only on the backend with no UI counterpart.
 */
export function PendingConfirmationCard({
  confirmation,
  onConfirmed,
  onRejected,
}: PendingConfirmationCardProps) {
  const { confirmPendingTransaction, rejectPendingTransaction, confirmationLoading } =
    useDashboardStore()

  const confidencePct = Math.round(confirmation.confidence * 100)
  const isHighConfidence = confidencePct >= 75

  const handleConfirm = async () => {
    const ok = await confirmPendingTransaction()
    if (ok) onConfirmed?.()
  }

  const handleReject = async () => {
    await rejectPendingTransaction()
    onRejected?.()
  }

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0, y: 16, scale: 0.97 }}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        exit={{ opacity: 0, y: -16, scale: 0.97 }}
        transition={{ type: 'spring', stiffness: 300, damping: 25 }}
        className="w-full rounded-2xl border border-amber-200 bg-amber-50 shadow-md overflow-hidden"
      >
        {/* Header */}
        <div className="flex items-center gap-3 px-5 py-4 bg-amber-100 border-b border-amber-200">
          <div className="w-9 h-9 rounded-full bg-amber-200 flex items-center justify-center flex-shrink-0">
            <AlertTriangle className="w-5 h-5 text-amber-700" />
          </div>
          <div className="flex-1 min-w-0">
            <p className="text-sm font-semibold text-amber-900">Confirm Transaction</p>
            <p className="text-xs text-amber-700 truncate">
              AI confidence: {confidencePct}%
              {confirmation.is_new_product && ' · New product'}
            </p>
          </div>
          <ConfidenceBadge pct={confidencePct} />
        </div>

        {/* Original voice input */}
        <div className="px-5 pt-4 pb-2">
          <div className="flex items-start gap-2 mb-4">
            <Mic className="w-4 h-4 text-slate-400 mt-0.5 flex-shrink-0" />
            <p className="text-sm text-slate-600 italic">
              &ldquo;{confirmation.original_transcription}&rdquo;
            </p>
          </div>

          {/* Extracted details */}
          <div className="rounded-xl bg-white border border-amber-100 divide-y divide-amber-50 overflow-hidden">
            <DetailRow
              icon={<Package className="w-4 h-4 text-slate-500" />}
              label="Product"
              value={confirmation.display_name}
              highlight={confirmation.is_new_product}
              highlightText="New"
            />
            <DetailRow
              icon={<Hash className="w-4 h-4 text-slate-500" />}
              label="Quantity"
              value={confirmation.display_quantity}
            />
            <DetailRow
              icon={<Tag className="w-4 h-4 text-slate-500" />}
              label="Unit price"
              value={confirmation.display_price}
            />
            <DetailRow
              icon={<Tag className="w-4 h-4 text-slate-500" />}
              label="Total"
              value={confirmation.display_total}
              bold
            />
          </div>

          {/* Proposed product match */}
          {confirmation.proposed_product && (
            <p className="mt-3 text-xs text-slate-500">
              Matched to existing product:{' '}
              <span className="font-medium text-slate-700">
                {confirmation.proposed_product.name}
              </span>{' '}
              (score {confirmation.proposed_product.match_score}/100)
            </p>
          )}
        </div>

        {/* Expiry indicator */}
        {typeof confirmation.remaining_seconds === 'number' && (
          <div className="px-5 pb-2">
            <div className="flex items-center gap-1.5 text-xs text-slate-400">
              <Clock className="w-3.5 h-3.5" />
              <span>Expires in {confirmation.remaining_seconds}s</span>
            </div>
          </div>
        )}

        {/* Action buttons */}
        <div className="px-5 pb-5 pt-2 grid grid-cols-2 gap-3">
          <motion.button
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.97 }}
            onClick={handleReject}
            disabled={confirmationLoading}
            className="flex items-center justify-center gap-2 rounded-xl px-4 py-3
              bg-white border border-red-200 text-red-600 text-sm font-medium
              hover:bg-red-50 disabled:opacity-50 transition-colors"
          >
            <XCircle className="w-4 h-4" />
            Reject
          </motion.button>

          <motion.button
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.97 }}
            onClick={handleConfirm}
            disabled={confirmationLoading}
            className="flex items-center justify-center gap-2 rounded-xl px-4 py-3
              bg-emerald-500 text-white text-sm font-semibold
              hover:bg-emerald-600 disabled:opacity-70 transition-colors shadow-sm"
          >
            {confirmationLoading ? (
              <span className="animate-spin w-4 h-4 border-2 border-white border-t-transparent rounded-full" />
            ) : (
              <CheckCircle2 className="w-4 h-4" />
            )}
            Confirm
          </motion.button>
        </div>
      </motion.div>
    </AnimatePresence>
  )
}

// ── Sub-components ────────────────────────────────────────────────────────

function DetailRow({
  icon,
  label,
  value,
  highlight,
  highlightText,
  bold,
}: {
  icon: React.ReactNode
  label: string
  value: string
  highlight?: boolean
  highlightText?: string
  bold?: boolean
}) {
  return (
    <div className="flex items-center justify-between px-4 py-2.5">
      <div className="flex items-center gap-2 text-slate-500">
        {icon}
        <span className="text-xs">{label}</span>
      </div>
      <div className="flex items-center gap-2">
        {highlight && highlightText && (
          <span className="text-xs px-1.5 py-0.5 rounded-full bg-amber-100 text-amber-700 font-medium">
            {highlightText}
          </span>
        )}
        <span className={`text-sm text-slate-800 ${bold ? 'font-semibold' : ''}`}>{value}</span>
      </div>
    </div>
  )
}

function ConfidenceBadge({ pct }: { pct: number }) {
  const color =
    pct >= 80 ? 'bg-emerald-100 text-emerald-700' :
    pct >= 60 ? 'bg-amber-100 text-amber-700' :
                'bg-red-100 text-red-700'

  return (
    <span className={`text-xs font-semibold px-2 py-1 rounded-full ${color}`}>
      {pct}%
    </span>
  )
}
