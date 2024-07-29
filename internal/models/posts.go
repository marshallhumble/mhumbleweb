package models

import (
	"database/sql"
	"html/template"
)

type PostsInterface interface {
	GetAll() ([]Post, error)
	GetById(id int) (Post, error)
}

type Post struct {
	ID      int
	Title   string
	Content template.HTML
	Created string
	Updated string
}

type PostsModel struct {
	DB *sql.DB
}

func (pm PostsModel) GetAll() ([]Post, error) {
	var posts []Post
	rows, err := pm.DB.Query("SELECT * FROM posts")
	if err != nil {
		return posts, err
	}

	for rows.Next() {
		var post Post
		err = rows.Scan(&post.ID, &post.Title, &post.Content, &post.Created, &post.Updated)
		if err != nil {
			return posts, err
		}
		posts = append(posts, post)
	}

	return posts, nil
}

func (pm PostsModel) GetById(id int) (Post, error) {
	var post Post
	if err := pm.DB.QueryRow("SELECT * FROM posts WHERE id = $1", id).Scan(&post.ID, &post.Title, &post.Content,
		&post.Created, &post.Updated); err != nil {
		return post, err
	}

	return post, nil
}
