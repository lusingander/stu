package main

import (
	"log"
	"os"

	"github.com/lusingander/stu/internal/aws"
	"github.com/lusingander/stu/internal/ui"
	"github.com/mattn/go-runewidth"
)

func setup() {
	runewidth.DefaultCondition = &runewidth.Condition{EastAsianWidth: false}
}

func run(args []string) error {
	setup()
	client, err := aws.NewS3Client()
	if err != nil {
		return err
	}
	return ui.Start(client)
}

func main() {
	if err := run(os.Args); err != nil {
		log.Fatal(err)
	}
}
