/**
 * API Client Tests
 */

import { apiClient, ApiError } from '@/lib/api/client';

// Mock fetch
global.fetch = jest.fn();

const mockFetch = fetch as jest.MockedFunction<typeof fetch>;

const API_BASE_URL = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:3000';

describe('ApiClient', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    localStorage.clear();
  });

  describe('Authentication', () => {
    it('should login successfully', async () => {
      const mockResponse = {
        success: true,
        data: {
          user: {
            id: '123',
            email: 'test@example.com',
            full_name: 'Test User',
          },
          token: 'mock_jwt_token',
        },
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      } as Response);

      const result = await apiClient.login('test@example.com', 'password');

      expect(result).toEqual(mockResponse);
      expect(mockFetch).toHaveBeenCalledWith(
        `${API_BASE_URL}/api/v1/auth/login`,
        expect.objectContaining({
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ email: 'test@example.com', password: 'password' }),
        })
      );
    });

    it('should throw ApiError on login failure', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 401,
        json: async () => ({ message: 'Invalid credentials' }),
      } as Response);

      await expect(apiClient.login('test@example.com', 'wrong'))
        .rejects
        .toThrow(ApiError);
    });

    it('should register successfully', async () => {
      const mockResponse = {
        success: true,
        data: {
          id: '123',
          email: 'new@example.com',
          full_name: 'New User',
        },
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      } as Response);

      const result = await apiClient.register({
        email: 'new@example.com',
        password: 'password',
        full_name: 'New User',
      });

      expect(result).toEqual(mockResponse);
    });
  });

  describe('Transactions', () => {
    it('should get transactions list', async () => {
      localStorage.setItem('token', 'mock_token');

      const mockResponse = {
        success: true,
        data: [
          { id: '1', total_amount: 100, quantity: 2 },
          { id: '2', total_amount: 50, quantity: 1 },
        ],
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      } as Response);

      const result = await apiClient.getTransactions();

      expect(result).toEqual(mockResponse);
      expect(mockFetch).toHaveBeenCalledWith(
        `${API_BASE_URL}/api/v1/transactions?limit=50&offset=0`,
        expect.objectContaining({
          headers: expect.objectContaining({
            'Authorization': 'Bearer mock_token',
          }),
        })
      );
    });

    it('should create transaction', async () => {
      localStorage.setItem('token', 'mock_token');

      const mockResponse = {
        success: true,
        data: { id: '1', total_amount: 100 },
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      } as Response);

      const result = await apiClient.createTransaction({
        product_id: 'prod-1',
        quantity: 2,
        price: 50,
        total_amount: 100,
      });

      expect(result).toEqual(mockResponse);
    });

    it('should create voice transaction', async () => {
      localStorage.setItem('token', 'mock_token');

      const mockResponse = {
        success: true,
        data: {
          transaction: { id: '1', total_amount: 100 },
          transcription: 'Sold 2 shirts for 50 dollars each',
        },
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      } as Response);

      const result = await apiClient.createVoiceTransaction('base64_audio_data');

      expect(result).toEqual(mockResponse);
    });
  });

  describe('Analytics', () => {
    it('should get analytics summary', async () => {
      localStorage.setItem('token', 'mock_token');

      const mockResponse = {
        success: true,
        data: {
          total_revenue: 1000,
          total_transactions: 10,
          total_items_sold: 25,
          average_transaction_value: 100,
        },
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      } as Response);

      const result = await apiClient.getAnalyticsSummary(7);

      expect(result).toEqual(mockResponse);
      expect(mockFetch).toHaveBeenCalledWith(
        `${API_BASE_URL}/api/v1/analytics/summary?days=7`,
        expect.any(Object)
      );
    });
  });

  describe('Products', () => {
    it('should get products list', async () => {
      localStorage.setItem('token', 'mock_token');

      const mockResponse = {
        success: true,
        data: [
          { id: '1', name: 'Product 1', price: 50 },
          { id: '2', name: 'Product 2', price: 100 },
        ],
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      } as Response);

      const result = await apiClient.getProducts();

      expect(result).toEqual(mockResponse);
    });
  });

  describe('Error Handling', () => {
    it('should throw ApiError with correct status', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 403,
        json: async () => ({ message: 'Forbidden' }),
      } as Response);

      try {
        await apiClient.getProducts();
        fail('Should have thrown ApiError');
      } catch (error) {
        expect(error).toBeInstanceOf(ApiError);
        if (error instanceof ApiError) {
          expect(error.status).toBe(403);
          expect(error.message).toBe('Forbidden');
        }
      }
    });

    it('should handle network errors', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network error'));

      await expect(apiClient.getProducts()).rejects.toThrow();
    });
  });
});

describe('ApiError', () => {
  it('should create ApiError with message and status', () => {
    const error = new ApiError('Test error', 500);
    expect(error.message).toBe('Test error');
    expect(error.status).toBe(500);
    expect(error.name).toBe('ApiError');
  });
});
