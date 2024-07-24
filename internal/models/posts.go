package models

import (
	"database/sql"
	"html/template"
)

type PostsInterface interface {
}

type Post struct {
	ID      int
	Title   string
	Content template.HTML
}

type PostsModel struct {
	DB *sql.DB
}
