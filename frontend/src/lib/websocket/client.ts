/**
 * WebSocket Client for Real-time Updates
 * Connects to backend WebSocket for live transaction and alert updates
 */

import { useAuthStore } from '@/stores/authStore';

export type WsMessageType = 
  | 'connected'
  | 'pong'
  | 'error'
  | 'new_alert'
  | 'transaction_update'
  | 'system_message'
  | 'voice_result'
  | 'notification';

export interface WsMessage {
  type: WsMessageType;
  [key: string]: any;
}

export type WsEventHandler = (message: WsMessage) => void;

export class WebSocketClient {
  private ws: WebSocket | null = null;
  private url: string;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000; // Start with 1s
  private heartbeatInterval: NodeJS.Timeout | null = null;
  private eventHandlers: Map<WsMessageType, Set<WsEventHandler>> = new Map();
  private isIntentionallyClosed = false;
  private isConnecting = false;
  private reconnectTimer: NodeJS.Timeout | null = null;

  constructor(backendUrl?: string) {
    // Convert http(s) to ws(s)
    const baseUrl = backendUrl || process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8888/api/v1';
    // Remove /api/v1 suffix if present and convert to ws
    const wsBase = baseUrl.replace(/\/api\/v1$/, '').replace(/^http/, 'ws');
    this.url = wsBase + '/ws';
  }

  /**
   * Connect to WebSocket server
   */
  connect(): void {
    // Prevent multiple simultaneous connection attempts
    if (this.isConnecting) {
      console.log('[WebSocket] Already connecting, skipping...');
      return;
    }
    
    if (this.ws?.readyState === WebSocket.OPEN) {
      console.log('[WebSocket] Already connected');
      return;
    }

    this.isConnecting = true;
    this.isIntentionallyClosed = false;

    // Get auth token
    const token = typeof window !== 'undefined' ? localStorage.getItem('token') : null;
    const url = token ? `${this.url}?token=${encodeURIComponent(token)}` : this.url;

    console.log('[WebSocket] Connecting...');
    
    try {
      this.ws = new WebSocket(url);

      this.ws.onopen = () => {
        this.isConnecting = false;
        this.handleOpen();
      };
      this.ws.onmessage = this.handleMessage.bind(this);
      this.ws.onclose = (event) => {
        this.isConnecting = false;
        this.handleClose(event);
      };
      this.ws.onerror = (error) => {
        this.isConnecting = false;
        this.handleError(error);
      };
    } catch (error) {
      this.isConnecting = false;
      console.error('[WebSocket] Failed to create connection:', error);
    }
  }

  /**
   * Disconnect from WebSocket server
   */
  disconnect(): void {
    this.isIntentionallyClosed = true;
    this.isConnecting = false;
    
    // Clear any pending reconnect timer
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    
    this.stopHeartbeat();
    
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    
    console.log('[WebSocket] Disconnected');
  }

  /**
   * Send a message to the server
   */
  send(type: string, payload?: any): void {
    if (this.ws?.readyState !== WebSocket.OPEN) {
      console.warn('[WebSocket] Cannot send, not connected');
      return;
    }

    const message = JSON.stringify({ type, ...payload });
    this.ws.send(message);
  }

  /**
   * Send ping to keep connection alive
   */
  ping(): void {
    this.send('ping');
  }

  /**
   * Subscribe to a channel
   */
  subscribe(channel: string): void {
    this.send('subscribe', { channel });
  }

  /**
   * Unsubscribe from a channel
   */
  unsubscribe(channel: string): void {
    this.send('unsubscribe', { channel });
  }

  /**
   * Mark alert as read
   */
  markAlertRead(alertId: string): void {
    this.send('mark_alert_read', { alert_id: alertId });
  }

  /**
   * Register an event handler
   */
  on(event: WsMessageType, handler: WsEventHandler): () => void {
    if (!this.eventHandlers.has(event)) {
      this.eventHandlers.set(event, new Set());
    }
    this.eventHandlers.get(event)!.add(handler);

    // Return unsubscribe function
    return () => {
      this.eventHandlers.get(event)?.delete(handler);
    };
  }

  /**
   * Check if connected
   */
  get isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  private handleOpen(): void {
    console.log('[WebSocket] Connected');
    this.reconnectAttempts = 0;
    this.reconnectDelay = 1000;
    this.startHeartbeat();
    
    // Subscribe to default channels
    this.subscribe('alerts');
    this.subscribe('transactions');
  }

  private handleMessage(event: MessageEvent): void {
    try {
      const message: WsMessage = JSON.parse(event.data);
      console.log('[WebSocket] Received:', message.type);

      // Emit to all handlers for this message type
      const handlers = this.eventHandlers.get(message.type);
      if (handlers) {
        handlers.forEach(handler => {
          try {
            handler(message);
          } catch (e) {
            console.error('[WebSocket] Handler error:', e);
          }
        });
      }

      // Also emit to wildcard handlers
      const wildcardHandlers = this.eventHandlers.get('*' as WsMessageType);
      if (wildcardHandlers) {
        wildcardHandlers.forEach(handler => {
          try {
            handler(message);
          } catch (e) {
            console.error('[WebSocket] Wildcard handler error:', e);
          }
        });
      }
    } catch (e) {
      console.error('[WebSocket] Failed to parse message:', e);
    }
  }

  private handleClose(event: CloseEvent): void {
    console.log('[WebSocket] Closed:', event.code, event.reason);
    this.stopHeartbeat();

    if (!this.isIntentionallyClosed && this.reconnectAttempts < this.maxReconnectAttempts) {
      this.attemptReconnect();
    }
  }

  private handleError(error: Event): void {
    console.error('[WebSocket] Error:', error);
  }

  private attemptReconnect(): void {
    this.reconnectAttempts++;
    console.log(`[WebSocket] Reconnecting... (${this.reconnectAttempts}/${this.maxReconnectAttempts})`);

    this.reconnectTimer = setTimeout(() => {
      this.connect();
    }, this.reconnectDelay);

    // Exponential backoff
    this.reconnectDelay = Math.min(this.reconnectDelay * 2, 30000);
  }

  private startHeartbeat(): void {
    this.heartbeatInterval = setInterval(() => {
      this.ping();
    }, 30000); // Ping every 30 seconds
  }

  private stopHeartbeat(): void {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
      this.heartbeatInterval = null;
    }
  }
}

// Singleton instance
let wsClient: WebSocketClient | null = null;

export function getWebSocketClient(): WebSocketClient {
  if (!wsClient) {
    wsClient = new WebSocketClient();
  }
  return wsClient;
}

export function initWebSocket(): WebSocketClient {
  const client = getWebSocketClient();
  client.connect();
  return client;
}

export function closeWebSocket(): void {
  wsClient?.disconnect();
  wsClient = null;
}
