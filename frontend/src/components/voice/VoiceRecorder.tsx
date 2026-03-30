'use client'

import { useState, useRef, useCallback } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { Mic, Square, Loader2, CheckCircle2, Volume2 } from 'lucide-react'
import { useDashboardStore } from '@/stores/dashboardStore'
import { voiceApi } from '@/lib/api/client'
import { PendingConfirmationCard } from './PendingConfirmationCard'

interface VoiceRecorderProps {
  onSuccess?: (message: string) => void
  onError?: (message: string) => void
}

/**
 * VoiceRecorder
 *
 * Fixes applied:
 *   Fix 2: After a confirmed transaction, calls /voice/synthesize and plays
 *           the TTS audio response in the browser.
 *   Fix 3: When the backend returns a PendingConfirmation, renders
 *           PendingConfirmationCard so the user can review and confirm/reject.
 */
export function VoiceRecorder({ onSuccess, onError }: VoiceRecorderProps) {
  const [isRecording, setIsRecording] = useState(false)
  const [isProcessing, setIsProcessing] = useState(false)
  const [isSpeaking, setIsSpeaking] = useState(false)
  const [transcription, setTranscription] = useState<string>('')
  const [recordingDuration, setRecordingDuration] = useState(0)
  const [audioLevel, setAudioLevel] = useState<number[]>(new Array(20).fill(10))

  const mediaRecorderRef = useRef<MediaRecorder | null>(null)
  const audioChunksRef = useRef<Blob[]>([])
  const recordingTimerRef = useRef<NodeJS.Timeout | null>(null)
  const autoStopTimerRef = useRef<NodeJS.Timeout | null>(null)
  const audioContextRef = useRef<AudioContext | null>(null)
  const analyserRef = useRef<AnalyserNode | null>(null)
  const animationFrameRef = useRef<number | null>(null)

  const MAX_RECORDING_SECONDS = 30

  const {
    createVoiceTransaction,
    lastTranscription,
    clearLastVoiceTransaction,
    pendingConfirmation,
    clearPendingConfirmation,
    awaitingPrice,
    clearAwaitingPrice,
  } = useDashboardStore()

  // ── Audio level visualizer ─────────────────────────────────────────

  const updateAudioLevel = useCallback(() => {
    if (!analyserRef.current) return
    const dataArray = new Uint8Array(analyserRef.current.frequencyBinCount)
    analyserRef.current.getByteFrequencyData(dataArray)
    const average = dataArray.reduce((a, b) => a + b) / dataArray.length
    setAudioLevel(prev => [...prev.slice(1), Math.max(10, average)])
    if (isRecording) {
      animationFrameRef.current = requestAnimationFrame(updateAudioLevel)
    }
  }, [isRecording])

  // ── Recording controls ────────────────────────────────────────────

  const startRecording = useCallback(async () => {
    try {
      clearLastVoiceTransaction()
      clearPendingConfirmation()
      setTranscription('')

      const stream = await navigator.mediaDevices.getUserMedia({ audio: true })

      audioContextRef.current = new (window.AudioContext ||
        (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext)()
      const source = audioContextRef.current.createMediaStreamSource(stream)
      analyserRef.current = audioContextRef.current.createAnalyser()
      analyserRef.current.fftSize = 256
      source.connect(analyserRef.current)

      // Prefer Opus (compressed, small) over PCM (uncompressed, large).
      // PCM/WAV recordings can exceed server body limits even for short clips.
      const mimeType = MediaRecorder.isTypeSupported('audio/webm;codecs=opus')
        ? 'audio/webm;codecs=opus'
        : MediaRecorder.isTypeSupported('audio/webm')
        ? 'audio/webm'
        : MediaRecorder.isTypeSupported('audio/ogg;codecs=opus')
        ? 'audio/ogg;codecs=opus'
        : 'audio/webm'

      const mediaRecorder = new MediaRecorder(stream, { mimeType })
      mediaRecorderRef.current = mediaRecorder
      audioChunksRef.current = []

      mediaRecorder.ondataavailable = (e) => {
        if (e.data.size > 0) audioChunksRef.current.push(e.data)
      }

      mediaRecorder.onstop = async () => {
        if (autoStopTimerRef.current) clearTimeout(autoStopTimerRef.current)
        const audioBlob = new Blob(audioChunksRef.current, { type: mimeType })
        await processAudio(audioBlob)
      }

      mediaRecorder.start()
      setIsRecording(true)
      setRecordingDuration(0)
      recordingTimerRef.current = setInterval(() => setRecordingDuration(p => p + 1), 1000)

      // Auto-stop after MAX_RECORDING_SECONDS to prevent oversized uploads
      autoStopTimerRef.current = setTimeout(() => {
        if (mediaRecorderRef.current?.state === 'recording') {
          mediaRecorderRef.current.stop()
          setIsRecording(false)
          setIsProcessing(true)
          if (recordingTimerRef.current) clearInterval(recordingTimerRef.current)
          if (animationFrameRef.current) cancelAnimationFrame(animationFrameRef.current)
          mediaRecorderRef.current.stream.getTracks().forEach(t => t.stop())
          if (audioContextRef.current) audioContextRef.current.close()
        }
      }, MAX_RECORDING_SECONDS * 1000)

      updateAudioLevel()
    } catch {
      onError?.('Could not access microphone. Please check permissions.')
    }
  }, [updateAudioLevel, clearLastVoiceTransaction, clearPendingConfirmation, onError])

  const stopRecording = useCallback(() => {
    if (mediaRecorderRef.current && isRecording) {
      mediaRecorderRef.current.stop()
      setIsRecording(false)
      setIsProcessing(true)
      if (recordingTimerRef.current) clearInterval(recordingTimerRef.current)
      if (autoStopTimerRef.current) clearTimeout(autoStopTimerRef.current)
      if (animationFrameRef.current) cancelAnimationFrame(animationFrameRef.current)
      mediaRecorderRef.current.stream.getTracks().forEach(t => t.stop())
      if (audioContextRef.current) audioContextRef.current.close()
    }
  }, [isRecording])

  // ── Audio processing ──────────────────────────────────────────────

  const processAudio = async (audioBlob: Blob) => {
    try {
      const reader = new FileReader()
      reader.readAsDataURL(audioBlob)
      reader.onloadend = async () => {
        const base64Audio = reader.result?.toString().split(',')[1]
        if (!base64Audio) { setIsProcessing(false); return }

        const result = await createVoiceTransaction(base64Audio)

        if (result.success) {
          if (result.type === 'pending') {
            setIsProcessing(false)
            return
          }

          if (result.type === 'awaiting_price') {
            // Saved without price — show prompt; backend will apply price on next recording
            setIsProcessing(false)
            return
          }

          // Immediate transaction committed
          if (result.transcription) {
            setTranscription(result.transcription)
            onSuccess?.(`Transaction recorded: "${result.transcription}"`)
            await speakConfirmation(result.transcription)
          }
        } else {
          onError?.('Failed to process voice input. Please try again.')
        }

        setIsProcessing(false)
      }
    } catch {
      onError?.('Error processing audio. Please try again.')
      setIsProcessing(false)
    }
  }

  // ── Fix 2: TTS playback ───────────────────────────────────────────

  /**
   * Call the backend TTS endpoint and play the audio response.
   * If TTS fails (network, model loading, etc.) we silently skip it —
   * the text confirmation is still shown in the UI.
   */
  const speakConfirmation = async (transcriptionText: string) => {
    const summary = buildTTSSummary(transcriptionText)
    setIsSpeaking(true)
    try {
      const resp = await voiceApi.synthesize(summary)
      if (resp.success && resp.data?.audio) {
        await voiceApi.playBase64Audio(resp.data.audio)
        return
      }
    } catch {
      // backend TTS unavailable — fall through to Web Speech API
    }
    // Fallback: browser-native TTS (no network required)
    if (typeof window !== 'undefined' && window.speechSynthesis) {
      const utterance = new SpeechSynthesisUtterance(summary)
      utterance.onend = () => setIsSpeaking(false)
      utterance.onerror = () => setIsSpeaking(false)
      window.speechSynthesis.speak(utterance)
      return
    }
    setIsSpeaking(false)
  }

  /** Build a short TTS phrase from the transcription. */
  const buildTTSSummary = (text: string) => {
    // Keep the TTS response short so MeloTTS responds quickly
    const trimmed = text.length > 80 ? text.slice(0, 80) + '...' : text
    return `Transaction recorded. ${trimmed}`
  }

  // ── Helpers ───────────────────────────────────────────────────────

  const formatDuration = (s: number) =>
    `${Math.floor(s / 60)}:${(s % 60).toString().padStart(2, '0')}`

  const displayTranscription = transcription || lastTranscription

  // ── Render ────────────────────────────────────────────────────────

  return (
    <div className="flex flex-col items-center gap-6 w-full">

      {/* ── Awaiting price card ──────────────────────────────────────── */}
      {awaitingPrice && !pendingConfirmation && (
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          className="w-full bg-amber-50 border border-amber-300 rounded-2xl p-4"
        >
          <div className="flex items-start gap-3">
            <div className="w-8 h-8 bg-amber-100 rounded-lg flex items-center justify-center flex-shrink-0">
              <span className="text-amber-600 text-lg font-bold">$</span>
            </div>
            <div className="flex-1">
              <p className="text-sm font-semibold text-amber-900 mb-0.5">Price needed</p>
              <p className="text-sm text-amber-700">
                No price heard for <strong>{awaitingPrice.productName}</strong>. Tap the mic and say the price — e.g. &quot;The price was $20&quot;
              </p>
              <button
                onClick={clearAwaitingPrice}
                className="mt-2 text-xs text-amber-500 underline"
              >
                Skip
              </button>
            </div>
          </div>
        </motion.div>
      )}

      {/* ── Pending confirmation card (Fix 3) ───────────────────────── */}
      {pendingConfirmation && (
        <div className="w-full">
          <PendingConfirmationCard
            confirmation={pendingConfirmation}
            onConfirmed={() => {
              onSuccess?.('Transaction confirmed and recorded.')
              speakConfirmation('Transaction confirmed.')
              // Refresh handled by parent via onSuccess callback
            }}
            onRejected={() => onError?.('Transaction rejected.')}
          />
        </div>
      )}

      {/* ── Recording / processing states ────────────────────────── */}
      {!pendingConfirmation && (
        <AnimatePresence mode="wait">
          {isProcessing ? (
            <motion.div
              key="processing"
              initial={{ opacity: 0, scale: 0.8 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.8 }}
              className="flex flex-col items-center gap-4 py-8"
            >
              <div className="relative">
                <div className="w-24 h-24 rounded-full bg-primary-100 flex items-center justify-center">
                  <Loader2 className="w-10 h-10 text-primary-600 animate-spin" />
                </div>
                <motion.div
                  className="absolute inset-0 rounded-full border-4 border-primary-200"
                  animate={{ scale: [1, 1.2, 1], opacity: [1, 0, 1] }}
                  transition={{ duration: 2, repeat: Infinity }}
                />
              </div>
              <div className="text-center">
                <p className="font-semibold text-slate-900">Processing…</p>
                <p className="text-sm text-slate-500">AI is analysing your voice</p>
              </div>
            </motion.div>
          ) : (
            <motion.div
              key="record"
              className="flex flex-col items-center gap-4"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
            >
              {/* Waveform visualizer */}
              {isRecording && (
                <motion.div
                  className="flex items-center gap-1 h-16"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                >
                  {audioLevel.map((level, i) => (
                    <motion.div
                      key={i}
                      className="w-1.5 bg-primary-500 rounded-full"
                      style={{ height: `${Math.max(10, level)}%` }}
                      animate={{
                        height: [`${Math.max(10, level)}%`, `${Math.max(10, level * 0.7)}%`, `${Math.max(10, level)}%`],
                      }}
                      transition={{ duration: 0.2, repeat: Infinity, repeatType: 'reverse', delay: i * 0.02 }}
                    />
                  ))}
                </motion.div>
              )}

              {/* Record / stop button */}
              <motion.button
                onClick={isRecording ? stopRecording : startRecording}
                whileHover={{ scale: 1.05 }}
                whileTap={{ scale: 0.95 }}
                className={`relative h-28 w-28 rounded-full flex items-center justify-center
                  transition-all duration-300 shadow-lg
                  ${isRecording
                    ? 'bg-red-500 hover:bg-red-600 shadow-red-500/30'
                    : 'bg-gradient-to-br from-primary-500 to-primary-700 hover:shadow-glow'
                  }`}
              >
                <AnimatePresence mode="wait">
                  {isRecording ? (
                    <motion.div key="stop" initial={{ scale: 0, rotate: -180 }} animate={{ scale: 1, rotate: 0 }} exit={{ scale: 0, rotate: 180 }}>
                      <Square className="h-10 w-10 text-white fill-current" />
                    </motion.div>
                  ) : (
                    <motion.div key="mic" initial={{ scale: 0, rotate: 180 }} animate={{ scale: 1, rotate: 0 }} exit={{ scale: 0, rotate: -180 }}>
                      <Mic className="h-10 w-10 text-white" />
                    </motion.div>
                  )}
                </AnimatePresence>

                {isRecording && (
                  <>
                    <motion.div className="absolute inset-0 rounded-full border-4 border-red-500"
                      initial={{ scale: 1, opacity: 1 }} animate={{ scale: 1.5, opacity: 0 }}
                      transition={{ duration: 1.5, repeat: Infinity }} />
                    <motion.div className="absolute inset-0 rounded-full border-4 border-red-400"
                      initial={{ scale: 1, opacity: 1 }} animate={{ scale: 1.3, opacity: 0 }}
                      transition={{ duration: 1.5, repeat: Infinity, delay: 0.5 }} />
                  </>
                )}
              </motion.button>

              {/* Recording info */}
              <div className="text-center">
                {isRecording ? (
                  <div className="flex flex-col items-center gap-2">
                    <p className="text-2xl font-mono font-semibold text-red-500">{formatDuration(recordingDuration)}</p>
                    {/* Progress bar toward 30-second limit */}
                    <div className="w-32 h-1.5 bg-slate-200 rounded-full overflow-hidden">
                      <div
                        className="h-full bg-red-400 rounded-full transition-all duration-1000"
                        style={{ width: `${Math.min((recordingDuration / MAX_RECORDING_SECONDS) * 100, 100)}%` }}
                      />
                    </div>
                    <p className="text-sm text-slate-500">Tap to stop · max {MAX_RECORDING_SECONDS}s</p>
                  </div>
                ) : (
                  <div className="flex flex-col items-center gap-1">
                    <p className="font-semibold text-slate-900">Tap to record</p>
                    <p className="text-sm text-slate-500">Speak your transaction naturally</p>
                  </div>
                )}
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      )}

      {/* ── TTS speaking indicator (Fix 2) ────────────────────────── */}
      <AnimatePresence>
        {isSpeaking && (
          <motion.div
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -8 }}
            className="flex items-center gap-2 text-sm text-primary-600"
          >
            <Volume2 className="w-4 h-4 animate-pulse" />
            <span>Speaking confirmation…</span>
          </motion.div>
        )}
      </AnimatePresence>

      {/* ── Transcription result ──────────────────────────────────── */}
      <AnimatePresence>
        {displayTranscription && !isProcessing && !pendingConfirmation && (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -20 }}
            className="w-full bg-secondary-50 border border-secondary-200 rounded-2xl p-4"
          >
            <div className="flex items-start gap-3">
              <div className="w-8 h-8 bg-secondary-100 rounded-lg flex items-center justify-center flex-shrink-0">
                <CheckCircle2 className="w-4 h-4 text-secondary-600" />
              </div>
              <div>
                <p className="text-sm font-semibold text-secondary-900 mb-1">Transaction Recorded</p>
                <p className="text-sm text-secondary-700">&quot;{displayTranscription}&quot;</p>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  )
}
