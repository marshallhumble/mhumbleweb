package main

import (
	"bytes"
	"fmt"
	"net/http"
	"runtime/debug"
	"time"
)

func (app *application) serverError(w http.ResponseWriter, r *http.Request, err error) {
	var (
		method = r.Method
		uri    = r.URL.RequestURI()
		trace  = string(debug.Stack())
	)

	app.logger.Error(err.Error(), "method", method, "uri", uri, "trace", trace)
	http.Error(w, http.StatusText(http.StatusInternalServerError), http.StatusInternalServerError)
}

func (app *application) clientError(w http.ResponseWriter, status int) {
	http.Error(w, http.StatusText(status), status)
}

// Add a notFound helper for cleaner 404 handling
func (app *application) notFound(w http.ResponseWriter) {
	app.clientError(w, http.StatusNotFound)
}

func (app *application) newTemplateData(r *http.Request) templateData {
	now := time.Now() // Single time call for consistency

	return templateData{
		Title:          "Marshall Humble", // Default title
		CurrentYear:    now.Year(),
		CurrentMonth:   int(now.Month()),
		CurrentDay:     now.Day(),
		CurrentDoW:     now.Weekday(),
		CurrentHour:    now.Hour(),
		CurrentMinutes: now.Minute(),
	}
}

func (app *application) render(w http.ResponseWriter, r *http.Request, status int, page string, data templateData) {
	ts, ok := app.templateCache[page]
	if !ok {
		err := fmt.Errorf("the template %s does not exist", page)
		app.serverError(w, r, err)
		return
	}

	// Initialize a new buffer.
	buf := new(bytes.Buffer)

	// Write the template to the buffer, instead of straight to the
	// http.ResponseWriter. If there's an error, call our serverError() helper
	// and then return.
	err := ts.ExecuteTemplate(buf, "base", data)
	if err != nil {
		app.serverError(w, r, err)
		return
	}

	// Remove this debug print or make it conditional
	// fmt.Println(data.Post.Content) // Remove this line

	// Set content type header before writing status
	w.Header().Set("Content-Type", "text/html; charset=utf-8")

	// If the template is written to the buffer without any errors, we are safe
	// to go ahead and write the HTTP status code to http.ResponseWriter.
	w.WriteHeader(status)

	// Write the contents of the buffer to the http.ResponseWriter. Note: this
	// is another time where we pass our http.ResponseWriter to a function that
	// takes an io.Writer.
	_, err = buf.WriteTo(w)
	if err != nil {
		// Log the error but don't call serverError since headers are already sent
		app.logger.Error("error writing response", "error", err)
	}
}

// Optional: Add a method to handle JSON responses
func (app *application) writeJSON(w http.ResponseWriter, status int, data interface{}) error {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)

	// You'd implement JSON marshaling here if needed
	return nil
}

// Optional: Add a method for redirects with logging
func (app *application) redirect(w http.ResponseWriter, r *http.Request, url string, status int) {
	app.logger.Info("redirecting", "from", r.URL.RequestURI(), "to", url, "status", status)
	http.Redirect(w, r, url, status)
}
