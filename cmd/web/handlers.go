package main

import "net/http"

func (app *application) home(w http.ResponseWriter, r *http.Request) {
	data := app.newTemplateData(r)
	app.render(w, r, http.StatusOK, "home.gohtml", data)
	return
}

func (app *application) about(w http.ResponseWriter, r *http.Request) {
	data := app.newTemplateData(r)
	app.render(w, r, http.StatusOK, "about.gohtml", data)
	return
}

func (app *application) articles(w http.ResponseWriter, r *http.Request) {
	data := app.newTemplateData(r)
	app.render(w, r, http.StatusOK, "articles.gohtml", data)
	return
}
