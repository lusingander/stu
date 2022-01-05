package main

import (
	"log"
	"os"

	"github.com/charmbracelet/bubbles/list"
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
	buckets, err := client.ListBuckets()
	if err != nil {
		return err
	}

	items := make([]list.Item, len(buckets))
	for i, bucket := range buckets {
		items[i] = bucket
	}

	return ui.Start(client, items)
}

func main() {
	if err := run(os.Args); err != nil {
		log.Fatal(err)
	}
}
