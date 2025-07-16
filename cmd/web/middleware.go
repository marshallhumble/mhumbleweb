package main

import (
	"fmt"
	"net/http"
	"strings"
)

func securityHeaders(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Content Security Policy - more comprehensive
		csp := strings.Join([]string{
			"default-src 'self'",
			"script-src 'self' https://kit.fontawesome.com https://ka-f.fontawesome.com",
			"style-src 'self' 'unsafe-inline' https://kit.fontawesome.com https://ka-f.fontawesome.com",
			"font-src 'self' https://kit.fontawesome.com https://ka-f.fontawesome.com",
			"img-src 'self' data:",
			"connect-src 'self'",
			"frame-ancestors 'none'", // More modern than X-Frame-Options
			"base-uri 'self'",
			"form-action 'self'",
		}, "; ")

		w.Header().Set("Content-Security-Policy", csp)

		// Security headers
		w.Header().Set("Strict-Transport-Security", "max-age=63072000; includeSubDomains; preload")
		w.Header().Set("X-Content-Type-Options", "nosniff")
		w.Header().Set("X-Frame-Options", "DENY") // Backup for older browsers
		w.Header().Set("X-XSS-Protection", "0")   // Correct - disables legacy XSS filter
		w.Header().Set("Referrer-Policy", "strict-origin-when-cross-origin")

		// Permissions Policy (formerly Feature-Policy)
		w.Header().Set("Permissions-Policy", "camera=(), microphone=(), geolocation=(), payment=()")

		next.ServeHTTP(w, r)
	})
}

func (app *application) logRequest(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Get real IP address (handles proxies/load balancers)
		ip := getRealIP(r)

		var (
			proto     = r.Proto
			method    = r.Method
			uri       = r.URL.RequestURI()
			userAgent = r.Header.Get("User-Agent")
		)

		app.logger.Info("received request",
			"ip", ip,
			"proto", proto,
			"method", method,
			"uri", uri,
			"user_agent", userAgent,
		)

		next.ServeHTTP(w, r)
	})
}

// Helper function to get real IP address
func getRealIP(r *http.Request) string {
	// Check X-Forwarded-For header (most common)
	if xff := r.Header.Get("X-Forwarded-For"); xff != "" {
		// Take the first IP (client IP)
		ips := strings.Split(xff, ",")
		return strings.TrimSpace(ips[0])
	}

	// Check X-Real-IP header
	if xri := r.Header.Get("X-Real-IP"); xri != "" {
		return xri
	}

	// Check CF-Connecting-IP (Cloudflare)
	if cfip := r.Header.Get("CF-Connecting-IP"); cfip != "" {
		return cfip
	}

	// Fall back to RemoteAddr
	return r.RemoteAddr
}

func (app *application) recoverPanic(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		defer func() {
			if err := recover(); err != nil {
				// Log the panic with more detail
				app.logger.Error("panic recovered",
					"error", err,
					"method", r.Method,
					"url", r.URL.String(),
					"remote_addr", r.RemoteAddr,
				)

				// Set connection close header
				w.Header().Set("Connection", "close")

				// Return 500 error
				app.serverError(w, r, fmt.Errorf("panic: %v", err))
			}
		}()

		next.ServeHTTP(w, r)
	})
}
