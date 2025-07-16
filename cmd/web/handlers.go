package main

import (
	"net/http"
	"strconv"
)

func (app *application) home(w http.ResponseWriter, r *http.Request) {
	data := app.newTemplateData(r)
	data.Title = "Marshall Humble - Software Engineer & Technology Leader"
	app.render(w, r, http.StatusOK, "home.gohtml", data)
}

func (app *application) about(w http.ResponseWriter, r *http.Request) {
	data := app.newTemplateData(r)
	data.Title = "About - Marshall Humble"
	app.render(w, r, http.StatusOK, "about.gohtml", data)
}

func (app *application) articles(w http.ResponseWriter, r *http.Request) {
	data := app.newTemplateData(r)
	data.Title = "Articles - Marshall Humble"

	Posts, err := app.posts.GetAll()
	if err != nil {
		app.serverError(w, r, err)
		return
	}

	data.Posts = Posts

	app.render(w, r, http.StatusOK, "articles.gohtml", data)
}

func (app *application) getArticle(w http.ResponseWriter, r *http.Request) {
	id, err := strconv.Atoi(r.PathValue("id"))
	if err != nil || id < 1 {
		http.NotFound(w, r)
		return
	}

	data := app.newTemplateData(r)

	post, err := app.posts.GetById(id)
	if err != nil {
		app.logger.Error("Server error:", "Error", err)
		http.Redirect(w, r, "/articles/", http.StatusSeeOther)
		return
	}

	data.Post = post
	data.Title = post.Title + " - Marshall Humble"

	app.render(w, r, http.StatusOK, string(post.Content), data)
}
