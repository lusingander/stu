package main

import (
	"context"
	"fmt"
	"log"
	"os"

	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/config"
	"github.com/aws/aws-sdk-go-v2/service/s3"
)

const (
	localstackUrl = "http://localhost:4572"
	region        = "ap-northeast-1"
	delimiter     = "/"
)

type s3Client struct {
	client *s3.Client
	ctx    context.Context
}

func newS3Client() (*s3Client, error) {
	ctx := context.Background()
	customResolver := aws.EndpointResolverWithOptionsFunc(func(service, region string, options ...interface{}) (aws.Endpoint, error) {
		return aws.Endpoint{
			URL:           localstackUrl,
			SigningRegion: region,
		}, nil
	})
	cfg, err := config.LoadDefaultConfig(ctx)
	cfg.EndpointResolverWithOptions = customResolver
	if err != nil {
		return nil, err
	}
	client := s3.NewFromConfig(cfg, func(o *s3.Options) {
		o.UsePathStyle = true
	})
	return &s3Client{
		client: client,
		ctx:    ctx,
	}, nil
}

type objectItem struct {
	dir  bool
	name string
}

func (c *s3Client) listObjects(bucket, prefix string) ([]*objectItem, error) {
	input := &s3.ListObjectsV2Input{
		Bucket:    aws.String(bucket),
		Delimiter: aws.String(delimiter),
		Prefix:    aws.String(prefix),
	}
	p := s3.NewListObjectsV2Paginator(c.client, input, func(o *s3.ListObjectsV2PaginatorOptions) {})
	items := make([]*objectItem, 0)
	for p.HasMorePages() {
		output, err := p.NextPage(c.ctx)
		if err != nil {
			return nil, err
		}
		for _, obj := range output.Contents {
			item := &objectItem{
				dir:  false,
				name: *obj.Key,
			}
			items = append(items, item)
		}
		for _, cp := range output.CommonPrefixes {
			item := &objectItem{
				dir:  true,
				name: *cp.Prefix,
			}
			items = append(items, item)
		}
	}
	return items, nil
}

type bucketItem struct {
	name string
}

func (c *s3Client) listBuckets() ([]*bucketItem, error) {
	input := &s3.ListBucketsInput{}
	output, err := c.client.ListBuckets(c.ctx, input)
	if err != nil {
		return nil, err
	}
	items := make([]*bucketItem, 0)
	for _, bucket := range output.Buckets {
		item := &bucketItem{name: *bucket.Name}
		items = append(items, item)
	}
	return items, nil
}

func run(args []string) error {
	client, err := newS3Client()
	if err != nil {
		return err
	}
	buckets, err := client.listBuckets()
	if err != nil {
		return err
	}
	for _, bucket := range buckets {
		fmt.Println(bucket.name)
	}
	objs, err := client.listObjects("test-bucket", "")
	// objs, err := client.listObjects("test-bucket", "hoge/")
	if err != nil {
		return err
	}
	for _, obj := range objs {
		fmt.Println(obj.name)
	}
	return nil
}

func main() {
	if err := run(os.Args); err != nil {
		log.Fatal(err)
	}
}
