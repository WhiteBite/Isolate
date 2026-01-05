// Client-side error handling for better debugging
import type { HandleClientError } from '@sveltejs/kit';

export const handleError: HandleClientError = ({ error, event, status, message }) => {
  // Log to console with full details
  console.error('=== CLIENT ERROR ===');
  console.error('Status:', status);
  console.error('Message:', message);
  console.error('URL:', event.url.pathname);
  
  if (error instanceof Error) {
    console.error('Error:', error.message);
    console.error('Stack:', error.stack);
    
    // Return detailed error for development
    return {
      message: error.message,
      stack: error.stack,
      status,
      url: event.url.pathname
    };
  }
  
  console.error('Error:', error);
  
  return {
    message: message || 'Unknown error',
    status
  };
};
