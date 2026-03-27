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
  const { autoConnect = true, onConnect, onDisconnect, onError } = options;
  const { isAuthenticated } = useAuthStore();
  const [isConnected, setIsConnected] = useState(false);
  const clientRef = useRef<WebSocketClient | null>(null);
  const handlersRef = useRef<Map<string, () => void>>(new Map());

  useEffect(() => {
    if (autoConnect && isAuthenticated) {
      clientRef.current = initWebSocket();
      
      // Set up connection status tracking
      const unsubscribeConnected = clientRef.current.on('connected', () => {
        setIsConnected(true);
        onConnect?.();
      });

      const unsubscribeError = clientRef.current.on('error', (msg) => {
        onError?.(new Event(msg.message || 'WebSocket error'));
      });

      // Wait a bit and check connection status
      const timer = setTimeout(() => {
        setIsConnected(clientRef.current?.isConnected || false);
      }, 500);

      return () => {
        clearTimeout(timer);
        unsubscribeConnected();
        unsubscribeError();
        closeWebSocket();
        setIsConnected(false);
        onDisconnect?.();
      };
    }
  }, [autoConnect, isAuthenticated, onConnect, onDisconnect, onError]);

  /**
   * Subscribe to a WebSocket event
   */
  const subscribe = useCallback((event: WsMessageType, handler: WsEventHandler) => {
    if (!clientRef.current) return () => {};
    
    const unsubscribe = clientRef.current.on(event, handler);
    const handlerId = `${event}_${Date.now()}`;
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

  useEffect(() => {
    if (!isConnected || !onUpdate) return;

    const unsubscribe = subscribe('transaction_update', (message) => {
      onUpdate(message);
    });

    return unsubscribe;
  }, [isConnected, onUpdate, subscribe]);

  return { isConnected };
}

/**
 * Hook specifically for alert updates
 */
export function useAlertUpdates(onAlert?: (alert: any) => void) {
  const { subscribe, isConnected, send } = useWebSocket();
  const [unreadCount, setUnreadCount] = useState(0);

  useEffect(() => {
    if (!isConnected) return;

    const unsubscribeNew = subscribe('new_alert', (message) => {
      setUnreadCount(prev => prev + 1);
      onAlert?.(message.alert);
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
  }, [isConnected, onAlert, subscribe]);

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
