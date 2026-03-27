'use client'

import { useState, useRef, useCallback } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { Mic, Square, Loader2, Volume2, CheckCircle2 } from 'lucide-react'
import { useDashboardStore } from '@/stores/dashboardStore'

interface VoiceRecorderProps {
  onSuccess?: (message: string) => void
  onError?: (message: string) => void
}

export function VoiceRecorder({ onSuccess, onError }: VoiceRecorderProps) {
  const [isRecording, setIsRecording] = useState(false)
  const [isProcessing, setIsProcessing] = useState(false)
  const [transcription, setTranscription] = useState<string>('')
  const [recordingDuration, setRecordingDuration] = useState(0)
  const [audioLevel, setAudioLevel] = useState<number[]>(new Array(20).fill(10))
  
  const mediaRecorderRef = useRef<MediaRecorder | null>(null)
  const audioChunksRef = useRef<Blob[]>([])
  const recordingTimerRef = useRef<NodeJS.Timeout | null>(null)
  const audioContextRef = useRef<AudioContext | null>(null)
  const analyserRef = useRef<AnalyserNode | null>(null)
  const animationFrameRef = useRef<number | null>(null)

  // Get store action
  const { createVoiceTransaction, lastTranscription, clearLastVoiceTransaction } = useDashboardStore()

  const updateAudioLevel = useCallback(() => {
    if (!analyserRef.current) return
    
    const dataArray = new Uint8Array(analyserRef.current.frequencyBinCount)
    analyserRef.current.getByteFrequencyData(dataArray)
    
    const average = dataArray.reduce((a, b) => a + b) / dataArray.length
    
    setAudioLevel(prev => {
      const newLevels = [...prev.slice(1), Math.max(10, average)]
      return newLevels
    })
    
    if (isRecording) {
      animationFrameRef.current = requestAnimationFrame(updateAudioLevel)
    }
  }, [isRecording])

  const startRecording = useCallback(async () => {
    try {
      // Clear previous transcription
      clearLastVoiceTransaction()
      setTranscription('')

      const stream = await navigator.mediaDevices.getUserMedia({ audio: true })
      
      audioContextRef.current = new (window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext)()
      const source = audioContextRef.current.createMediaStreamSource(stream)
      analyserRef.current = audioContextRef.current.createAnalyser()
      analyserRef.current.fftSize = 256
      source.connect(analyserRef.current)
      
      const mediaRecorder = new MediaRecorder(stream)
      mediaRecorderRef.current = mediaRecorder
      audioChunksRef.current = []

      mediaRecorder.ondataavailable = (event) => {
        if (event.data.size > 0) {
          audioChunksRef.current.push(event.data)
        }
      }

      mediaRecorder.onstop = async () => {
        const audioBlob = new Blob(audioChunksRef.current, { type: 'audio/wav' })
        await processAudio(audioBlob)
      }

      mediaRecorder.start()
      setIsRecording(true)
      setRecordingDuration(0)
      
      recordingTimerRef.current = setInterval(() => {
        setRecordingDuration(prev => prev + 1)
      }, 1000)
      
      updateAudioLevel()
      
    } catch (error) {
      console.error('Error accessing microphone:', error)
      onError?.('Could not access microphone. Please check permissions.')
    }
  }, [updateAudioLevel, clearLastVoiceTransaction, onError])

  const stopRecording = useCallback(() => {
    if (mediaRecorderRef.current && isRecording) {
      mediaRecorderRef.current.stop()
      setIsRecording(false)
      setIsProcessing(true)
      
      if (recordingTimerRef.current) {
        clearInterval(recordingTimerRef.current)
      }
      
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current)
      }

      mediaRecorderRef.current.stream.getTracks().forEach(track => track.stop())
      
      if (audioContextRef.current) {
        audioContextRef.current.close()
      }
    }
  }, [isRecording])

  const processAudio = async (audioBlob: Blob) => {
    try {
      const reader = new FileReader()
      reader.readAsDataURL(audioBlob)
      reader.onloadend = async () => {
        const base64Audio = reader.result?.toString().split(',')[1]
        
        if (base64Audio) {
          const result = await createVoiceTransaction(base64Audio)
          
          if (result.success && result.transcription) {
            setTranscription(result.transcription)
            onSuccess?.(`Transaction recorded: "${result.transcription}"`)
          } else {
            onError?.('Failed to process voice input. Please try again.')
          }
        }
      }
    } catch (error) {
      console.error('Error processing audio:', error)
      onError?.('Error processing audio. Please try again.')
    } finally {
      setIsProcessing(false)
      setRecordingDuration(0)
    }
  }

  const formatDuration = (seconds: number) => {
    const mins = Math.floor(seconds / 60)
    const secs = seconds % 60
    return `${mins}:${secs.toString().padStart(2, '0')}`
  }

  // Use transcription from store if available
  const displayTranscription = transcription || lastTranscription

  return (
    <div className="flex flex-col items-center gap-6">
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
              <p className="font-semibold text-slate-900">Processing...</p>
              <p className="text-sm text-slate-500">AI is analyzing your voice</p>
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
            {/* Audio Visualizer */}
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
                    style={{
                      height: `${Math.max(10, level)}%`,
                    }}
                    animate={{
                      height: [`${Math.max(10, level)}%`, `${Math.max(10, level * 0.7)}%`, `${Math.max(10, level)}%`],
                    }}
                    transition={{
                      duration: 0.2,
                      repeat: Infinity,
                      repeatType: 'reverse',
                      delay: i * 0.02,
                    }}
                  />
                ))}
              </motion.div>
            )}

            {/* Record Button */}
            <motion.button
              onClick={isRecording ? stopRecording : startRecording}
              whileHover={{ scale: 1.05 }}
              whileTap={{ scale: 0.95 }}
              className={`
                relative h-28 w-28 rounded-full flex items-center justify-center
                transition-all duration-300 shadow-lg
                ${isRecording 
                  ? 'bg-red-500 hover:bg-red-600 shadow-red-500/30' 
                  : 'bg-gradient-to-br from-primary-500 to-primary-700 hover:shadow-glow'
                }
              `}
            >
              <AnimatePresence mode="wait">
                {isRecording ? (
                  <motion.div
                    key="stop"
                    initial={{ scale: 0, rotate: -180 }}
                    animate={{ scale: 1, rotate: 0 }}
                    exit={{ scale: 0, rotate: 180 }}
                  >
                    <Square className="h-10 w-10 text-white fill-current" />
                  </motion.div>
                ) : (
                  <motion.div
                    key="mic"
                    initial={{ scale: 0, rotate: 180 }}
                    animate={{ scale: 1, rotate: 0 }}
                    exit={{ scale: 0, rotate: -180 }}
                  >
                    <Mic className="h-10 w-10 text-white" />
                  </motion.div>
                )}
              </AnimatePresence>
              
              {isRecording && (
                <>
                  <motion.div
                    className="absolute inset-0 rounded-full border-4 border-red-500"
                    initial={{ scale: 1, opacity: 1 }}
                    animate={{ scale: 1.5, opacity: 0 }}
                    transition={{ duration: 1.5, repeat: Infinity }}
                  />
                  <motion.div
                    className="absolute inset-0 rounded-full border-4 border-red-400"
                    initial={{ scale: 1, opacity: 1 }}
                    animate={{ scale: 1.3, opacity: 0 }}
                    transition={{ duration: 1.5, repeat: Infinity, delay: 0.5 }}
                  />
                </>
              )}
            </motion.button>
            
            {/* Recording Info */}
            <div className="text-center">
              {isRecording ? (
                <div className="flex flex-col items-center gap-1">
                  <p className="text-2xl font-mono font-semibold text-red-500">
                    {formatDuration(recordingDuration)}
                  </p>
                  <p className="text-sm text-slate-500">Tap to stop recording</p>
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

      {/* Transcription Result */}
      <AnimatePresence>
        {displayTranscription && !isProcessing && (
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
