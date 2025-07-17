package main

import (
	"context"
	"errors"
	"flag"
	"html/template"
	"log/slog"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"mhumbleweb/internal/models"
)

type application struct {
	logger        *slog.Logger
	posts         models.PostsModel
	templateCache map[string]*template.Template
}

func main() {
	logger := slog.New(slog.NewTextHandler(os.Stdout, &slog.HandlerOptions{
		Level:     slog.LevelInfo,
		AddSource: true,
	}))

	addr := flag.String("addr", ":8080", "HTTPS network address")

	flag.Parse()

	templateCache, err := newTemplateCache()
	if err != nil {
		logger.Error("failed to create template cache", "error", err)
		os.Exit(1)
	}

	app := &application{
		logger:        logger,
		posts:         models.PostsModel{},
		templateCache: templateCache,
	}

	srv := &http.Server{
		Addr:         *addr,
		Handler:      app.routes(),
		ErrorLog:     slog.NewLogLogger(logger.Handler(), slog.LevelError),
		IdleTimeout:  time.Minute,
		ReadTimeout:  5 * time.Second,
		WriteTimeout: 10 * time.Second,
	}

	// Channel to listen for interrupt signal for graceful shutdown
	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)

	// Start server in a goroutine
	go func() {
		logger.Info("starting HTTP server", "addr", srv.Addr)
		err := srv.ListenAndServe()
		if err != nil && !errors.Is(err, http.ErrServerClosed) {
			logger.Error("server failed to start", "error", err)
			os.Exit(1)
		}
	}()

	logger.Info("server started successfully")

	// Wait for interrupt signal to gracefully shut down the server
	<-quit
	logger.Info("shutting down server...")

	// Give outstanding requests 30 seconds to complete
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	if err := srv.Shutdown(ctx); err != nil {
		logger.Error("server forced to shutdown", "error", err)
		os.Exit(1)
	}

	logger.Info("server exited gracefully")
}
