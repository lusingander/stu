package aws

import (
	"context"
	"strings"

	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/config"
	"github.com/aws/aws-sdk-go-v2/service/s3"
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

type ObjectItem struct {
	Dir   bool
	name  string
	paths []string
}

func (i *ObjectItem) Text() string {
	name := i.Filename()
	if i.Dir {
		name += delimiter
	}
	return name
}

func (i *ObjectItem) FilterValue() string {
	return i.Filename()
}

func (i *ObjectItem) ObjectKey() string {
	return i.name
}

func (i *ObjectItem) Filename() string {
	return i.paths[len(i.paths)-1]
}

func (c *S3Client) ListObjects(bucket, prefix string) ([]*ObjectItem, error) {
	input := &s3.ListObjectsV2Input{
		Bucket:    aws.String(bucket),
		Delimiter: aws.String(delimiter),
		Prefix:    aws.String(prefix),
	}
	p := s3.NewListObjectsV2Paginator(c.client, input, func(o *s3.ListObjectsV2PaginatorOptions) {})
	items := make([]*ObjectItem, 0)
	for p.HasMorePages() {
		output, err := p.NextPage(c.ctx)
		if err != nil {
			return nil, err
		}
		for _, obj := range output.Contents {
			key := *obj.Key
			item := &ObjectItem{
				Dir:   false,
				name:  key,
				paths: parseObjectKey(key, false),
			}
			items = append(items, item)
		}
		for _, cp := range output.CommonPrefixes {
			key := *cp.Prefix
			item := &ObjectItem{
				Dir:   true,
				name:  key,
				paths: parseObjectKey(key, true),
			}
			items = append(items, item)
		}
	}
	return items, nil
}

func parseObjectKey(key string, dir bool) []string {
	ss := strings.Split(key, delimiter)
	if dir {
		li := len(ss) - 2 // foo/bar/baz/ => ["foo", "bar", "baz", ""]
		return append(ss[:li], ss[li])
	}
	return ss
}

type BucketItem struct {
	name string
}

func (i *BucketItem) Text() string {
	return i.name
}

func (i *BucketItem) FilterValue() string {
	return i.name
}

func (i *BucketItem) BucketName() string {
	return i.name
}

func (c *S3Client) ListBuckets() ([]*BucketItem, error) {
	input := &s3.ListBucketsInput{}
	output, err := c.client.ListBuckets(c.ctx, input)
	if err != nil {
		return nil, err
	}
	items := make([]*BucketItem, 0)
	for _, bucket := range output.Buckets {
		item := &BucketItem{name: *bucket.Name}
		items = append(items, item)
	}
	return items, nil
}