package main

import "net/http"

func (app *application) home(w http.ResponseWriter, r *http.Request) {
	w.Write([]byte("Welcome to the Home!"))
}

func (app *application) about(w http.ResponseWriter, r *http.Request) {
	w.Write([]byte("Welcome to the About Page!"))
}

func (app *application) contact(w http.ResponseWriter, r *http.Request) {
	w.Write([]byte("Welcome to the Contact Page!"))
}
