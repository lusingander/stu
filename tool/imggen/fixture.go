package main

import (
	"bytes"
	_ "embed"
	"fmt"
	"image"
	"image/color"
	"image/png"
	"strings"
)

type bucket struct {
	name string
	objs []object
}

func newBucket(name string, objs ...object) bucket {
	return bucket{name, objs}
}

type object struct {
	objectKey       string
	objectType      fixtureObjectType
	multipleVersion bool
}

type fixtureObjectType int

type fixtureObjectMap map[fixtureObjectType][]byte

const (
	smallText fixtureObjectType = iota
	mediumText
	largeText
	smallHtml
	rustCode
	imagePng
	imageJpg
	empty
)

func buildObjectMap() fixtureObjectMap {
	return fixtureObjectMap{
		smallText:  smallTextObject(),
		mediumText: mediumTextObject(),
		largeText:  largeTextObject(),
		smallHtml:  smallHtmlObject(),
		rustCode:   rustCodeObject(),
		imagePng:   imagePngObject(),
		imageJpg:   imageJpgObject(),
		empty:      emptyObject(),
	}
}

func smallTextObject() []byte {
	return []byte("test")
}

//go:embed fixture/medium-text.txt
var embedMediumText []byte

func mediumTextObject() []byte {
	return embedMediumText
}

func largeTextObject() []byte {
	text := strings.Repeat("0123456789\n", 100000)
	return []byte(text)
}

//go:embed fixture/small-html.html
var embedSmallHtml []byte

func smallHtmlObject() []byte {
	return embedSmallHtml
}

//go:embed fixture/rust-code.rs
var embedRustCode []byte

func rustCodeObject() []byte {
	return embedRustCode
}

func imagePngObject() []byte {
	var buf bytes.Buffer
	png.Encode(&buf, dummyImage())
	return buf.Bytes()
}

//go:embed fixture/icon.jpg
var embedIconImage []byte

func imageJpgObject() []byte {
	return embedIconImage
}

func dummyImage() image.Image {
	w, h := 600, 400
	img := image.NewRGBA(image.Rect(0, 0, w, h))
	for x := 0; x < w; x++ {
		for y := 0; y < h; y++ {
			var c color.Color
			if x < w/2 && y < h/2 {
				c = color.RGBA{R: 128, G: 128, B: 0, A: 100}
			} else if x >= w/2 && y < h/2 {
				c = color.RGBA{R: 128, G: 0, B: 128, A: 100}
			} else if x < w/2 && h >= y/2 {
				c = color.RGBA{R: 0, G: 128, B: 128, A: 100}
			} else {
				c = color.RGBA{R: 128, G: 128, B: 128, A: 100}
			}
			img.Set(x, y, c)
		}
	}
	return img
}

func emptyObject() []byte {
	return []byte{}
}

func defaultFixture() []bucket {
	return []bucket{
		newBucket("test-bucket-1", hierarchicalObjects()...),
		newBucket("test-bucket-2", variousContentTypeObjects()...),
		newBucket("test-bucket-3", largeNumberObjects()...),
		newBucket("test-bucket-4"),
		newBucket("test-bucket-5"),
		newBucket("test-bucket-6"),
		newBucket("test-bucket-7"),
		newBucket("test-bucket-8"),
		newBucket("test-bucket-9"),
		newBucket("test-bucket-a"),
		newBucket("test-bucket-b"),
		newBucket("test-bucket-c"),
		newBucket("test-bucket-d"),
		newBucket("test-bucket-e"),
		newBucket("test-bucket-f"),
		newBucket("test-bucket-g"),
		newBucket("test-bucket-h"),
		newBucket("test-bucket-i"),
		newBucket("test-bucket-j"),
		newBucket("test-bucket-k"),
		newBucket("test-bucket-l"),
		newBucket("test-bucket-m"),
		newBucket("test-bucket-n"),
		newBucket("test-bucket-o"),
		newBucket("test-bucket-p"),
		newBucket("test-bucket-q"),
		newBucket("test-bucket-r"),
		newBucket("test-bucket-s"),
		newBucket("test-bucket-t"),
		newBucket("test-bucket-u"),
		newBucket("test-bucket-v"),
		newBucket("test-bucket-w"),
		newBucket("test-bucket-x"),
		newBucket("test-bucket-y"),
		newBucket("test-bucket-z"),
	}
}

func hierarchicalObjects() []object {
	return []object{
		{
			objectKey:  "file_1.txt",
			objectType: smallText,
		},
		{
			objectKey:  "file_2.txt",
			objectType: smallText,
		},
		{
			objectKey:  "foo/file_1.txt",
			objectType: smallText,
		},
		{
			objectKey:  "foo/file_2.txt",
			objectType: smallText,
		},
		{
			objectKey:  "foo/file_3.txt",
			objectType: smallText,
		},
		{
			objectKey:  "foo/quux/garply/file_1.txt",
			objectType: smallText,
		},
		{
			objectKey:  "foo/quux/waldo/file_1.txt",
			objectType: smallText,
		},
		{
			objectKey:  "foo/quux/fred/file_1.rs",
			objectType: rustCode,
		},
		{
			objectKey:  "foo/quux/fred/file_2.txt",
			objectType: mediumText,
		},
		{
			objectKey:  "foo/quux/fred/plugh/file_1.txt",
			objectType: smallText,
		},
		{
			objectKey:  "foo/quux/fred/xyzzy/file_1.txt",
			objectType: smallText,
		},
		{
			objectKey:  "foo/quux/fred/thud/file_1.txt",
			objectType: smallText,
		},
		{
			objectKey:  "foo/corge/file_1.txt",
			objectType: smallText,
		},
		{
			objectKey:  "foo/grault/file_1.txt",
			objectType: smallText,
		},
		{
			objectKey:  "bar/file_1.txt",
			objectType: smallText,
		},
		{
			objectKey:  "bar/file_2.txt",
			objectType: smallText,
		},
		{
			objectKey:  "baz/file_1.txt",
			objectType: smallText,
		},
		{
			objectKey:  "qux/file_1.txt",
			objectType: smallText,
		},
	}
}

func variousContentTypeObjects() []object {
	return []object{
		{
			objectKey:  "small.txt",
			objectType: smallText,
		},
		{
			objectKey:  "medium.txt",
			objectType: mediumText,
		},
		{
			objectKey:  "large.txt",
			objectType: largeText,
		},
		{
			objectKey:  "hello.html",
			objectType: smallHtml,
		},
		{
			objectKey:       "image.png",
			objectType:      imagePng,
			multipleVersion: true,
		},
		{
			objectKey:       "image.jpeg",
			objectType:      imageJpg,
			multipleVersion: true,
		},
		{
			objectKey:  "empty.txt",
			objectType: empty,
		},
	}
}

func largeNumberObjects() []object {
	size := 3000
	objs := make([]object, size)
	for i := 0; i < size; i++ {
		objs[i] = object{
			objectKey:  fmt.Sprintf("file-%05d.txt", i),
			objectType: smallText,
		}
	}
	return objs
}
