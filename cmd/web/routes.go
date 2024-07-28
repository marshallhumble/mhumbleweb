package main

import (
	"mhumbleweb/ui"
	"net/http"
	"path/filepath"

	//External
	"github.com/justinas/alice"
)

type neuteredFileSystem struct {
	fs http.FileSystem
}

func (app *application) routes() http.Handler {
	mux := http.NewServeMux()

	//Create file system for static files
	fileServer := http.FileServer(neuteredFileSystem{http.Dir("./ui/static")})
	mux.Handle("/static", http.NotFoundHandler())
	mux.Handle("/static/", http.StripPrefix("/static", fileServer))
	mux.Handle("GET /static/", http.FileServerFS(ui.Files))

	mux.HandleFunc("/{$}", app.home)
	mux.HandleFunc("GET /about/", app.about)
	mux.HandleFunc("GET /articles/", app.articles)
	mux.HandleFunc("GET /articles/{id}", app.getArticle)

	standard := alice.New(app.recoverPanic, app.logRequest, commonHeaders)
	return standard.Then(mux)
}

// Open don't show the contents of a directory, if we have an index file show that.
func (nfs neuteredFileSystem) Open(path string) (http.File, error) {
	f, err := nfs.fs.Open(path)
	if err != nil {
		return nil, err
	}

	s, err := f.Stat()
	if s.IsDir() {
		index := filepath.Join(path, "index.css")
		if _, err := nfs.fs.Open(index); err != nil {
			closeErr := f.Close()
			if closeErr != nil {
				return nil, closeErr
			}

			return nil, err
		}
	}

	return f, nil
}
