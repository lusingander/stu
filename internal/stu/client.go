package stu

import "strings"

const (
	delimiter = "/"
)

type Client interface {
	ListObjects(bucket, prefix string) ([]*ObjectItem, error)
	ListBuckets() ([]*BucketItem, error)
}

type ObjectItem struct {
	Dir   bool
	name  string
	paths []string
}

func NewFileObjectItem(key string) *ObjectItem {
	return &ObjectItem{
		Dir:   false,
		name:  key,
		paths: parseObjectKey(key, false),
	}
}

func NewDirObjectItem(key string) *ObjectItem {
	return &ObjectItem{
		Dir:   true,
		name:  key,
		paths: parseObjectKey(key, true),
	}
}

func parseObjectKey(key string, dir bool) []string {
	ss := strings.Split(key, delimiter)
	if dir {
		li := len(ss) - 2 // foo/bar/baz/ => ["foo", "bar", "baz", ""]
		return append(ss[:li], ss[li])
	}
	return ss
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

type BucketItem struct {
	name string
}

func NewBucketItem(name string) *BucketItem {
	return &BucketItem{
		name: name,
	}
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
