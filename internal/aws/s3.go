package aws

import (
	"context"

	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/config"
	"github.com/aws/aws-sdk-go-v2/service/s3"
	"github.com/lusingander/stu/internal/stu"
)

const (
	localstackUrl = "http://localhost:4572"
	region        = "ap-northeast-1"
	delimiter     = "/"
)

type S3Client struct {
	client *s3.Client
	ctx    context.Context
}

func NewS3Client() (*S3Client, error) {
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
	return &S3Client{
		client: client,
		ctx:    ctx,
	}, nil
}

func (c *S3Client) ListObjects(bucket, prefix string) ([]*stu.ObjectItem, error) {
	input := &s3.ListObjectsV2Input{
		Bucket:    aws.String(bucket),
		Delimiter: aws.String(delimiter),
		Prefix:    aws.String(prefix),
	}
	p := s3.NewListObjectsV2Paginator(c.client, input, func(o *s3.ListObjectsV2PaginatorOptions) {})
	items := make([]*stu.ObjectItem, 0)
	for p.HasMorePages() {
		output, err := p.NextPage(c.ctx)
		if err != nil {
			return nil, err
		}
		for _, obj := range output.Contents {
			item := stu.NewFileObjectItem(*obj.Key)
			items = append(items, item)
		}
		for _, cp := range output.CommonPrefixes {
			item := stu.NewDirObjectItem(*cp.Prefix)
			items = append(items, item)
		}
	}
	return items, nil
}

func (c *S3Client) ListBuckets() ([]*stu.BucketItem, error) {
	input := &s3.ListBucketsInput{}
	output, err := c.client.ListBuckets(c.ctx, input)
	if err != nil {
		return nil, err
	}
	items := make([]*stu.BucketItem, 0)
	for _, bucket := range output.Buckets {
		item := stu.NewBucketItem(*bucket.Name)
		items = append(items, item)
	}
	return items, nil
}
