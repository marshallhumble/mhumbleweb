package models

import (
	"encoding/json"
	"fmt"
	"html/template"
	"log"
	"os"
	"sort"
	"time"
)

type PostsInterface interface {
	GetAll() ([]Post, error)
	GetById(id int) (string, error)
}

type Post struct {
	Id      int
	Title   string
	Content template.HTML
	Created time.Time
	Updated time.Time
	Topic   string
}

type PostsModel struct {
}

func (pm PostsModel) GetAll() ([]Post, error) {
	var posts []Post

	j, err := os.ReadFile("internal/models/json/data.json")
	if err != nil {
		log.Fatalf("Unable to open file due to %s", err)
	}

	err = json.Unmarshal(j, &posts)
	if err != nil {
		log.Fatalf("Error marshaling json %s", err)
	}

	//Sort by time
	return sortByTime(posts)

}

func (pm PostsModel) GetById(id int) (Post, error) {
	var posts []Post

	j, err := os.ReadFile("internal/models/json/data.json")
	if err != nil {
		log.Fatalf("Unable to open file due to %s", err)
	}

	err = json.Unmarshal(j, &posts)
	if err != nil {
		log.Fatalf("Error marshaling json %s", err)
	}

	fmt.Println()

	if id < 1 || id > len(posts) {
		return posts[len(posts)-1], nil
	}
	return posts[id-1], nil
}

func sortByTime(posts []Post) ([]Post, error) {
	sort.Slice(posts, func(i, j int) bool {
		return posts[i].Created.After(posts[j].Created)
	})

	return posts, nil
}
