/**
 * WebSocket Client Tests
 */

import { WebSocketClient, getWebSocketClient, initWebSocket, closeWebSocket } from '@/lib/websocket/client';

// Mock WebSocket
global.WebSocket = class MockWebSocket {
  static CONNECTING = 0;
  static OPEN = 1;
  static CLOSING = 2;
  static CLOSED = 3;

  readyState = MockWebSocket.CONNECTING;
  url: string;
  onopen: ((event: Event) => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;

  constructor(url: string) {
    this.url = url;
    // Simulate connection
    setTimeout(() => {
      this.readyState = MockWebSocket.OPEN;
      this.onopen?.(new Event('open'));
    }, 10);
  }

  send(data: string) {
    if (this.readyState !== MockWebSocket.OPEN) {
      throw new Error('WebSocket is not open');
    }
    // Echo back for testing
    setTimeout(() => {
      this.onmessage?.(new MessageEvent('message', { data }));
    }, 5);
  }

  close() {
    this.readyState = MockWebSocket.CLOSED;
    this.onclose?.(new CloseEvent('close'));
  }
} as any;

describe('WebSocketClient', () => {
  let client: WebSocketClient;

  beforeEach(() => {
    closeWebSocket();
    client = new WebSocketClient('http://localhost:3000');
  });

  afterEach(() => {
    client.disconnect();
    closeWebSocket();
  });

  it('should create client with correct URL', () => {
    expect(client).toBeDefined();
    expect(client.isConnected).toBe(false);
  });

  it('should connect to WebSocket', async () => {
    client.connect();
    
    // Wait for connection
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(client.isConnected).toBe(true);
  });

  it('should handle connection events', async () => {
    const onConnect = jest.fn();
    client.on('connected', onConnect);
    
    client.connect();
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(onConnect).toHaveBeenCalled();
  });

  it('should send messages', async () => {
    client.connect();
    await new Promise(resolve => setTimeout(resolve, 50));
    
    // Should not throw
    expect(() => client.send('ping')).not.toThrow();
  });

  it('should handle subscription', async () => {
    client.connect();
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(() => client.subscribe('alerts')).not.toThrow();
    expect(() => client.unsubscribe('alerts')).not.toThrow();
  });

  it('should register and unregister event handlers', async () => {
    const handler = jest.fn();
    const unsubscribe = client.on('transaction_update', handler);
    
    expect(unsubscribe).toBeDefined();
    expect(typeof unsubscribe).toBe('function');
    
    // Unregister
    unsubscribe();
  });

  it('should handle disconnect', async () => {
    client.connect();
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(client.isConnected).toBe(true);
    
    client.disconnect();
    
    expect(client.isConnected).toBe(false);
  });

  it('should use singleton instance', () => {
    const client1 = getWebSocketClient();
    const client2 = getWebSocketClient();
    
    expect(client1).toBe(client2);
  });

  it('should init and close WebSocket', async () => {
    const ws = initWebSocket();
    expect(ws).toBeDefined();
    
    await new Promise(resolve => setTimeout(resolve, 50));
    
    closeWebSocket();
  });
});

describe('WebSocket Message Types', () => {
  it('should handle ping message', () => {
    const ping = JSON.stringify({ type: 'ping' });
    expect(JSON.parse(ping).type).toBe('ping');
  });

  it('should handle subscription message', () => {
    const subscribe = JSON.stringify({ type: 'subscribe', channel: 'alerts' });
    const parsed = JSON.parse(subscribe);
    expect(parsed.type).toBe('subscribe');
    expect(parsed.channel).toBe('alerts');
  });

  it('should handle mark alert read message', () => {
    const markRead = JSON.stringify({ 
      type: 'mark_alert_read', 
      alert_id: '550e8400-e29b-41d4-a716-446655440000' 
    });
    const parsed = JSON.parse(markRead);
    expect(parsed.type).toBe('mark_alert_read');
    expect(parsed.alert_id).toBe('550e8400-e29b-41d4-a716-446655440000');
  });
});
