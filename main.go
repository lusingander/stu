package main

import (
	"fmt"
	"log"
	"os"

	"github.com/lusingander/stu/internal/aws"
)

func run(args []string) error {
	client, err := aws.NewS3Client()
	if err != nil {
		return err
	}
	buckets, err := client.ListBuckets()
	if err != nil {
		return err
	}
	for _, bucket := range buckets {
		fmt.Println(bucket.Name)
	}
	objs, err := client.ListObjects("test-bucket", "")
	// objs, err := client.listObjects("test-bucket", "hoge/")
	if err != nil {
		return err
	}
	for _, obj := range objs {
		fmt.Println(obj.Name)
	}
	return nil
}

func main() {
	if err := run(os.Args); err != nil {
		log.Fatal(err)
	}
}
