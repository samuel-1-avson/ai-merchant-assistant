/**
 * React Hook for WebSocket Connection
 * Manages WebSocket lifecycle and provides real-time updates
 */

import { useEffect, useRef, useCallback, useState } from 'react';
import { 
  getWebSocketClient, 
  WebSocketClient, 
  WsMessageType, 
  WsEventHandler,
  initWebSocket,
  closeWebSocket
} from '@/lib/websocket/client';
import { useAuthStore } from '@/stores/authStore';

interface UseWebSocketOptions {
  autoConnect?: boolean;
  onConnect?: () => void;
  onDisconnect?: () => void;
  onError?: (error: Event) => void;
}

export function useWebSocket(options: UseWebSocketOptions = {}) {
  const { autoConnect = true } = options;
  const { isAuthenticated } = useAuthStore();
  const [isConnected, setIsConnected] = useState(false);
  const clientRef = useRef<WebSocketClient | null>(null);
  const handlersRef = useRef<Map<string, () => void>>(new Map());
  
  // Use refs for callbacks to avoid effect re-runs
  const callbacksRef = useRef(options);
  callbacksRef.current = options;

  useEffect(() => {
    if (autoConnect && isAuthenticated) {
      clientRef.current = initWebSocket();
      
      // Set up connection status tracking
      const unsubscribeConnected = clientRef.current.on('connected', () => {
        setIsConnected(true);
        callbacksRef.current.onConnect?.();
      });

      const unsubscribeError = clientRef.current.on('error', (msg) => {
        callbacksRef.current.onError?.(new Event(msg.message || 'WebSocket error'));
      });

      // Wait a bit and check connection status
      const timer = setTimeout(() => {
        setIsConnected(clientRef.current?.isConnected || false);
      }, 500);

      return () => {
        clearTimeout(timer);
        unsubscribeConnected();
        unsubscribeError();
        // Don't destroy the singleton on page navigation — keep the connection
        // alive so navigating between pages doesn't cause rapid disconnect/reconnect.
        setIsConnected(false);
        callbacksRef.current.onDisconnect?.();
      };
    }
  }, [autoConnect, isAuthenticated]); // Removed callback dependencies

  /**
   * Subscribe to a WebSocket event
   */
  const subscribe = useCallback((event: WsMessageType, handler: WsEventHandler) => {
    if (!clientRef.current) return () => {};
    
    const unsubscribe = clientRef.current.on(event, handler);
    const handlerId = `${event}_${Date.now()}_${Math.random()}`;
    handlersRef.current.set(handlerId, unsubscribe);
    
    return () => {
      unsubscribe();
      handlersRef.current.delete(handlerId);
    };
  }, []);

  /**
   * Send a message through WebSocket
   */
  const send = useCallback((type: string, payload?: any) => {
    clientRef.current?.send(type, payload);
  }, []);

  /**
   * Manually connect
   */
  const connect = useCallback(() => {
    if (!clientRef.current) {
      clientRef.current = initWebSocket();
    } else {
      clientRef.current.connect();
    }
  }, []);

  /**
   * Manually disconnect
   */
  const disconnect = useCallback(() => {
    closeWebSocket();
    clientRef.current = null;
    setIsConnected(false);
  }, []);

  return {
    isConnected,
    subscribe,
    send,
    connect,
    disconnect,
    client: clientRef.current,
  };
}

/**
 * Hook specifically for transaction updates
 */
export function useTransactionUpdates(onUpdate?: (data: any) => void) {
  const { subscribe, isConnected } = useWebSocket();
  const onUpdateRef = useRef(onUpdate);
  onUpdateRef.current = onUpdate;

  useEffect(() => {
    if (!isConnected) return;

    const unsubscribe = subscribe('transaction_update', (message) => {
      onUpdateRef.current?.(message);
    });

    return unsubscribe;
  }, [isConnected, subscribe]);

  return { isConnected };
}

/**
 * Hook specifically for alert updates
 */
export function useAlertUpdates(onAlert?: (alert: any) => void) {
  const { subscribe, isConnected, send } = useWebSocket();
  const [unreadCount, setUnreadCount] = useState(0);
  const onAlertRef = useRef(onAlert);
  onAlertRef.current = onAlert;

  useEffect(() => {
    if (!isConnected) return;

    const unsubscribeNew = subscribe('new_alert', (message) => {
      setUnreadCount(prev => prev + 1);
      onAlertRef.current?.(message.alert);
    });

    const unsubscribeNotification = subscribe('notification', (message) => {
      if (message.severity === 'warning' || message.severity === 'critical') {
        setUnreadCount(prev => prev + 1);
      }
    });

    return () => {
      unsubscribeNew();
      unsubscribeNotification();
    };
  }, [isConnected, subscribe]);

  const markAsRead = useCallback((alertId: string) => {
    send('mark_alert_read', { alert_id: alertId });
    setUnreadCount(prev => Math.max(0, prev - 1));
  }, [send]);

  const clearCount = useCallback(() => {
    setUnreadCount(0);
  }, []);

  return { 
    isConnected, 
    unreadCount, 
    markAsRead,
    clearCount,
  };
}
